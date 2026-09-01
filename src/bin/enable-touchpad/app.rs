//! Dioxus UI: a small Gruvbox-styled settings window (hidden by default,
//! opened from the tray) for the mouse-layer key bindings. Follows the
//! system light/dark theme via `prefers-color-scheme`.
//!
//! Bindings are captured, not chosen from a list: click a row's button, press
//! any supported key, and the physical key becomes that action inside the
//! `mouse` layer. The touchpad itself is toggled by the operating system: the
//! embedded kanata taps Ctrl+Win+F24 on layer-key press and on release, the
//! watchdog in [`crate::watchdog`] corrects any state drift. This app never
//! touches devices.

use crate::config_store;
use crate::watchdog::WatchdogState;
use dioxus::desktop::tao;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder, WindowCloseBehaviour, window};
use dioxus::prelude::*;
use etp_core::{AppConfig, CANCEL_KEY, HOLD_KEY, KEY_NONE, is_bindable, key_label};
use etp_platform::Platform;
use std::sync::{Arc, OnceLock};

/// Main window handle; tao windows are `Send`, so the tray thread can use it.
static MAIN_WINDOW: OnceLock<Arc<tao::window::Window>> = OnceLock::new();

/// Process-wide platform adapter, installed by [`launch`] before the Dioxus
/// root runs (Dioxus `launch` accepts a `fn() -> Element`, so the UI root
/// cannot capture arguments).
static PLATFORM: OnceLock<PlatformRef> = OnceLock::new();

/// Process-wide watchdog state, installed by [`launch`] for the same reason.
static WATCHDOG: OnceLock<Arc<WatchdogState>> = OnceLock::new();

