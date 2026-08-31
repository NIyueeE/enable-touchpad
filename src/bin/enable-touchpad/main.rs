//! enable-touchpad — Windows feasibility demo (single exe).
//!
//! One binary that embeds kanata as a library: holding `CapsLock` activates
//! the `mouse` layer and taps Ctrl+Win+F24, which the operating system /
//! touchpad driver maps to the soft touchpad enable/disable. The app provides
//! a tray icon, a small settings window for the layer key bindings, and file
//! logging. Non-Windows targets build a stub so the repository's host-side
//! gates keep passing unchanged.

// GUI subsystem: no console window pops up when the exe is double-clicked.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
use std::sync::OnceLock;

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod kanata_embed;
#[cfg(windows)]
mod tray;

/// Sentinel port held open for the process lifetime: if another instance
/// already bound it, this one exits instead of double-capturing keys.
#[cfg(windows)]
const INSTANCE_LOCK_PORT: u16 = 58270;

#[cfg(windows)]
static INSTANCE_LOCK: OnceLock<std::net::TcpListener> = OnceLock::new();

#[cfg(windows)]
fn acquire_single_instance_lock() -> bool {
    match std::net::TcpListener::bind(("127.0.0.1", INSTANCE_LOCK_PORT)) {
        Ok(listener) => INSTANCE_LOCK.set(listener).is_ok(),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn main() {
    if !acquire_single_instance_lock() {
        std::process::exit(1);
    }
    app::init_logging();
    log::info!("single-instance lock acquired");
    tray::install();
    kanata_embed::start();
    tray::spawn_forwarder();
    app::launch();
}

#[cfg(not(windows))]
fn main() {
    println!("enable-touchpad is a Windows demo; see demo/README.md");
}
