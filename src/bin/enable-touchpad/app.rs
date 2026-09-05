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
use dioxus::prelude::SyncStorage;
use dioxus::prelude::*;
use etp_core::{AppConfig, CANCEL_KEY, HOLD_KEY, KEY_NONE, is_bindable, key_label};
use etp_platform::Platform;
use std::sync::{Arc, OnceLock};

/// The tray action enum, re-exported so the forwarder can name it
/// `app::TrayAction`.
pub use crate::tray::TrayAction;

/// The master-switch checkbox signal (`SyncStorage`), shared with foreign
/// threads (tray toggles) via [`sync_feature_signal`].
static FEATURE_SIGNAL: OnceLock<Signal<bool, SyncStorage>> = OnceLock::new();

/// Push a master-switch change into the settings UI checkbox from any
/// thread.
pub fn sync_feature_signal(on: bool) {
    if let Some(feature) = FEATURE_SIGNAL.get() {
        *feature.write_unchecked() = on;
    }
}

/// Handler running on the main thread for door tasks (tray visuals, window
/// show). Registered in `main` via `etp_ffi::window::init`.
pub fn on_door_message(task: usize, param: usize) {
    match task {
        etp_ffi::window::TASK_APPLY_MASTER_VISUALS => {
            crate::tray::apply_master_visuals(param != 0);
        }
        etp_ffi::window::TASK_OPEN_SETTINGS => {
            if let Some(win) = MAIN_WINDOW.get() {
                // Main thread: tao's executor runs inline here, which keeps
                // its internal visibility state in sync — a raw ShowWindow
                // desyncs it and turns the ✕ hide into a no-op.
                win.set_minimized(false);
                win.set_visible(true);
                win.set_focus();
            }
        }
        _ => log::warn!("unknown door task {task}"),
    }
}

/// Main window handle; tao windows are `Send`, so the tray thread can use it.
static MAIN_WINDOW: OnceLock<Arc<tao::window::Window>> = OnceLock::new();

/// Process-wide platform adapter, installed by [`launch`] before the Dioxus
/// root runs (Dioxus `launch` accepts a `fn() -> Element`, so the UI root
/// cannot capture arguments).
static PLATFORM: OnceLock<PlatformRef> = OnceLock::new();

/// Process-wide watchdog state, installed by [`launch`] for the same reason.
static WATCHDOG: OnceLock<Arc<WatchdogState>> = OnceLock::new();

/// `data:` URI of the designed icon for the title-bar `<img>`, built once
/// from the same embedded 32px asset the window icon uses.
static TITLE_ICON_SRC: OnceLock<String> = OnceLock::new();

/// Returns the title-bar icon `data:` URI, encoding the embedded asset on
/// first use.
fn title_icon_src() -> &'static str {
    TITLE_ICON_SRC.get_or_init(|| {
        use base64::Engine as _;
        const PNG: &[u8] = include_bytes!("../../../assets/icon_32.png");
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(PNG)
        )
    })
}

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

/// Outcome of the last 保存并应用 click; rendered as a coloured pill.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SaveStatus {
    /// Nothing saved yet (or the window was reopened).
    Idle,
    /// Config persisted and hot-applied to the running engine.
    Ok,
    /// Something failed; carries the error text.
    Err(String),
}

