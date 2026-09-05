//! Click-through cursor badge: a 16px always-on-top overlay pinned to the
//! bottom-right of the mouse cursor while the mouse layer is held.
//!
//! Design notes:
//! - The overlay is a `WS_POPUP` layered window (`WS_EX_LAYERED |
//!   TRANSPARENT | TOPMOST | TOOLWINDOW | NOACTIVATE`): it never takes
//!   focus, never receives mouse input, and stays out of Alt-Tab.
//! - Pixels go through `UpdateLayeredWindow` once at startup, so the window
//!   has no paint cycle; repositioning is a single `SetWindowPos` blit.
//! - One dedicated thread owns the window for its whole life: it pumps the
//!   message queue, polls `GetCursorPos` at ~125 Hz and follows the cursor,
//!   but only while the shared flag is set (the layer is held).
//! - The badge appears only after the request has been up for `SHOW_DELAY`,
//!   so quick CapsLock taps do not flash it, and hides on the first tick
//!   after the request drops.
//! - Nothing survives a process kill: the window dies with the thread and
//!   no system state (cursor scheme, devices, ...) is ever modified.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// The badge is anchored like the native Help Select cursor: hugging the
/// arrow glyph's right side from roughly mid-height down. The arrow glyph
/// occupies about the left 45% of the sprite box (SM_CXCURSOR/SM_CYCURSOR,
/// DPI-scaled), so the badge starts just inside that edge, halfway down.
/// Reposition cadence: one `GetCursorPos` + one `SetWindowPos` per tick,
/// only while the badge is visible.
const POLL: Duration = Duration::from_millis(8);
/// Show the badge only after the layer request stayed up this long.
const SHOW_DELAY: Duration = Duration::from_millis(150);
/// Shared request: `true` while the mouse layer is held (watchdog thread).
static VISIBLE: AtomicBool = AtomicBool::new(false);

// Win32 handles, kept as `isize` so every struct stays `#[repr(C)]`-simple.
// (Rust-side aliases use clippy-friendly casing; the SDK names appear in
// the extern declarations' comments.)
type Hwnd = isize;
type Hdc = isize;
type Hgdiobj = isize;
type Hinstance = isize;
type Handle = isize;
type Bool = i32;
type Lresult = isize;
type Wparam = usize;
type Lparam = isize;

#[derive(Clone, Copy)]
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Size {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    time: u32,
    pt: Point,
}

#[repr(C)]
struct BlendFunction {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

#[repr(C)]
struct WndClass {
    style: u32,
    wndproc: usize,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Hinstance,
    icon: Hgdiobj,
    cursor: Hgdiobj,
    background: Hgdiobj,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_ppm: i32,
    y_ppm: i32,
    clr_used: u32,
    clr_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
}

const _: () = assert!(size_of::<BitmapInfoHeader>() == 40);

// User32 surface. Every call site carries its own safety argument; this
// module is inside the designated unsafe boundary of the workspace.
#[allow(unsafe_code)]
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClass) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Hinstance,
        param: *mut std::ffi::c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, msg: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn GetModuleHandleW(name: *const u16) -> Hinstance;
    fn GetCursorPos(point: *mut Point) -> Bool;
    fn GetSystemMetrics(index: i32) -> i32;
    fn SetWindowPos(hwnd: Hwnd, after: Hwnd, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> Bool;
    fn UpdateLayeredWindow(
        hwnd: Hwnd,
        dc_dst: Hdc,
        pt_dst: *const Point,
        size: *const Size,
        dc_src: Hdc,
        pt_src: *const Point,
        color_key: u32,
        blend: *const BlendFunction,
        flags: u32,
    ) -> Bool;
    fn GetDC(hwnd: Hwnd) -> Hdc;
    fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    fn PeekMessageW(msg: *mut Msg, hwnd: Hwnd, min: u32, max: u32, remove: u32) -> Bool;
    fn TranslateMessage(msg: *const Msg) -> Bool;
    fn DispatchMessageW(msg: *const Msg) -> Lresult;
}

// GDI surface for the premultiplied-alpha pixel source of the layered
// window.
#[allow(unsafe_code)]
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: Hdc) -> Hdc;
    fn DeleteDC(dc: Hdc) -> Bool;
    fn CreateDIBSection(
        dc: Hdc,
        info: *const BitmapInfo,
        usage: u32,
        bits: *mut *mut u8,
        section: Handle,
        offset: u32,
    ) -> Hgdiobj;
    fn SelectObject(dc: Hdc, obj: Hgdiobj) -> Hgdiobj;
    fn DeleteObject(obj: Hgdiobj) -> Bool;
}

