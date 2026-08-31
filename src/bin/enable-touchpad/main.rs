//! enable-touchpad — Windows feasibility demo (single exe).
//!
//! One binary that embeds kanata as a library: holding `CapsLock` activates
//! the `mouse` layer and taps Ctrl+Win+F24, which the operating system /
//! touchpad driver maps to the soft touchpad enable/disable. The app provides
//! a tray icon, a small settings window for the layer key bindings, and file
//! logging. Non-Windows targets build a stub so the repository's host-side
//! gates keep passing unchanged.

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod kanata_embed;
#[cfg(windows)]
mod tray;

#[cfg(windows)]
fn main() {
    app::init_logging();
    tray::install();
    kanata_embed::start();
    tray::spawn_forwarder();
    app::launch();
}

#[cfg(not(windows))]
fn main() {
    println!("enable-touchpad is a Windows demo; see demo/README.md");
}
