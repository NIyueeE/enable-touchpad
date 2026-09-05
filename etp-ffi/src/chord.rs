//! Direct Ctrl+Win+F24 chord injection (the touchpad soft toggle).
//!
//! Mirrors kanata's proven Windows output path exactly
//! (`oskbd/windows::send_key_sendinput` with the
//! `win_sendinput_send_scancodes` feature): scancodes from
//! `MapVirtualKeyW(VK, MAPVK_VK_TO_VSC)`, `KEYEVENTF_SCANCODE`, and
//! `KEYEVENTF_EXTENDEDKEY` for keys in kanata's extended list (LWin).
//! Events go out one `SendInput` per key with a small gap between them —
//! the same pacing a kanata macro produces on the processing loop.
//!
//! This deliberately bypasses kanata's `ActOnFakeKey` TCP command: the
//! config-macro chord is device-proven, while the fake-key-over-TCP path
//! was never exercised on real hardware until it visibly failed.

/// Virtual-key codes for the chord.
const VK_LCONTROL: u32 = 0xA2;
const VK_LWIN: u32 = 0x5B;
const VK_F24: u32 = 0x87;

const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_SCANCODE: u32 = 0x0008;
/// `MAPVK_VK_TO_VSC` — translate a virtual key to its scancode.
const MAPVK_VK_TO_VSC: u32 = 0;
/// Gap between injected events (ms); kanata macrospace their steps on the
/// processing loop at a similar cadence.
const EVENT_GAP_MS: u64 = 12;

/// Keys kanata treats as extended (`EXTENDED_KEYS` in its oskbd windows
/// backend) — LWin is the only chord member on that list.
fn is_extended(scancode: u32) -> bool {
    // The relevant slice of kanata's EXTENDED_KEYS table (VK low bytes):
    // includes 0x5b (LWin) and 0x5c (RWin) among others.
    matches!(scancode, 0x5b | 0x5c)
}

#[repr(C)]
struct KeyboardInput {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    extra_info: usize,
}

/// Mirror of Win32 `INPUT` with the keyboard union member active. The size
/// assert keeps the layout honest (x64 `INPUT` is 40 bytes because the
/// union's `MOUSEINPUT` member is the largest).
#[repr(C)]
struct Input {
    input_type: u32,
    _pad: u32,
    ki: KeyboardInput,
    _union_tail: u64,
}

const _: () = assert!(size_of::<Input>() == 40);

#[allow(unsafe_code)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SendInput(inputs: i32, array: *const Input, size: i32) -> i32;
    fn MapVirtualKeyW(vk: u32, map_type: u32) -> u32;
}

/// Inject one key event for `vk` (scancode-based, kanata-mirrored).
// This module is inside the designated unsafe boundary; the two calls below
// only read the given constants and copy one stack INPUT synchronously.
#[allow(unsafe_code)]
fn send_one(vk: u32, key_up: bool) -> Result<(), String> {
    // SAFETY: MapVirtualKeyW is a pure translation call.
    let scancode = unsafe { MapVirtualKeyW(vk, MAPVK_VK_TO_VSC) };
    if scancode == 0 {
        return Err(format!("MapVirtualKeyW returned 0 for VK {vk:#x}"));
    }
    let mut flags = KEYEVENTF_SCANCODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    if is_extended(scancode) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    let input = Input {
        input_type: INPUT_KEYBOARD,
        _pad: 0,
        ki: KeyboardInput {
            w_vk: 0,
            w_scan: scancode as u16,
            dw_flags: flags,
            time: 0,
            extra_info: 0,
        },
        _union_tail: 0,
    };
    // SAFETY: `input` is a fully initialised 40-byte INPUT mirror and one
    // event is requested; SendInput copies the array synchronously.
    // SAFETY: `input` is a fully initialised 40-byte INPUT mirror and one
    // event is requested; SendInput copies the array synchronously.
    let sent = unsafe { SendInput(1, &input, size_of::<Input>() as i32) };
    if sent != 1 {
        return Err(format!("SendInput sent {sent} of 1 events for VK {vk:#x}"));
    }
    Ok(())
}

/// Tap the full Ctrl+Win+F24 chord: keys down in order, released in reverse,
/// with a small gap between events. This is the same output the proven
/// kanata config macro produced on the device.
pub fn tap() -> Result<(), String> {
    for vk in [VK_LCONTROL, VK_LWIN, VK_F24] {
        send_one(vk, false)?;
        std::thread::sleep(std::time::Duration::from_millis(EVENT_GAP_MS));
    }
    for vk in [VK_F24, VK_LWIN, VK_LCONTROL] {
        send_one(vk, true)?;
        std::thread::sleep(std::time::Duration::from_millis(EVENT_GAP_MS));
    }
    Ok(())
}