// Style / flag constants (WinUser.h).
const WS_POPUP: u32 = 0x8000_0000;
/// `SetWindowPos` insert-after handle: keep the badge in the topmost band
/// (asserted on every move — `WS_EX_TOPMOST` alone does not always stick).
const TOPMOST_INSERT: Hwnd = -1;
const WS_EX_LAYERED: u32 = 0x0008_0000;
const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
const WS_EX_TOPMOST: u32 = 0x0000_0008;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SWP_HIDEWINDOW: u32 = 0x0080;
const PM_REMOVE: u32 = 1;
const DIB_RGB_COLORS: u32 = 0;
const ULW_ALPHA: u32 = 2;
const AC_SRC_OVER: u8 = 0;
const AC_SRC_ALPHA: u8 = 1;

/// Show or hide the badge. Safe to call from any thread at any rate; the
/// owner thread picks it up on its next poll tick.
pub fn set_visible(visible: bool) {
    VISIBLE.store(visible, Ordering::Relaxed);
}

/// Create the overlay window on a dedicated thread and enter the follow
/// loop. `rgba` is `width * height` pixels of straight-alpha RGBA (exactly
/// what a PNG decoder yields). Returns once the window is up (or with the
/// creation error); the thread then runs for the process lifetime.
///
/// # Errors
///
/// Returns a human-readable message when the window or its pixel surface
/// could not be created (missing Win32 desktop, class registration, ...).
pub fn start(rgba: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::Relaxed) {
        return Ok(());
    }
    let expected = usize::try_from(width).map_or(usize::MAX, |w| {
        usize::try_from(height).map_or(usize::MAX, |h| w * h * 4)
    });
    if expected != rgba.len() {
        return Err(format!(
            "badge pixel buffer is {} bytes, expected {expected} for {width}x{height}",
            rgba.len()
        ));
    }
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let bgra = premultiply_bgra(&rgba);
    std::thread::Builder::new()
        .name("etp-cursor-badge".to_string())
        .spawn(move || owner_thread(bgra, width, height, tx))
        .map_err(|e| format!("badge thread spawn failed: {e}"))?;
    rx.recv().map_err(|_| "badge thread died".to_string())?
}

/// Body of the badge owner thread: create the overlay, report readiness,
/// then follow the cursor until the process exits.
// Safe bridge into the unsafe overlay functions below; the unsafe blocks
// here perform exactly the operations documented on those functions.
#[allow(unsafe_code)]
fn owner_thread(bgra: Vec<u8>, width: u32, height: u32, ready: Sender<Result<(), String>>) {
    let outcome = unsafe { create_overlay(&bgra, width, height) };
    let hwnd = match outcome {
        Ok(hwnd) => hwnd,
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        // Caller died before the answer; keep serving the badge anyway.
    }
    unsafe { follow_loop(hwnd) };
}

/// Encode the class name as a NUL-terminated UTF-16 string.
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Create the layered window and hand the badge pixels to it. Runs on the
/// owner thread before the follow loop starts.
///
/// # Safety
///
/// Plain Win32 window/GDI setup; every pointer argument is constructed in
/// this function and used only for the duration of the calls.
#[allow(unsafe_code)]
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "the entire function is one documented Win32 setup sequence;               wrapping every call would repeat the same header-level argument"
)]
unsafe fn create_overlay(bgra: &[u8], width: u32, height: u32) -> Result<Hwnd, String> {
    let class_name = wide("etp-cursor-badge");
    let class = WndClass {
        style: 0,
        wndproc: DefWindowProcW as *const () as usize,
        cls_extra: 0,
        wnd_extra: 0,
        instance: GetModuleHandleW(std::ptr::null()),
        icon: 0,
        cursor: 0,
        background: 0,
        menu_name: std::ptr::null(),
        class_name: class_name.as_ptr(),
    };
    if RegisterClassW(&class) == 0 {
        return Err("RegisterClassW failed".to_string());
    }
    // Created hidden: the follow loop shows it when the badge becomes due.
    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        class_name.as_ptr(),
        std::ptr::null(),
        WS_POPUP,
        0,
        0,
        width as i32,
        height as i32,
        0,
        0,
        class.instance,
        std::ptr::null_mut(),
    );
    if hwnd == 0 {
        return Err("CreateWindowExW failed".to_string());
    }

    // Top-down 32bpp DIB with premultiplied BGRA pixels.
    let screen_dc = GetDC(0);
    if screen_dc == 0 {
        return Err("GetDC failed".to_string());
    }
    let mem_dc = CreateCompatibleDC(screen_dc);
    if mem_dc == 0 {
        ReleaseDC(0, screen_dc);
        return Err("CreateCompatibleDC failed".to_string());
    }
    let info = BitmapInfo {
        header: BitmapInfoHeader {
            size: 40,
            width: width as i32,
            height: -(height as i32), // negative: top-down rows
            planes: 1,
            bit_count: 32,
            compression: 0, // BI_RGB
            size_image: 0,
            x_ppm: 0,
            y_ppm: 0,
            clr_used: 0,
            clr_important: 0,
        },
    };
    let mut bits: *mut u8 = std::ptr::null_mut();
    let dib = CreateDIBSection(mem_dc, &info, DIB_RGB_COLORS, &mut bits, 0, 0);
    if dib == 0 || bits.is_null() {
        DeleteDC(mem_dc);
        ReleaseDC(0, screen_dc);
        return Err("CreateDIBSection failed".to_string());
    }
    // SAFETY: the DIB is width*height*4 bytes (32bpp, same row stride as
    // `bgra`), so this copy stays inside the allocation.
    std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits, bgra.len());
    let old = SelectObject(mem_dc, dib);
    let blend = BlendFunction {
        blend_op: AC_SRC_OVER,
        blend_flags: 0,
        source_constant_alpha: 255,
        alpha_format: AC_SRC_ALPHA,
    };
    let src_pt = Point { x: 0, y: 0 };
    let dst_pt = Point { x: 0, y: 0 };
    let size = Size {
        cx: width as i32,
        cy: height as i32,
    };
    let ok = UpdateLayeredWindow(
        hwnd, screen_dc, &dst_pt, &size, mem_dc, &src_pt, 0, &blend, ULW_ALPHA,
    );
    SelectObject(mem_dc, old);
    DeleteObject(dib);
    DeleteDC(mem_dc);
    ReleaseDC(0, screen_dc);
    if ok == 0 {
        return Err("UpdateLayeredWindow failed".to_string());
    }
    Ok(hwnd)
}

