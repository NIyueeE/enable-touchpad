//! Dioxus UI: a small settings window (hidden by default, opened from the
//! tray) for the mouse-layer key bindings, plus file logging setup.
//!
//! The touchpad itself is toggled by the operating system: the embedded
//! kanata taps Ctrl+Win+F24 on `CapsLock` press/release, and whatever the
//! system binds that combo to performs the soft enable/disable. This app
//! never touches devices.

use crate::config::{self, AppConfig, LALT_ACTIONS, MOUSE_ACTIONS};
use crate::kanata_embed;
use dioxus::desktop::tao;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour, window};
use dioxus::prelude::*;
use std::sync::{Arc, OnceLock};

/// Main window handle; tao windows are `Send`, so the tray thread can use it.
static MAIN_WINDOW: OnceLock<Arc<tao::window::Window>> = OnceLock::new();

/// Configure logging to `%APPDATA%\enable-touchpad\enable-touchpad.log`.
/// Kanata logs through the `log` crate too, so its output lands in the same
/// file.
pub fn init_logging() {
    let Ok(dir) = config::app_dir() else {
        return;
    };
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("enable-touchpad.log"))
    else {
        return;
    };
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        file,
    );
    log::info!("app starting");
}

/// Configure and launch the desktop app. The window starts hidden and is
/// opened from the tray menu.
pub fn launch() {
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("enable-touchpad")
                .with_visible(false)
                .with_inner_size(LogicalSize::new(460.0, 340.0)),
        )
        .with_close_behaviour(WindowCloseBehaviour::WindowHides);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui_root);
}

/// A tray menu entry was selected (runs on the tray forwarder thread).
pub fn handle_tray(action: crate::tray::TrayAction) {
    match action {
        crate::tray::TrayAction::OpenSettings => {
            if let Some(win) = MAIN_WINDOW.get() {
                win.set_visible(true);
                win.set_focus();
            }
        }
        crate::tray::TrayAction::Quit => std::process::exit(0),
    }
}

fn ui_root() -> Element {
    let AppConfig {
        feature_enabled: initial_feature,
        key_q: initial_q,
        key_w: initial_w,
        key_e: initial_e,
        key_lalt: initial_lalt,
    } = AppConfig::load();

    let mut feature = use_signal(move || initial_feature);
    let key_q = use_signal(move || initial_q);
    let key_w = use_signal(move || initial_w);
    let key_e = use_signal(move || initial_e);
    let key_lalt = use_signal(move || initial_lalt);
    let mut save_state = use_signal(String::new);

    // Register the main window handle so the tray thread can open it.
    use_future(|| async move {
        let _ = MAIN_WINDOW.set(Arc::clone(&window().window));
        log::info!("main window created");
    });

    let section = "background:#1c1f26;border-radius:12px;padding:14px 16px;";
    let label = "color:#9aa3b2;font-size:12px;margin-bottom:8px;";
    let button = "background:#2f6fd6;color:#fff;border:none;border-radius:8px;padding:7px 16px;cursor:pointer;font-size:13px;";
    let row = "display:flex;align-items:center;margin-bottom:8px;gap:10px;";

    rsx! {
        div {
            style: "background:#14161a;color:#e6e9ef;font-family:'Segoe UI','Microsoft YaHei',system-ui,sans-serif;padding:14px 16px;box-sizing:border-box;",

            div {
                style: "{section}",
                div { style: "{label}", "Mouse 层按键(CapsLock 按住时生效;松开时软关闭触摸板)" }
                binding_row { name: "Q", value: key_q, actions: &MOUSE_ACTIONS }
                binding_row { name: "W", value: key_w, actions: &MOUSE_ACTIONS }
                binding_row { name: "E", value: key_e, actions: &MOUSE_ACTIONS }
                binding_row { name: "Left Alt", value: key_lalt, actions: &LALT_ACTIONS }
                div {
                    style: "{row}",
                    input {
                        r#type: "checkbox",
                        checked: feature.cloned(),
                        onclick: move |_| feature.set(!feature.cloned()),
                    }
                    span { "总开关(CapsLock 层功能)" }
                }
                div {
                    style: "{row}",
                    button {
                        style: "{button}",
                        onclick: move |_| {
                            let cfg = AppConfig {
                                feature_enabled: feature.cloned(),
                                key_q: key_q.cloned(),
                                key_w: key_w.cloned(),
                                key_e: key_e.cloned(),
                                key_lalt: key_lalt.cloned(),
                            };
                            match apply(&cfg) {
                                Ok(()) => save_state.set("已保存并热应用 ✓".to_string()),
                                Err(e) => save_state.set(format!("失败: {e}")),
                            }
                        },
                        "保存并应用"
                    }
                    span { style: "color:#9aa3b2;font-size:12px;", "{save_state}" }
                }
            }

            div {
                style: "color:#5d6572;font-size:12px;line-height:1.8;margin-top:10px;",
                "CapsLock 按下/松开会各发出一次 Ctrl+Win+F24,由系统(触摸板驱动)完成触摸板软开关。"
                br {}
                "详细日志:%APPDATA%\\enable-touchpad\\enable-touchpad.log"
            }
        }
    }
}

#[component]
fn binding_row(
    name: String,
    mut value: Signal<String>,
    actions: &'static [(&'static str, &'static str)],
) -> Element {
    let row = "display:flex;align-items:center;margin-bottom:8px;gap:10px;";
    let key_label = "width:64px;color:#e6e9ef;font-size:13px;";
    let items = actions
        .iter()
        .map(|(id, text)| (*id, *text))
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "{row}",
            span { style: "{key_label}", "{name}" }
            select {
                style: "background:#14161a;color:#e6e9ef;border:1px solid #343a46;border-radius:6px;padding:4px 8px;width:110px;",
                onchange: move |e| value.set(e.value()),
                for (id, text) in items {
                    option { value: "{id}", selected: value.cloned() == id, "{text}" }
                }
            }
        }
    }
}

fn apply(cfg: &AppConfig) -> Result<(), String> {
    cfg.save()?;
    kanata_embed::apply_config(cfg)?;
    log::info!("settings saved and applied: {cfg:?}");
    Ok(())
}
