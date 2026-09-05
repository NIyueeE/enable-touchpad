//! System tray: right-click menu with only "open settings" and "quit".

use crate::app;
use crate::watchdog::WatchdogState;
use etp_platform::Platform;
use std::sync::Arc;
use std::time::Duration;
use tray_icon::TrayIconBuilder;
use tray_icon::TrayIconEvent;
use tray_icon::menu::{IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};

/// Actions the tray can request from the app.
#[derive(Debug, Clone, Copy)]
pub enum TrayAction {
    /// Show and focus the settings window.
    OpenSettings,
    /// Exit the app.
    Quit,
}

/// Build the tray icon and menu. Call once from `main`; the handle is leaked
/// on purpose because `TrayIcon` is `!Send` and must simply outlive `main`.
///
/// Failure is loud, not silent: the settings window starts hidden and the
/// tray is its only door — an installation failure means the user has no way
/// to reach the app at all, so it must at least land in the log.
pub fn install() {
    let settings = MenuItem::with_id("et-settings", "打开设置", true, None);
    let quit = MenuItem::with_id("et-quit", "退出", true, None);
    let items: [&dyn IsMenuItem; 3] = [&settings, &PredefinedMenuItem::separator(), &quit];
    let menu = Menu::new();
    for item in items {
        let _ = menu.append(item);
    }
    // Prefer the designed icon that build.rs embeds into the exe as icon
    // resource 1; fall back to the in-process disc when the resource is
    // missing (e.g. a build where no resource compiler was available).
    let icon = match tray_icon::Icon::from_resource(1, Some((32, 32))) {
        Ok(icon) => icon,
        Err(e) => {
            log::warn!("icon resource 1 unavailable ({e:?}); using generated fallback");
            match tray_icon::Icon::from_rgba(icon_rgba(), 32, 32) {
                Ok(icon) => icon,
                Err(e) => {
                    log::error!("tray icon creation failed: {e}");
                    return;
                }
            }
        }
    };
    match TrayIconBuilder::new()
        .with_id("enable-touchpad-tray")
        .with_menu(Box::new(menu))
        .with_tooltip("enable-touchpad")
        .with_menu_on_left_click(false)
        .with_icon(icon)
        .build()
    {
        // `TrayIcon` is `!Send`; leaking it keeps the tray alive for the
        // process lifetime without a `'static` self-reference.
        Ok(tray) => std::mem::forget(tray),
        Err(e) => log::error!("tray installation failed: {e}"),
    }
}

/// Poll muda's menu queue and tray-icon's click queue and dispatch selections.
pub fn spawn_forwarder(platform: &'static dyn Platform, state: Arc<WatchdogState>) {
    std::thread::spawn(move || {
        loop {
            while let Ok(event) = MenuEvent::receiver().try_recv()
                && let Some(action) = decode(&event.id.0)
            {
                app::handle_tray(action, platform, &state);
            }
            while let Ok(event) = TrayIconEvent::receiver().try_recv()
                && let Some(action) = decode_click(&event)
            {
                app::handle_tray(action, platform, &state);
            }
            std::thread::sleep(Duration::from_millis(120));
        }
    });
}

fn decode(id: &str) -> Option<TrayAction> {
    match id {
        "et-settings" => Some(TrayAction::OpenSettings),
        "et-quit" => Some(TrayAction::Quit),
        _ => None,
    }
}

/// A plain left-click **release** on the tray icon opens the settings window
/// (the right button owns the context menu).
fn decode_click(event: &TrayIconEvent) -> Option<TrayAction> {
    match event {
        TrayIconEvent::Click {
            button: tray_icon::MouseButton::Left,
            button_state: tray_icon::MouseButtonState::Up,
            ..
        } => Some(TrayAction::OpenSettings),
        _ => None,
    }
}

/// A 32x32 blue disc with a darker rim, generated in-process so the app
/// ships no binary assets. Also reused as the settings-window/taskbar icon.
pub(crate) fn icon_rgba() -> Vec<u8> {
    let mut data = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32i32 {
        for x in 0..32i32 {
            let dx = f64::from(x) - 15.5;
            let dy = f64::from(y) - 15.5;
            let dist2 = dx * dx + dy * dy;
            let (r, g, b, a) = if dist2 <= 169.0 {
                (0x22, 0x6b, 0xe6, 0xff)
            } else if dist2 <= 196.0 {
                (0x14, 0x3c, 0x8a, 0xff)
            } else {
                (0x00, 0x00, 0x00, 0x00)
            };
            data.extend_from_slice(&[r, g, b, a]);
        }
    }
    data
}
