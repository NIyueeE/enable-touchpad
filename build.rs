//! Build script: embeds `assets/icon.ico` as the executable's first icon
//! resource so Explorer, the taskbar, and the tray
//! ([`tray_icon::Icon::from_resource`]) all pick up the designed icon.
//!
//! Compiling the resource is deliberately not fatal: on hosts without a
//! Win32 resource compiler (e.g. Linux-host cross checks) a
//! `cargo:warning` is emitted and the build continues without it.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=assets/icon.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed icon.ico resource: {e}");
        }
    }
}
