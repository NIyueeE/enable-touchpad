//! System tray: right-click menu with only "open settings" and "quit".

use crate::app;
use std::time::Duration;
use tray_icon::TrayIconBuilder;
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
pub fn install() {
    let settings = MenuItem::with_id("et-settings", "打开设置", true, None);
    let quit = MenuItem::with_id("et-quit", "退出", true, None);
    let items: [&dyn IsMenuItem; 3] = [&settings, &PredefinedMenuItem::separator(), &quit];
    let menu = Menu::new();
    for item in items {
        let _ = menu.append(item);
    }
    if let Ok(icon) = tray_icon::Icon::from_rgba(icon_rgba(), 32, 32)
        && let Ok(tray) = TrayIconBuilder::new()
            .with_id("enable-touchpad-tray")
            .with_menu(Box::new(menu))
            .with_tooltip("enable-touchpad")
            .with_menu_on_left_click(false)
            .with_icon(icon)
            .build()
    {
        std::mem::forget(tray);
    }
}

/// Poll muda's global menu-event queue and dispatch selections.
pub fn spawn_forwarder() {
    std::thread::spawn(|| {
        loop {
            if let Ok(event) = MenuEvent::receiver().try_recv()
                && let Some(action) = decode(&event.id.0)
            {
                app::handle_tray(action);
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

/// A 32x32 blue disc with a darker rim, generated in-process so the app
/// ships no binary assets.
fn icon_rgba() -> Vec<u8> {
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
