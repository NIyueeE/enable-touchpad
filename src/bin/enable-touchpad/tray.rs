//! System tray.
//!
//! **Left click** toggles the master switch; the right-click menu holds
//! 打开设置, a checkable 总开关 item, and 退出. The icon reflects the
//! master switch (coloured when on, greyscale when off).
//!
//! tray-icon is `Rc<RefCell>` internally, so **every mutation runs on the
//! main thread** through `thread_local` state; foreign threads request
//! changes via the etp-ffi main-thread door ([`etp_ffi::window`]).

use crate::app;
use crate::watchdog::WatchdogState;
use etp_ffi::window as door;
use etp_platform::Platform;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tray_icon::menu::{CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Actions the tray can request from the app.
#[derive(Debug, Clone, Copy)]
pub enum TrayAction {
    /// Show and focus the settings window.
    OpenSettings,
    /// Flip the master switch (left click or the menu check item).
    ToggleMaster,
    /// Exit the app.
    Quit,
}

thread_local! {
    /// The live tray, kept for the process lifetime. Main thread only.
    static TRAY: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
    /// The checkable 总开关 menu item. Main thread only.
    static MASTER_ITEM: RefCell<Option<Rc<CheckMenuItem>>> = const { RefCell::new(None) };
    /// Coloured (master on) tray icon. Main thread only.
    static ICON_ON: RefCell<Option<tray_icon::Icon>> = const { RefCell::new(None) };
    /// Greyscale (master off) tray icon. Main thread only.
    static ICON_OFF: RefCell<Option<tray_icon::Icon>> = const { RefCell::new(None) };
}

/// Build the tray with the menu and both icon variants. Call once from
/// `main` (the main thread). Failures are logged loudly — the tray is the
/// settings window's only door.
pub fn install(master: bool) {
    let settings = MenuItem::with_id("et-settings", "打开设置", true, None);
    let master_item = CheckMenuItem::with_id("et-master", "总开关", true, master, None);
    let quit = MenuItem::with_id("et-quit", "退出", true, None);
    let items: [&dyn IsMenuItem; 4] = [
        &settings,
        &master_item,
        &PredefinedMenuItem::separator(),
        &quit,
    ];
    let menu = Menu::new();
    for item in items {
        let _ = menu.append(item);
    }

    let on_icon = badge_icon(false);
    let off_icon = badge_icon(true);
    let icon = if master {
        on_icon.as_ref()
    } else {
        off_icon.as_ref()
    };

    let mut builder = TrayIconBuilder::new()
        .with_id("enable-touchpad-tray")
        .with_menu(Box::new(menu))
        .with_tooltip("enable-touchpad")
        .with_menu_on_left_click(false);
    if let Some(icon) = icon {
        builder = builder.with_icon(icon.clone());
    }
    match builder.build() {
        // Main-thread-only handle, stored for the process lifetime.
        Ok(tray) => {
            MASTER_ITEM.with(|slot| *slot.borrow_mut() = Some(Rc::new(master_item)));
            ICON_ON.with(|slot| *slot.borrow_mut() = on_icon);
            ICON_OFF.with(|slot| *slot.borrow_mut() = off_icon);
            TRAY.with(|slot| *slot.borrow_mut() = Some(tray));
        }
        Err(e) => log::error!("tray installation failed: {e}"),
    }
}

/// Apply the master switch to the tray visuals (icon + check item).
/// **Main thread only** — foreign threads use [`request_master_visuals`].
pub fn apply_master_visuals(on: bool) {
    let icon = if on {
        ICON_ON.with(|slot| slot.borrow().clone())
    } else {
        ICON_OFF.with(|slot| slot.borrow().clone())
    };
    TRAY.with(|slot| {
        if let Some(tray) = slot.borrow().as_ref()
            && let Err(e) = tray.set_icon(icon)
        {
            log::error!("tray set_icon failed: {e}");
        }
    });
    MASTER_ITEM.with(|slot| {
        if let Some(item) = slot.borrow().as_ref() {
            item.set_checked(on);
        }
    });
}

/// Ask the main thread to apply the master switch to the tray visuals.
/// Thread-safe; callable from any thread.
pub fn request_master_visuals(on: bool) {
    door::post(door::TASK_APPLY_MASTER_VISUALS, usize::from(on));
}

/// Poll muda's menu queue and tray-icon's click queue and dispatch actions.
pub fn spawn_forwarder(platform: &'static dyn Platform, state: Arc<WatchdogState>) {
    std::thread::spawn(move || {
        loop {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                let action = match event.id.0.as_str() {
                    "et-settings" => Some(app::TrayAction::OpenSettings),
                    "et-master" => Some(app::TrayAction::ToggleMaster),
                    "et-quit" => Some(app::TrayAction::Quit),
                    _ => None,
                };
                if let Some(action) = action {
                    app::handle_tray(action, platform, &state);
                }
            }
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let Some(action) = decode_click(&event) {
                    app::handle_tray(action, platform, &state);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
    });
}

/// A plain left-click release toggles the master switch (the right button
/// owns the context menu).
fn decode_click(event: &TrayIconEvent) -> Option<app::TrayAction> {
    match event {
        TrayIconEvent::Click {
            button: tray_icon::MouseButton::Left,
            button_state: tray_icon::MouseButtonState::Up,
            ..
        } => Some(app::TrayAction::ToggleMaster),
        _ => None,
    }
}

/// Decode the designed 32px icon into a tray `Icon`; `gray` desaturates it
/// for the master-off state.
fn badge_icon(gray: bool) -> Option<tray_icon::Icon> {
    const PNG: &[u8] = include_bytes!("../../../assets/icon_32.png");
    let mut reader = png::Decoder::new(std::io::Cursor::new(PNG))
        .read_info()
        .ok()?;
    let mut buf = vec![0_u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    if gray {
        for px in buf.as_chunks_mut::<4>().0 {
            // Max luminance is 255*10/10 = 255, so the u8 cast is lossless;
            // express it through try_from's unwrap_or to satisfy the lint.
            let lum =
                u8::try_from((u16::from(px[0]) * 3 + u16::from(px[1]) * 6 + u16::from(px[2])) / 10)
                    .unwrap_or(u8::MAX);
            px[0] = lum;
            px[1] = lum;
            px[2] = lum;
        }
    }
    tray_icon::Icon::from_rgba(buf, info.width, info.height).ok()
}