/// Newtype so `OnceLock` can hold an unsized trait-object reference.
#[derive(Clone, Copy)]
struct PlatformRef(&'static dyn Platform);

/// Slot index of the 鼠标左键 row.
const SLOT_LEFT: i8 = 0;
/// Slot index of the 鼠标中键 row.
const SLOT_MIDDLE: i8 = 1;
/// Slot index of the 鼠标右键 row.
const SLOT_RIGHT: i8 = 2;
/// Slot index of the `CapsLock` action row.
const SLOT_CAPS: i8 = 3;

/// Gruvbox palette for both themes; the webview follows the system setting.
const GRUVBOX_CSS: &str = r#"
:root{--bg0:#282828;--bg1:#3c3836;--bg2:#504945;--fg:#ebdbb2;--dim:#a89984;
--line:#504945;--accent:#83a598;--accent2:#8ec07c;--red:#fb4934;}
@media (prefers-color-scheme: light){:root{--bg0:#fbf1c7;--bg1:#ebdbb2;--bg2:#d5c4a1;--fg:#3c3836;--dim:#7c6f64;
--line:#d5c4a1;--accent:#076678;--accent2:#427b58;--red:#9d0006;}}
html,body{margin:0;overflow:hidden;background:var(--bg0);}
*{box-sizing:border-box;user-select:none;cursor:default;
font-family:'Segoe UI','Microsoft YaHei',system-ui,sans-serif;}
.key-btn{background:var(--bg1);color:var(--fg);border:1px solid var(--line);
border-radius:6px;padding:5px 12px;min-width:132px;text-align:left;
font-size:13px;cursor:pointer;}
.key-btn:hover{border-color:var(--accent);}
.key-btn.capturing{border-color:var(--accent);background:var(--bg2);color:var(--accent);}
.btn-clear{background:transparent;color:var(--dim);border:1px solid transparent;
border-radius:5px;width:24px;height:26px;cursor:pointer;font-size:13px;}
.btn-clear:hover{color:var(--red);border-color:var(--line);}
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

/// Configure and launch the desktop app. The window starts hidden and is
/// opened from the tray menu.
pub fn launch(platform: &'static dyn Platform, state: Arc<WatchdogState>) {
    let _ = PLATFORM.set(PlatformRef(platform));
    let _ = WATCHDOG.set(state);
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("enable-touchpad")
                .with_visible(false)
                .with_decorations(false)
                .with_inner_size(LogicalSize::new(440.0, 400.0)),
        )
        .with_close_behaviour(WindowCloseBehaviour::WindowHides);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui_root);
}

/// A tray menu entry was selected (runs on the tray forwarder thread).
pub fn handle_tray(
    action: crate::tray::TrayAction,
    platform: &'static dyn Platform,
    state: &Arc<WatchdogState>,
) {
    match action {
        crate::tray::TrayAction::OpenSettings => {
            if let Some(win) = MAIN_WINDOW.get() {
                win.set_visible(true);
                win.set_focus();
            }
        }
        crate::tray::TrayAction::Quit => {
            // Best effort: leaving while the layer key is held would strand
            // the touchpad in the enabled state — tap once more.
            if state.expected_on() {
                let _ = platform.tap_toggle_chord();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            std::process::exit(0);
        }
    }
}

/// Returns the process-wide platform adapter installed by [`launch`].
fn platform() -> &'static dyn Platform {
    match PLATFORM.get() {
        Some(platform_ref) => platform_ref.0,
        None => panic!("platform adapter not initialised"),
    }
}

/// Returns a clone of the process-wide watchdog state installed by [`launch`].
fn watchdog() -> Arc<WatchdogState> {
    match WATCHDOG.get() {
        Some(state) => Arc::clone(state),
        None => panic!("watchdog state not initialised"),
    }
}

fn ui_root() -> Element {
    let AppConfig {
        feature_enabled: initial_feature,
        left_click_key: initial_left,
        middle_click_key: initial_middle,
        right_click_key: initial_right,
        capslock_key: initial_caps,
    } = config_store::load(platform());

    let feature = use_signal(move || initial_feature);
    let left_key = use_signal(move || initial_left);
    let middle_key = use_signal(move || initial_middle);
    let right_key = use_signal(move || initial_right);
    let caps_key = use_signal(move || initial_caps);
    let save_state = use_signal(String::new);
    let capture_hint = use_signal(String::new);
    let capturing = use_signal(|| -1_i8);

    // Register the main window handle so the tray thread can open it.
    use_future(|| async move {
        let _ = MAIN_WINDOW.set(Arc::clone(&window().window));
        log::info!("main window created");
    });

    rsx! {
        style { "{GRUVBOX_CSS}" }
        settings_form {
            feature,
            left_key,
            middle_key,
            right_key,
            caps_key,
            save_state,
            capture_hint,
            capturing,
        }
    }
}

/// The settings page body: capture rows, master switch, save button, footer.
#[component]
fn settings_form(
    mut feature: Signal<bool>,
    mut left_key: Signal<String>,
    mut middle_key: Signal<String>,
    mut right_key: Signal<String>,
    mut caps_key: Signal<String>,
    mut save_state: Signal<String>,
    mut capture_hint: Signal<String>,
    mut capturing: Signal<i8>,
) -> Element {
    let platform = platform();
    let state = watchdog();

    rsx! {
        div {
            style: "background:var(--bg0);color:var(--fg);height:100vh;display:flex;flex-direction:column;",
            onmousedown: move |_| {
                // Clicking anywhere outside a capture button stops the capture.
                if capturing.cloned() >= 0 {
                    capturing.set(-1);
                    capture_hint.set(String::new());
                }
            },
            onkeydown: move |ev| {
                if capturing.cloned() < 0 {
                    return;
                }
                ev.prevent_default();
                ev.stop_propagation();
                let code = ev.code().to_string();
                if code == CANCEL_KEY {
                    capturing.set(-1);
                    capture_hint.set(String::new());
                    return;
                }
                if !is_bindable(&code) {
                    capture_hint.set(format!("按键 {code} 不受支持,换一个吧(Esc 取消)"));
                    return;
                }
                match capturing.cloned() {
                    SLOT_LEFT => left_key.set(code),
                    SLOT_MIDDLE => middle_key.set(code),
                    SLOT_RIGHT => right_key.set(code),
                    _ => caps_key.set(code),
                }
                capturing.set(-1);
                capture_hint.set(String::new());
            },
            title_bar {}
            div {
                style: "flex:1;padding:4px 16px 10px 16px;display:flex;flex-direction:column;",
                capture_instructions {}
                binding_row { name: "鼠标左键", value: left_key, capturing, slot: SLOT_LEFT }
                binding_row { name: "鼠标中键", value: middle_key, capturing, slot: SLOT_MIDDLE }
                binding_row { name: "鼠标右键", value: right_key, capturing, slot: SLOT_RIGHT }
                binding_row { name: "CapsLock", value: caps_key, capturing, slot: SLOT_CAPS }
                capture_status { hint: capture_hint }
                div {
                    style: "display:flex;align-items:center;gap:8px;margin:0 0 12px 0;",
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
                            match apply(platform, &state, &cfg) {
                                Ok(()) => save_state.set("已保存并热应用 ✓".to_string()),
                                Err(e) => save_state.set(format!("失败: {e}")),
                            }
                        },
                        "保存并应用"
                    }
                    span { style: "color:var(--dim);font-size:12px;", "{save_state}" }
                }
                div {
                    style: "margin-top:auto;",
                    footer_notes {}
                }
            }
        }
    }
}