/// Gruvbox palette for both themes; the webview follows the system setting.
/// Three elevation steps per theme (window `--bg0` → card `--card` → keycap
/// `--btn`) use the hard/soft background variants so surfaces visibly stack,
/// backed by `--card-shadow` (drop shadow + top bevel) and `--key-grad`
/// (keycap face gradient).
const GRUVBOX_CSS: &str = r#"
:root{--bg0:#1d2021;--card:#32302f;--line:#454039;--btn:#504945;--btn-h:#5e554d;
--fg:#ebdbb2;--dim:#a89984;--accent:#83a598;--green:#8ec07c;--yellow:#fabd2f;
--red:#fb4934;--accent-soft:rgba(131,165,152,.14);--shadow:rgba(0,0,0,.5);
--card-shadow:0 2px 6px rgba(0,0,0,.35),inset 0 1px 0 rgba(255,255,255,.03);
--key-grad:linear-gradient(180deg,#57504b,#4a423e);}
@media (prefers-color-scheme: light){:root{--bg0:#f2e5bc;--card:#fbf1c7;--line:#e3d7b2;
--btn:#fff8e2;--btn-h:#fffcf0;--fg:#3c3836;--dim:#7c6f64;--accent:#076678;--green:#427b58;
--yellow:#b57614;--red:#9d0006;--accent-soft:rgba(7,102,120,.10);--shadow:#d5c4a1;
--card-shadow:0 2px 6px rgba(133,111,76,.22),inset 0 1px 0 rgba(255,255,255,.6);
--key-grad:linear-gradient(180deg,#fffbef,#fdf3d9);}}
html,body{margin:0;height:100%;overflow:hidden;background:var(--bg0);}
*{box-sizing:border-box;user-select:none;cursor:default;
font-family:'Segoe UI','Microsoft YaHei',system-ui,sans-serif;}
.root{height:100vh;display:flex;flex-direction:column;background:var(--bg0);
color:var(--fg);outline:none;}
.title{height:38px;flex:none;display:flex;align-items:center;gap:9px;
padding:0 8px 0 14px;background:var(--card);border-bottom:1px solid var(--line);
font-size:12.5px;color:var(--dim);letter-spacing:.2px;}
.app-icon{width:18px;height:18px;flex:none;border-radius:4px;-webkit-user-drag:none;}
.btn-title{background:transparent;color:var(--dim);border:none;width:32px;height:28px;
cursor:pointer;font-size:12px;border-radius:6px;
transition:background .12s,color .12s;}
.btn-title:hover{background:var(--btn);}
.btn-title.btn-close:hover{background:var(--red);color:var(--bg0);}
.body{flex:1;min-height:0;padding:12px 16px 12px;display:flex;flex-direction:column;
gap:9px;}
.instructions{color:var(--dim);font-size:12px;line-height:1.5;}
.card{background:var(--card);border:1px solid var(--line);border-radius:10px;
box-shadow:var(--card-shadow);
padding:9px 12px;display:flex;flex-direction:column;gap:7px;}
.row{display:flex;align-items:center;gap:8px;}
.row-label{width:82px;flex:none;color:var(--fg);font-size:13px;}
.key-btn{background:var(--key-grad);color:var(--fg);border:1px solid var(--line);
border-radius:8px;padding:6px 12px;min-width:128px;text-align:left;font-size:13px;
cursor:pointer;box-shadow:0 2px 0 var(--shadow);
transition:border-color .12s,background .12s,color .12s,box-shadow .12s;}
.key-btn:hover{border-color:var(--accent);background:var(--btn-h);}
.key-btn:active{box-shadow:0 1px 0 var(--shadow);transform:translateY(1px);}
.key-btn.empty{color:var(--dim);}
.key-btn .key-name{font-weight:600;letter-spacing:.4px;}
.key-btn.capturing{border-color:var(--accent);background:var(--accent-soft);
color:var(--accent);animation:breathe 1.2s ease-in-out infinite;}
@keyframes breathe{50%{box-shadow:0 0 0 4px var(--accent-soft);}}
.btn-clear{background:transparent;color:var(--dim);border:1px solid transparent;
border-radius:6px;width:26px;height:29px;flex:none;cursor:pointer;font-size:14px;
transition:color .12s,border-color .12s,background .12s;}
.btn-clear:hover{color:var(--red);border-color:var(--line);background:var(--btn);}
.status{min-height:18px;flex:none;font-size:11.5px;line-height:1.4;}
.hint-err{color:var(--red);}
.hint-warn{color:var(--yellow);}
.switch{appearance:none;-webkit-appearance:none;position:relative;width:36px;height:20px;
flex:none;margin:0;background:var(--btn);border:1px solid var(--line);
border-radius:999px;cursor:pointer;display:block;
transition:background .15s,border-color .15s;}
.switch::before{content:"";position:absolute;top:2px;left:2px;width:14px;height:14px;
border-radius:50%;background:var(--dim);transition:transform .15s,background .15s;}
.switch:checked{background:var(--accent);border-color:var(--accent);}
.switch:checked::before{transform:translateX(16px);background:var(--bg0);}
.master{display:flex;align-items:center;gap:10px;}
.master-label{font-size:13px;}
.master-desc{margin-left:auto;color:var(--dim);font-size:11px;}
.save-row{display:flex;align-items:center;gap:10px;margin-top:auto;flex-wrap:wrap;}
.btn-primary{background:var(--accent);color:var(--bg0);border:none;border-radius:8px;
padding:8px 18px;cursor:pointer;font-size:13px;font-weight:600;letter-spacing:.3px;
box-shadow:0 2px 0 var(--shadow);
transition:filter .12s,transform .12s,box-shadow .12s;}
.btn-primary:hover{filter:brightness(1.12);}
.btn-primary:active{transform:translateY(1px);box-shadow:0 1px 0 var(--shadow);}
.pill{font-size:11.5px;padding:2px 10px;border-radius:999px;border:1px solid;}
.pill-ok{color:var(--green);border-color:var(--green);}
.pill-err{color:var(--red);border-color:var(--red);}
.divider{flex:none;height:1px;background:var(--line);}
.footer{flex:none;color:var(--dim);font-size:11px;line-height:1.65;}
"#;

/// Configure and launch the desktop app. The window starts hidden and is
/// opened from the tray menu.
pub fn launch(platform: &'static dyn Platform, state: Arc<WatchdogState>) {
    let _ = PLATFORM.set(PlatformRef(platform));
    let _ = WATCHDOG.set(state);
    let icon = window_icon();
    let config = Config::new()
        .with_disable_context_menu(true)
        .with_window(
            WindowBuilder::new()
                .with_title("enable-touchpad")
                .with_visible(false)
                .with_decorations(false)
                .with_resizable(false)
                .with_window_icon(icon)
                .with_inner_size(LogicalSize::new(440.0, 486.0)),
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
        TrayAction::OpenSettings => {
            // Main thread shows/restores the window (async door task — the
            // forwarder thread must never hop through tao's executor).
            etp_ffi::window::post(etp_ffi::window::TASK_OPEN_SETTINGS, 0);
        }
        TrayAction::ToggleMaster => {
            let new = !crate::MASTER_SWITCH.load(std::sync::atomic::Ordering::Relaxed);
            state.set_managed(new);
            sync_feature_signal(new);
            crate::tray::request_master_visuals(new);
            log::info!("master switch toggled from the tray: {new}");
        }
        TrayAction::Quit => {
            // Best effort: leaving while the layer key is held would strand
            // the touchpad enabled — tap once more, but only when the system
            // still reports it on (the chord is a toggle, never blind).
            if state.expected_on() && matches!(platform.touchpad_enabled(), Ok(true)) {
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

/// Decode the designed 32×32 icon (embedded at compile time from
/// `assets/icon_32.png`) into RGBA for the tao window/taskbar icon. `None`
/// lets the caller fall back instead of failing the launch on a bad asset.
fn window_icon() -> Option<tao::window::Icon> {
    let bytes = include_bytes!("../../../assets/icon_32.png");
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0_u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    tao::window::Icon::from_rgba(buf, info.width, info.height).ok()
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

    let feature = use_signal_sync(move || initial_feature);
    let _ = FEATURE_SIGNAL.set(feature);
    let left_key = use_signal(move || initial_left);
    let middle_key = use_signal(move || initial_middle);
    let right_key = use_signal(move || initial_right);
    let caps_key = use_signal(move || initial_caps);
    let save_state = use_signal(|| SaveStatus::Idle);
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
    mut feature: Signal<bool, SyncStorage>,
    mut left_key: Signal<String>,
    mut middle_key: Signal<String>,
    mut right_key: Signal<String>,
    mut caps_key: Signal<String>,
    mut save_state: Signal<SaveStatus>,
    mut capture_hint: Signal<String>,
    mut capturing: Signal<i8>,
) -> Element {
    let platform = platform();
    let state = watchdog();

    // Duplicate-binding detection: the generator resolves a shared key by
    // first claim (左键 → 中键 → 右键 → CapsLock), so a later row with the
    // same key would silently do nothing. Surface that instead of hiding it.
    let bindings = [
        ("鼠标左键", left_key.cloned()),
        ("鼠标中键", middle_key.cloned()),
        ("鼠标右键", right_key.cloned()),
        ("CapsLock", caps_key.cloned()),
    ];
    let conflict: Option<String> = bindings.iter().enumerate().find_map(|(i, (name, key))| {
        if key == KEY_NONE {
            return None;
        }
        bindings[..i]
            .iter()
            .find(|(_, k)| k == key)
            .map(|(prev, _)| {
                format!(
                    "{prev} 与 {name} 绑定了同一个键 {},只保留先声明的 {prev}",
                    key_label(key)
                )
            })
    });

    rsx! {
        div {
            class: "root",
            tabindex: "-1",
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
                class: "body",
                capture_instructions {}
                div {
                    class: "card",
                    binding_row { name: "鼠标左键", value: left_key, capturing, slot: SLOT_LEFT }
                    binding_row { name: "鼠标中键", value: middle_key, capturing, slot: SLOT_MIDDLE }
                    binding_row { name: "鼠标右键", value: right_key, capturing, slot: SLOT_RIGHT }
                    binding_row { name: "CapsLock", value: caps_key, capturing, slot: SLOT_CAPS }
                }
                capture_status { hint: capture_hint, conflict }
                div {
                    class: "card",
                    label {
                        class: "master",
                        input {
                            r#type: "checkbox",
                            class: "switch",
                            checked: feature.cloned(),
                            onclick: move |_| feature.set(!feature.cloned()),
                        }
                        span { class: "master-label", "总开关 · CapsLock 层功能" }
                        span { class: "master-desc", "关闭后 CapsLock 恢复系统默认" }
                    }
                }
                div {
                    class: "save-row",
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
                                Ok(()) => save_state.set(SaveStatus::Ok),
                                Err(e) => save_state.set(SaveStatus::Err(e)),
                            }
                        },
                        "保存并应用"
                    }
                    match save_state.cloned() {
                        SaveStatus::Idle => rsx! {},
                        SaveStatus::Ok => rsx! {
                            span { class: "pill pill-ok", "已保存并热应用 ✓" }
                        },
                        SaveStatus::Err(e) => rsx! {
                            span { class: "pill pill-err", "失败: {e}" }
                        },
                    }
                }
                div { class: "divider" }
                footer_notes {}
            }
        }
    }
}

/// How key capture works, rendered above the rows.
#[component]
fn capture_instructions() -> Element {
    rsx! {
        div {
            class: "instructions",
            "点击按键框后按下任意键即可绑定 · Esc 取消 · × 恢复为无 · {HOLD_KEY} 是固定的层触发键"
        }
    }
}

/// Status line under the rows: red capture notices win over the amber
/// duplicate-binding hint (both must stay visible, one at a time is enough).
#[component]
fn capture_status(hint: Signal<String>, conflict: Option<String>) -> Element {
    let has_hint = !hint.cloned().is_empty();
    rsx! {
        div {
            class: "status",
            if has_hint {
                span { class: "hint-err", "{hint}" }
            } else if let Some(text) = conflict {
                span { class: "hint-warn", "{text}" }
            }
        }
    }
}

/// Static footer: how the toggle works, watchdog note, log location.
#[component]
fn footer_notes() -> Element {
    let log = crate::logging::log_file().map_or_else(
        || "未初始化(需可写的数据目录或 exe 目录)".to_string(),
        |p| p.display().to_string(),
    );
    rsx! {
        div {
            class: "footer",
            "日志:{log}"
        }
    }
}

#[component]
fn title_bar() -> Element {
    rsx! {
        div {
            class: "title",
            onmousedown: move |_| {
                let _ = window().drag_window();
            },
            img { class: "app-icon", src: title_icon_src() }
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
                class: "btn-title btn-close",
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
    let empty = value.cloned() == KEY_NONE;
    let label = key_label(&value.cloned());
    let btn_class = if active {
        "key-btn capturing"
    } else if empty {
        "key-btn empty"
    } else {
        "key-btn"
    };

    rsx! {
        div {
            class: "row",
            span { class: "row-label", "{name}" }
            button {
                class: "{btn_class}",
                onclick: move |_| capturing.set(slot),
                if active {
                    "按下任意键…"
                } else {
                    span { class: "key-name", "{label}" }
                }
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
    crate::tray::apply_master_visuals(cfg.feature_enabled);
    sync_feature_signal(cfg.feature_enabled);
    log::info!("settings saved and applied: {cfg:?}");
    Ok(())
}
