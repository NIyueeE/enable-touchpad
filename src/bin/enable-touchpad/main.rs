//! enable-touchpad — Windows feasibility demo (single exe).
//!
//! Composition root: wires the platform adapter, input engine, touchpad
//! watchdog, tray, and settings UI together. The architecture is layered:
//!
//! 1. `etp-core` — pure domain logic (config model, key allowlist, kanata
//!    config generator), unit-tested on every host;
//! 2. `etp-platform` — the single platform-adaptation layer (OS paths,
//!    embedded input engine, touchpad state query). Future OS backends are
//!    added behind `#[cfg(...)]` in that crate;
//! 3. this binary — application layer (tray + Dioxus settings UI +
//!    watchdog), written against `etp_platform::Platform`.
//!
//! Non-Windows targets build a stub so the repository's host-side gates keep
//! passing unchanged.

// GUI subsystem: no console window pops up when the exe is double-clicked.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod config_store;
#[cfg(windows)]
mod cursor_badge;
#[cfg(windows)]
mod logging;
#[cfg(windows)]
mod tray;
#[cfg(windows)]
mod watchdog;

#[cfg(windows)]
use std::sync::{Arc, OnceLock};

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
    // Logging goes first: the data directory must exist before the lock check
    // so even a second launch leaves a trace in the log file.
    let platform = etp_platform::current();
    logging::init(platform);

    if !acquire_single_instance_lock() {
        log::error!("another enable-touchpad instance is already running; exiting");
        std::process::exit(1);
    }
    log::info!("single-instance lock acquired");
    log::info!("app starting");

    let watchdog = Arc::new(watchdog::WatchdogState::new(platform));
    let cfg = config_store::load(platform);
    watchdog.set_managed(cfg.feature_enabled);

    let (layer_tx, layer_rx) = std::sync::mpsc::sync_channel(8);
    if let Err(e) = platform.start_engine(&cfg, layer_tx) {
        log::error!("input engine failed to start: {e}");
    } else {
        watchdog::spawn(Arc::clone(&watchdog), layer_rx);
    }

    tray::install();
    tray::spawn_forwarder(platform, Arc::clone(&watchdog));
    // Cosmetic: the cursor badge fails soft — log and run without it.
    if let Err(e) = cursor_badge::start() {
        log::warn!("{e}; running without the layer cursor badge");
    }
    app::launch(platform, Arc::clone(&watchdog));
}

#[cfg(not(windows))]
fn main() {
    println!("enable-touchpad is a Windows demo; see demo/README.md");
}
