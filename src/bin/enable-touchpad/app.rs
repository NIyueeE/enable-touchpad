//! Dioxus UI: a small Gruvbox-styled settings window (hidden by default,
//! opened from the tray) for the mouse-layer key bindings, plus file logging
//! setup. Follows the system light/dark theme via `prefers-color-scheme`.
//!
//! The touchpad itself is toggled by the operating system: the embedded
//! kanata taps Ctrl+Win+F24 on `CapsLock` press and on release, and whatever
//! the system binds that combo to performs the soft enable/disable. This app
//! never touches devices.

use crate::config::{self, AppConfig, KEY_CHOICES};
use crate::kanata_embed;
use dioxus::desktop::tao;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour, window};
use dioxus::prelude::*;
use std::sync::{Arc, OnceLock};

/// Main window handle; tao windows are `Send`, so the tray thread can use it.
static MAIN_WINDOW: OnceLock<Arc<tao::window::Window>> = OnceLock::new();

/// Gruvbox palette for both themes; the webview follows the system setting.
const GRUVBOX_CSS: &str = r#"
:root{--bg0:#282828;--bg1:#3c3836;--bg2:#504945;--fg:#ebdbb2;--dim:#a89984;
--line:#504945;--accent:#83a598;--accent2:#8ec07c;--red:#fb4934;
--chev:url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6'><path d='M1 1l4 4 4-4' stroke='%23a89984' stroke-width='1.5' fill='none'/></svg>");}
@media (prefers-color-scheme: light){:root{--bg0:#fbf1c7;--bg1:#ebdbb2;--bg2:#d5c4a1;--fg:#3c3836;--dim:#7c6f64;
--line:#d5c4a1;--accent:#076678;--accent2:#427b58;--red:#9d0006;
--chev:url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='10' height='6'><path d='M1 1l4 4 4-4' stroke='%237c6f64' stroke-width='1.5' fill='none'/></svg>");}}
html,body{margin:0;overflow:hidden;background:var(--bg0);}
*{box-sizing:border-box;user-select:none;cursor:default;
font-family:'Segoe UI','Microsoft YaHei',system-ui,sans-serif;}
select{appearance:none;-webkit-appearance:none;color:var(--fg);
background:var(--bg1) var(--chev) no-repeat right 8px center;
border:1px solid var(--line);border-radius:6px;padding:5px 26px 5px 10px;
font-size:13px;cursor:pointer;}
select:hover{border-color:var(--accent);}
input[type=checkbox]{appearance:none;-webkit-appearance:none;width:15px;height:15px;
border:1px solid var(--line);border-radius:4px;background:var(--bg1);
display:inline-grid;place-content:center;cursor:pointer;margin:0;}
input[type=checkbox]::before{content:"";width:8px;height:8px;
transform:scale(0);transition:transform .08s;background:var(--accent);
clip-path:polygon(14% 44%,0 65%,50% 100%,100% 16%,80% 0,43% 62%);}
input[type=checkbox]:checked{border-color:var(--accent);}
input[type=checkbox]:checked::before{transform:scale(1);}
.btn-primary{background:var(--accent);color:var(--bg0);border:none;border-radius:6px;
padding:7px 16px;cursor:pointer;font-size:13px;font-weight:600;}
.btn-primary:hover{background:var(--accent2);}
.btn-title{background:transparent;color:var(--dim);border:none;width:30px;height:26px;
cursor:pointer;font-size:13px;border-radius:5px;}
.btn-title:hover{background:var(--bg2);}
.btn-close{background:transparent;color:var(--dim);border:none;width:30px;height:26px;
cursor:pointer;font-size:13px;border-radius:5px;}
.btn-close:hover{background:var(--red);color:var(--bg0);}
"#;

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
                .with_decorations(false)
                .with_inner_size(LogicalSize::new(440.0, 330.0)),
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
        left_click_key: initial_left,
        middle_click_key: initial_middle,
        right_click_key: initial_right,
        capslock_key: initial_caps,
    } = AppConfig::load();

    let mut feature = use_signal(move || initial_feature);
    let left_key = use_signal(move || initial_left);
    let middle_key = use_signal(move || initial_middle);
    let right_key = use_signal(move || initial_right);
    let caps_key = use_signal(move || initial_caps);
    let mut save_state = use_signal(String::new);

    // Register the main window handle so the tray thread can open it.
    use_future(|| async move {
        let _ = MAIN_WINDOW.set(Arc::clone(&window().window));
        log::info!("main window created");
    });

    rsx! {
        style { "{GRUVBOX_CSS}" }
        div {
            style: "background:var(--bg0);color:var(--fg);height:100vh;display:flex;flex-direction:column;",
            title_bar {}
            div {
                style: "flex:1;padding:4px 16px 10px 16px;display:flex;flex-direction:column;",
                div {
                    style: "color:var(--dim);font-size:12px;margin:6px 0 10px 0;",
                    "为每个鼠标动作选择 mouse 层里的键盘键(CapsLock 按住时生效,松开时还原)"
                }
                binding_row { name: "鼠标左键", value: left_key, actions: &KEY_CHOICES }
                binding_row { name: "鼠标中键", value: middle_key, actions: &KEY_CHOICES }
                binding_row { name: "鼠标右键", value: right_key, actions: &KEY_CHOICES }
                binding_row { name: "CapsLock", value: caps_key, actions: &KEY_CHOICES }
                div {
                    style: "display:flex;align-items:center;gap:8px;margin:8px 0 12px 0;",
                    input {
                        r#type: "checkbox",
                        checked: feature.cloned(),
                        onclick: move |_| feature.set(!feature.cloned()),
                    }
                    span { style: "font-size:13px;", "总开关(CapsLock 层功能)" }
                }
                div {
                    style: "display:flex;align-items:center;gap:10px;",
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let cfg = AppConfig {
                                feature_enabled: feature.cloned(),
                                left_click_key: left_key.cloned(),
                                middle_click_key: middle_key.cloned(),
                                right_click_key: right_key.cloned(),
                                capslock_key: caps_key.cloned(),
                            };
                            match apply(&cfg) {
                                Ok(()) => save_state.set("已保存并热应用 ✓".to_string()),
                                Err(e) => save_state.set(format!("失败: {e}")),
                            }
                        },
                        "保存并应用"
                    }
                    span { style: "color:var(--dim);font-size:12px;", "{save_state}" }
                }
                div {
                    style: "margin-top:auto;color:var(--dim);font-size:11px;line-height:1.7;",
                    "CapsLock 按下/松开各发出一次 Ctrl+Win+F24(软开关由系统触摸板驱动执行)"
                    br {}
                    "日志:%APPDATA%\\enable-touchpad\\enable-touchpad.log"
                }
            }
        }
    }
}

#[component]
fn title_bar() -> Element {
    rsx! {
        div {
            style: "height:34px;display:flex;align-items:center;padding:0 6px 0 12px;gap:8px;
                    background:var(--bg1);border-bottom:1px solid var(--line);
                    font-size:12px;color:var(--dim);",
            onmousedown: move |_| {
                let _ = window().drag_window();
            },
            span {
                style: "width:9px;height:9px;border-radius:50%;background:var(--accent);",
            }
            span { "enable-touchpad" }
            span { style: "flex:1;" }
            button {
                class: "btn-title",
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| {
                    window().set_minimized(true);
                },
                "—"
            }
            button {
                class: "btn-close",
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| {
                    if let Some(win) = MAIN_WINDOW.get() {
                        win.set_visible(false);
                    }
                },
                "✕"
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
    let row = "display:flex;align-items:center;margin-bottom:8px;gap:12px;";
    let action_label = "width:80px;color:var(--fg);font-size:13px;";
    let items = actions
        .iter()
        .map(|(id, text)| (*id, *text))
        .collect::<Vec<_>>();

    rsx! {
        div {
            style: "{row}",
            span { style: "{action_label}", "{name}" }
            select {
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