/// Owner-thread main loop: pump messages, follow the cursor while requested,
/// hide otherwise. Runs until the process exits.
///
/// # Safety
///
/// All calls take `hwnd` from [`create_overlay`] and fixed constants.
#[allow(unsafe_code)]
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "every call in the loop targets the overlay window created by               create_overlay; per-call blocks would repeat that one argument"
)]
unsafe fn follow_loop(hwnd: Hwnd) {
    let mut msg = Msg {
        hwnd: 0,
        message: 0,
        wparam: 0,
        lparam: 0,
        time: 0,
        pt: Point { x: 0, y: 0 },
    };
    let mut window_on = false;
    let mut requested_since: Option<Instant> = None;
    loop {
        while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let due = match VISIBLE.load(Ordering::Relaxed) {
            true => requested_since.get_or_insert_with(Instant::now).elapsed() >= SHOW_DELAY,
            false => {
                requested_since = None;
                false
            }
        };
        match (due, window_on) {
            (true, false) => {
                window_on = true;
                move_to_cursor(hwnd, SWP_SHOWWINDOW);
            }
            (true, true) => move_to_cursor(hwnd, 0),
            (false, true) => {
                window_on = false;
                SetWindowPos(
                    hwnd,
                    TOPMOST_INSERT,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOACTIVATE | SWP_HIDEWINDOW,
                );
            }
            (false, false) => {}
        }
        std::thread::sleep(POLL);
    }
}

/// Reposition the badge to cursor + offset (and optionally show it).
///
/// # Safety
///
/// `hwnd` is the overlay created by [`create_overlay`].
#[allow(unsafe_code)]
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "both calls act on the same documented overlay window handle"
)]
unsafe fn move_to_cursor(hwnd: Hwnd, extra: u32) {
    // SM_CXCURSOR = 13, SM_CYCURSOR = 14: current cursor sprite size,
    // already DPI-scaled for this process. Anchor like the native Help
    // Select cursor: start just inside the arrow glyph's right edge (the
    // glyph occupies about the left 45% of the sprite box), halfway down.
    let (cx, cy) = (GetSystemMetrics(13), GetSystemMetrics(14));
    let mut pt = Point { x: 0, y: 0 };
    if GetCursorPos(&mut pt) != 0 {
        SetWindowPos(
            hwnd,
            TOPMOST_INSERT,
            pt.x + cx * 45 / 100 - 2,
            pt.y + cy / 2,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | extra,
        );
    }
}

/// Straight-alpha RGBA to premultiplied-alpha BGRA (what
/// `UpdateLayeredWindow` + `AC_SRC_ALPHA` expect).
#[must_use]
pub fn premultiply_bgra(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|px| {
            let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
            let premul =
                |c: u8| u8::try_from((u16::from(c) * u16::from(a) + 127) / 255).unwrap_or(255);
            [b, g, r].map(premul).into_iter().chain(std::iter::once(a))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::premultiply_bgra;

    #[test]
    fn premultiplies_towards_black_and_keeps_alpha() {
        // Fully opaque: channels unchanged (RGBA -> BGRA swap only).
        assert_eq!(premultiply_bgra(&[255, 64, 32, 255]), [32, 64, 255, 255]);
        // Half alpha: channels scale towards zero, alpha passes through.
        assert_eq!(premultiply_bgra(&[255, 64, 32, 128]), [16, 32, 127, 128]);
        // Fully transparent: everything black.
        assert_eq!(premultiply_bgra(&[255, 255, 255, 0]), [0, 0, 0, 0]);
    }
}
