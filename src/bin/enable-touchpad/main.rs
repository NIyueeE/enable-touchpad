//! enable-touchpad — Windows feasibility demo (Dioxus UI + kanata layer signal).
//!
//! Hold `CapsLock`: the kanata `mouse` layer activates, the touchpad is enabled,
//! and a small click-through indicator follows the mouse. Release `CapsLock`:
//! the layer is restored and the touchpad is disabled again.
//!
//! Non-Windows targets build a stub binary so the repository's host-side gates
//! (fmt / machete / docs / clippy / test on Linux) keep passing unchanged;
//! the real application only compiles for `cfg(windows)`.

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod kanata_embed;
#[cfg(windows)]
mod signal;
#[cfg(windows)]
mod touchpad;
#[cfg(windows)]
mod tray;

// Feature unification only: pulls in `dioxus-desktop/transparent` (needed for
// the see-through indicator window); the crate's API is reached through
// `dioxus::desktop`.
#[cfg(windows)]
use dioxus_desktop as _;

#[cfg(windows)]
fn main() {
    tray::install();
    kanata_embed::start();
    signal::spawn_all();
    tray::spawn_forwarder();
    app::launch();
}

#[cfg(not(windows))]
fn main() {
    println!("enable-touchpad is a Windows demo; see demo/README.md");
}
