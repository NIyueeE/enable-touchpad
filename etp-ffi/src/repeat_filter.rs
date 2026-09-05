//! Auto-repeat filter for the mouse layer.
//!
//! While the mouse layer is held, the physical layer keys (Q/W/E/...) are
//! held down, and Windows auto-repeat re-delivers their keydowns at ~30 Hz.
//! kanata's repeat handling re-emits the *currently held output* for each
//! repeat — for a held `mlft` that means re-injecting LEFT-DOWN thirty
//! times a second, which stutters touchpad drags (device-verified: only
//! large movements registered, while a *physical* mouse button held was
//! perfectly smooth).
//!
//! This module installs a low-level keyboard hook **after** kanata's. LL
//! hooks run last-installed-first, so this one sees auto-repeat keydowns of
//! the layer keys before kanata and eats them while the mouse layer is
//! active: the button is injected exactly once per press and drags are
//! clean. Key-ups always pass through, so kanata's state stays correct.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, AtomicUsize, Ordering};

/// The mouse layer is currently active (key held). When inactive, repeats
/// pass through so normal typing auto-repeat is untouched.
static LAYER_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Bit `i` = the key in `FILTER_KEYS[i]` is currently held down (as seen by
/// this hook). A keydown for an already-held key is an auto-repeat.
static HELD_BITS: AtomicU32 = AtomicU32::new(0);
/// Virtual-key codes currently bound to layer actions.
static FILTER_KEYS: Mutex<[u16; MAX_FILTER_KEYS]> = Mutex::new([0; MAX_FILTER_KEYS]);
/// How many entries of [`FILTER_KEYS`] are meaningful.
static FILTER_KEY_COUNT: AtomicUsize = AtomicUsize::new(0);
/// The installed hook handle, for `CallNextHookEx`.
static HHOOK: AtomicIsize = AtomicIsize::new(0);

/// At most the four action slots (plus headroom) are filterable.
const MAX_FILTER_KEYS: usize = 8;

const WM_KEYDOWN: usize = 0x0100;
const WM_KEYUP: usize = 0x0101;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_SYSKEYUP: usize = 0x0105;
/// `LLKHF_INJECTED` — events injected by SendInput (our own mouse clicks,
/// the toggle chord) must pass untouched.
const LLKHF_INJECTED: u32 = 0x10;

#[repr(C)]
struct KbdLlHookStruct {
    vk_code: u32,
    scan_code: u32,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[allow(unsafe_code)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowsHookExW(hook_type: i32, proc: usize, instance: isize, thread_id: u32) -> isize;
    fn CallNextHookEx(hhook: isize, code: i32, wparam: usize, lparam: isize) -> isize;
}

/// Configure which virtual keys are filtered while the mouse layer is
/// active. Derived from the current config on startup and on every apply.
/// Stale held-bits are cleared (the key set changed).
pub fn set_keys(vks: &[u16]) {
    let mut filtered = [0_u16; MAX_FILTER_KEYS];
    for (i, vk) in vks.iter().take(MAX_FILTER_KEYS).enumerate() {
        filtered[i] = *vk;
    }
    let count = filtered
        .iter()
        .position(|vk| *vk == 0)
        .unwrap_or(MAX_FILTER_KEYS);
    if let Ok(mut keys) = FILTER_KEYS.lock() {
        *keys = filtered;
        FILTER_KEY_COUNT.store(count, Ordering::Relaxed);
        HELD_BITS.store(0, Ordering::Relaxed);
        log::info!(
            "repeat filter keys: [{}] (count {count})",
            filtered
                .iter()
                .take(count)
                .map(|vk| format!("{vk:#04x}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// Mark the mouse layer active/inactive. Deactivating also clears the
/// held-key bits so a key still physically held cannot be mistaken for a
/// repeat after the layer exits.
pub fn set_active(active: bool) {
    if !active {
        HELD_BITS.store(0, Ordering::Relaxed);
    }
    LAYER_ACTIVE.store(active, Ordering::Relaxed);
}

/// Install the filter hook. Call **after** the embedded kanata started (LL
/// hooks run last-installed-first, so this one must be kanata's junior).
///
/// # Errors
///
/// Returns a message when the hook could not be installed.
// This module is inside the designated unsafe boundary; the unsafe blocks
// perform the hook install/continuation documented on each function.
#[allow(unsafe_code)]
pub fn install() -> Result<(), String> {
    // SAFETY: WH_KEYBOARD_LL (= 13) with the static hook procedure below;
    // its thread pumps messages via kanata's main loop.
    let handle = unsafe { SetWindowsHookExW(13, hook_proc as *const () as usize, 0, 0) };
    if handle == 0 {
        return Err("keyboard repeat filter hook failed to install".to_string());
    }
    HHOOK.store(handle, Ordering::Relaxed);
    log::info!("repeat filter hook installed (handle {handle:#x})");
    Ok(())
}

/// Decide whether a keydown for `vk` is an auto-repeat that should be eaten.
fn is_repeat_to_eat(vk: u32) -> bool {
    if !LAYER_ACTIVE.load(Ordering::Relaxed) {
        return false;
    }
    let keys = FILTER_KEYS.lock().map_or([0_u16; MAX_FILTER_KEYS], |k| *k);
    let count = FILTER_KEY_COUNT.load(Ordering::Relaxed);
    let Some(slot) = keys.iter().take(count).position(|key| *key == vk as u16) else {
        return false;
    };
    let bit = 1_u32 << slot;
    if HELD_BITS.load(Ordering::Relaxed) & bit != 0 {
        true // already held: this keydown is an auto-repeat
    } else {
        HELD_BITS.fetch_or(bit, Ordering::Relaxed);
        log::info!(
            "repeat filter: key {vk:#04x} pressed while the layer is active; \
             its auto-repeats will be eaten"
        );
        false // first press: let it through
    }
}

/// Clear the held-bit for `vk` on key-up so the next press passes through.
fn clear_held_bit(vk: u32) {
    let keys = FILTER_KEYS.lock().map_or([0_u16; MAX_FILTER_KEYS], |k| *k);
    let count = FILTER_KEY_COUNT.load(Ordering::Relaxed);
    if let Some(slot) = keys.iter().take(count).position(|key| *key == vk as u16) {
        HELD_BITS.fetch_and(!(1_u32 << slot), Ordering::Relaxed);
    }
}

/// LL keyboard hook procedure.
///
/// # Safety
///
/// Standard hook signature; `lparam` points to a `KbdLlHookStruct` provided
/// by Windows for the duration of the call.
#[allow(unsafe_code)]
unsafe extern "system" fn hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 {
        let msg = wparam;
        // SAFETY: Windows guarantees the struct is valid for the hook call.
        let info = unsafe { &*(lparam as *const KbdLlHookStruct) };
        let injected = info.flags & LLKHF_INJECTED != 0;
        if !injected {
            match msg {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    if is_repeat_to_eat(info.vk_code) {
                        return 1; // eat the auto-repeat
                    }
                }
                WM_KEYUP | WM_SYSKEYUP => clear_held_bit(info.vk_code),
                _ => {}
            }
        }
    }
    // SAFETY: mandatory continuation for LL hooks that do not block.
    unsafe { CallNextHookEx(HHOOK.load(Ordering::Relaxed), code, wparam, lparam) }
}
