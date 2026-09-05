//! Single-instance guard via a named Win32 mutex.
//!
//! The previous sentinel (a `TcpListener` bound to a fixed loopback port)
//! was unreliable on Windows: the standard library sets `SO_REUSEADDR` for
//! `TcpListener`, which lets a second process bind the very same port and
//! start a second instance — two engines, two keyboard hooks, two
//! watchdogs racing on the touchpad toggle (all device-observed).
//!
//! A named mutex is the canonical Windows single-instance primitive and is
//! immune to socket-option semantics.

use std::sync::atomic::{AtomicIsize, Ordering};

/// Handle of the instance mutex, held for the process lifetime.
static MUTEX_HANDLE: AtomicIsize = AtomicIsize::new(0);

const ERROR_ALREADY_EXISTS: u32 = 183;

#[allow(unsafe_code)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(attributes: isize, initial_owner: i32, name: *const u16) -> isize;
    fn GetLastError() -> u32;
}

/// Try to become the single running instance.
///
/// On success the named mutex is held until the process exits. Returns an
/// error when another instance already owns it (the caller should log and
/// exit).
///
/// # Errors
///
/// Returns `"instance already running"` when the mutex already exists, or
/// the Win32 error code when creation fails.
// This module is inside the designated unsafe boundary; the unsafe block
// performs the single documented mutex creation and reads its immediate
// error state.
#[allow(unsafe_code)]
pub fn acquire(name: &str) -> Result<(), String> {
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    // SAFETY: `wide` is a NUL-terminated name owned here; the mutex handle
    // is stored in a static and intentionally never released — the OS
    // reclaims it at process exit, which is exactly the lifetime we want.
    // GetLastError is read immediately after the call, before anything else
    // can disturb the thread's error state.
    let (handle, last_error) = unsafe {
        let handle = CreateMutexW(0, 1, wide.as_ptr());
        (handle, GetLastError())
    };
    if last_error == ERROR_ALREADY_EXISTS {
        return Err("instance already running".to_string());
    }
    MUTEX_HANDLE.store(handle, Ordering::Relaxed);
    Ok(())
}
