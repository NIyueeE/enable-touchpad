//! Main-thread task door + async window operations.
//!
//! The tray lives on the main thread (`Rc<RefCell>` inside tray-icon makes
//! cross-thread mutation unsound), and window shows must not hop through
//! tao's blocking thread-executor. Both are solved by a tiny **message-only
//! window** created on the main thread: foreign threads `PostMessageW` task
//! codes to it, and its window procedure runs a registered handler on the
//! main thread's message pump (the dioxus event loop drains that queue
//! already).

use std::sync::Mutex;
use std::sync::atomic::{AtomicIsize, Ordering};

/// Apply the master switch to the tray visuals (`param`: 0 = off, 1 = on).
pub const TASK_APPLY_MASTER_VISUALS: usize = 1;

/// Show, restore and foreground the settings window.
pub const TASK_OPEN_SETTINGS: usize = 2;

static DOOR_HWND: AtomicIsize = AtomicIsize::new(0);

/// Signature of the main-thread door handler.
type DoorHandler = fn(usize, usize);

static HANDLER: Mutex<Option<DoorHandler>> = Mutex::new(None);

const WM_APP_BASE: u32 = 0x8000;
const SW_RESTORE: i32 = 9;
const SW_SHOW: i32 = 5;

#[repr(C)]
struct WndClassW {
    style: u32,
    wndproc: usize,
    cls_extra: i32,
    wnd_extra: i32,
    instance: isize,
    icon: isize,
    cursor: isize,
    background: isize,
    menu_name: *const u16,
    class_name: *const u16,
}

#[allow(unsafe_code)]
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        param: *mut std::ffi::c_void,
    ) -> isize;
    fn DefWindowProcW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn PostMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn ShowWindow(hwnd: isize, cmd: i32) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn GetModuleHandleW(name: *const u16) -> isize;
}

/// Register the main-thread handler and create the door window. Call once
/// from `main` (the main thread) before anything posts tasks.
///
/// # Errors
///
/// Returns a message when the window/class could not be created.
// This module is inside the designated unsafe boundary; the unsafe blocks
// here perform the window setup and message post documented above.
#[allow(unsafe_code)]
pub fn init(handler: fn(usize, usize)) -> Result<(), String> {
    *HANDLER.lock().map_err(|_| "door handler mutex poisoned")? = Some(handler);
    // SAFETY: plain Win32 class/window setup with constants owned here; the
    // window procedure below only touches the statics of this module.
    let hwnd = unsafe { create_door() }?;
    DOOR_HWND.store(hwnd, Ordering::Relaxed);
    Ok(())
}

/// Queue a task onto the main thread. Non-blocking and thread-safe; a no-op
/// before [`init`].
#[allow(unsafe_code)]
pub fn post(task: usize, param: usize) {
    let hwnd = DOOR_HWND.load(Ordering::Relaxed);
    if hwnd == 0 {
        return;
    }
    // SAFETY: posting to our own door window; the handler was registered in
    // `init` and runs on the main thread's message pump.
    unsafe {
        PostMessageW(hwnd, WM_APP_BASE, task, param as isize);
    }
}

/// Show, restore and foreground the settings window. Plain async Win32 —
/// safe to call from any thread, never blocks the caller.
#[allow(unsafe_code)]
pub fn show_and_activate(hwnd: isize) {
    // SAFETY: `hwnd` is the caller's own settings window; both calls are
    // plain asynchronous Win32 operations.
    unsafe {
        ShowWindow(hwnd, SW_RESTORE);
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }
}

/// Create a message-only window (HWND_MESSAGE parent) that dispatches
/// posted task codes to the registered handler.
///
/// # Safety
///
/// Plain Win32 class/window setup with constants owned by this module.
#[allow(unsafe_code)]
#[expect(
    unsafe_op_in_unsafe_fn,
    reason = "the entire function is one documented Win32 setup sequence"
)]
unsafe fn create_door() -> Result<isize, String> {
    static CLASS_NAME: &[u16] = &[
        b'e' as u16,
        b't' as u16,
        b'p' as u16,
        b'-' as u16,
        b'd' as u16,
        b'o' as u16,
        b'o' as u16,
        b'r' as u16,
        0,
    ];
    let class = WndClassW {
        style: 0,
        wndproc: door_wndproc as *const () as usize,
        cls_extra: 0,
        wnd_extra: 0,
        instance: GetModuleHandleW(std::ptr::null()),
        icon: 0,
        cursor: 0,
        background: 0,
        menu_name: std::ptr::null(),
        class_name: CLASS_NAME.as_ptr(),
    };
    if RegisterClassW(&class) == 0 {
        return Err("door RegisterClassW failed".to_string());
    }
    let hwnd = CreateWindowExW(
        0,
        CLASS_NAME.as_ptr(),
        std::ptr::null(),
        0, // not visible, not top-level: message-only via parent below
        0,
        0,
        0,
        0,
        -3, // HWND_MESSAGE
        0,
        class.instance,
        std::ptr::null_mut(),
    );
    if hwnd == 0 {
        return Err("door CreateWindowExW failed".to_string());
    }
    Ok(hwnd)
}

/// Window procedure of the door: dispatch task codes to the handler.
///
/// # Safety
///
/// Standard WndProc signature; only touches this module's statics.
#[allow(unsafe_code)]
unsafe extern "system" fn door_wndproc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if msg == WM_APP_BASE
        && let Ok(guard) = HANDLER.lock()
        && let Some(handler) = *guard
    {
        handler(wparam, lparam as usize);
        return 0;
    }
    // SAFETY: default processing for non-door messages.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