/// How key capture works, rendered above the rows.
#[component]
fn capture_instructions() -> Element {
    rsx! {
        div {
            style: "color:var(--dim);font-size:12px;margin:6px 0 10px 0;line-height:1.5;",
            "点击按钮后按下任意键即可绑定 · Esc 取消 · × 恢复为无 · {HOLD_KEY} 是固定的层触发键"
        }
    }
}

/// Red status line under the rows (unsupported key notices live here).
#[component]
fn capture_status(hint: Signal<String>) -> Element {
    rsx! {
        div {
            style: "height:16px;color:var(--red);font-size:11px;margin:0 0 4px 0;",
            "{hint}"
        }
    }
}

/// Static footer: how the toggle works, watchdog note, log location.
#[component]
fn footer_notes() -> Element {
    rsx! {
        div {
            style: "color:var(--dim);font-size:11px;line-height:1.7;",
            "CapsLock 按下/松开各发出一次 Ctrl+Win+F24(软开关由系统触摸板驱动执行)"
            br {}
            "状态矫正:未按住 CapsLock 时自动检测触摸板状态并软关闭(需 Win11 精确式触摸板)"
            br {}
            "日志:%APPDATA%\\enable-touchpad\\enable-touchpad.log"
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

/// One action row: a capture button (click, then press the desired key) and a
/// clear button that resets the action to "none".
#[component]
fn binding_row(
    name: String,
    mut value: Signal<String>,
    mut capturing: Signal<i8>,
    slot: i8,
) -> Element {
    let active = capturing.cloned() == slot;
    let label = key_label(&value.cloned());
    let btn_class = if active {
        "key-btn capturing"
    } else {
        "key-btn"
    };
    let row = "display:flex;align-items:center;margin-bottom:8px;gap:8px;";
    let action_label = "width:80px;color:var(--fg);font-size:13px;";

    rsx! {
        div {
            style: "{row}",
            span { style: "{action_label}", "{name}" }
            button {
                class: "{btn_class}",
                onclick: move |_| capturing.set(slot),
                if active { "按下任意键…" } else { "{label}" }
            }
            button {
                class: "btn-clear",
                title: "恢复为无",
                onclick: move |_| {
                    value.set(KEY_NONE.to_string());
                    capturing.set(-1);
                },
                "×"
            }
        }
    }
}

fn apply(
    platform: &'static dyn Platform,
    state: &Arc<WatchdogState>,
    cfg: &AppConfig,
) -> Result<(), String> {
    config_store::save(platform, cfg)?;
    platform
        .apply_engine_config(cfg)
        .map_err(|e| e.to_string())?;
    state.set_managed(cfg.feature_enabled);
    log::info!("settings saved and applied: {cfg:?}");
    Ok(())
}
