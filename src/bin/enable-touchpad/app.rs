//! Dioxus UI: the settings window, tray entry points, and the click-through
//! layer indicator that follows the mouse.
//!
//! Threading model: worker threads (kanata TCP, F24 watcher, tray, toggle)
//! write UI state directly through `SyncSignal`s stored in statics — plain
//! 0.7 `Signal`s use thread-local storage and must stay on the UI thread.

use crate::config::{self, AppConfig};
use crate::touchpad;
use crate::tray::TrayAction;
use device_query::DeviceState;
use dioxus::core::VirtualDom;
use dioxus::desktop::tao;
use dioxus::desktop::tao::dpi::PhysicalPosition;
use dioxus::desktop::{
    Config, DesktopContext, LogicalSize, WindowBuilder, WindowCloseBehaviour, window,
};
use dioxus::prelude::*;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::time::sleep;

/// Touchpad state text, written by worker threads.
static STATUS: OnceLock<SyncSignal<String>> = OnceLock::new();
/// Raw device rows from the last query.
static DEVICES: OnceLock<SyncSignal<String>> = OnceLock::new();
/// Rolling diagnostics log (oldest first).
static LOGS: OnceLock<SyncSignal<Vec<String>>> = OnceLock::new();
/// Main window handle; tao windows are `Send`, so the tray thread can use it.
static MAIN_WINDOW: OnceLock<Arc<tao::window::Window>> = OnceLock::new();
/// Command channel to the indicator window task.
static INDICATOR_TX: OnceLock<UnboundedSender<WinCmd>> = OnceLock::new();

/// Commands accepted by the indicator window task.
#[derive(Debug)]
pub enum WinCmd {
    /// Show the indicator at the current mouse position.
    Show,
    /// Hide the indicator.
    Hide,
}

/// Configure and launch the desktop app.
pub fn launch() {
    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("enable-touchpad")
                .with_inner_size(LogicalSize::new(680.0, 620.0)),
        )
        .with_close_behaviour(WindowCloseBehaviour::WindowHides);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui_root);
}

/// A layer signal arrived from kanata (TCP or F24): drive indicator and touchpad.
pub fn handle_layer(active: bool) {
    let shared = config::shared();
    push_log(format!("层信号: {}", if active { "激活" } else { "还原" }));
    if shared.indicator_enabled()
        && let Some(tx) = INDICATOR_TX.get()
    {
        let _ = tx.send(if active { WinCmd::Show } else { WinCmd::Hide });
    }
    if shared.feature_enabled() {
        spawn_toggle(active);
    }
}

/// A tray menu entry was selected (runs on the tray forwarder thread).
pub fn handle_tray(action: TrayAction) {
    match action {
        TrayAction::OpenMain => {
            if let Some(win) = MAIN_WINDOW.get() {
                win.set_visible(true);
                win.set_focus();
            }
        }
        TrayAction::Enable => spawn_toggle(true),
        TrayAction::Disable => spawn_toggle(false),
        TrayAction::Refresh => spawn_query(),
        TrayAction::Quit => std::process::exit(0),
    }
}

/// Append a line to the diagnostics log, keeping it bounded.
pub fn push_log(line: String) {
    if let Some(mut logs) = LOGS.get().copied() {
        logs.with_mut(|log| {
            log.push(line);
            if log.len() > 12 {
                log.remove(0);
            }
        });
    }
}

/// Toggle the touchpad on a worker thread and report back into the UI state.
pub fn spawn_toggle(enable: bool) {
    std::thread::spawn(move || {
        let result = touchpad::set_enabled(enable);
        apply_report(result);
    });
}

/// Re-query the touchpad state on a worker thread.
pub fn spawn_query() {
    std::thread::spawn(|| {
        let result = touchpad::query();
        apply_report(result);
    });
}

fn apply_report(result: Result<touchpad::TouchpadReport, String>) {
    match result {
        Ok(report) => {
            if let Some(mut status) = STATUS.get().copied() {
                status.set(report.status_text.clone());
            }
            if let Some(mut devices) = DEVICES.get().copied() {
                devices.set(report.lines.join("\n"));
            }
            push_log(format!("触摸板状态: {}", report.status_text));
        }
        Err(e) => {
            if let Some(mut status) = STATUS.get().copied() {
                status.set("操作失败".to_string());
            }
            push_log(format!("触摸板操作失败: {e}"));
        }
    }
}

fn ui_root() -> Element {
    let status = use_signal_sync(|| "检测中…".to_string());
    let devices = use_signal_sync(String::new);
    let logs = use_signal_sync(Vec::<String>::new);
    let _ = STATUS.set(status);
    let _ = DEVICES.set(devices);
    let _ = LOGS.set(logs);

    let AppConfig {
        feature_enabled: initial_feature,
        use_tcp: initial_tcp,
        tcp_port: initial_port,
        layer_name: initial_layer,
        indicator_enabled: initial_indicator,
    } = AppConfig::load();

    let feature = use_signal(move || initial_feature);
    let use_tcp = use_signal(move || initial_tcp);
    let port_text = use_signal(move || initial_port.to_string());
    let layer_name = use_signal(move || initial_layer.clone());
    let indicator = use_signal(move || initial_indicator);

    use_future(|| async move {
        let _ = MAIN_WINDOW.set(Arc::clone(&window().window));
        dioxus::core::spawn(indicator_manager());
        spawn_query();
    });

    rsx! {
        div {
            style: "background:#14161a;color:#e6e9ef;font-family:'Segoe UI','Microsoft YaHei',system-ui,sans-serif;min-height:100vh;padding:18px 20px;box-sizing:border-box;",
            h1 { style: "font-size:18px;margin:0 0 14px 0;", "enable-touchpad — Windows 可行性 Demo" }
            status_section { status, devices }
            signal_section { use_tcp, port_text, layer_name, indicator, feature }
            log_section { logs }
            div {
                style: "color:#5d6572;font-size:12px;line-height:1.8;",
                "关闭窗口 = 隐藏到托盘(托盘菜单可退出)。触摸板启停需要管理员权限,请以管理员身份运行。"
            }
        }
    }
}

#[component]
fn status_section(mut status: SyncSignal<String>, mut devices: SyncSignal<String>) -> Element {
    let section = "background:#1c1f26;border-radius:12px;padding:14px 16px;margin-bottom:12px;";
    let label = "color:#9aa3b2;font-size:12px;margin-bottom:6px;";
    let button = "background:#2f6fd6;color:#fff;border:none;border-radius:8px;padding:7px 14px;margin-right:8px;cursor:pointer;font-size:13px;";
    let status_text = status.cloned();
    let device_text = devices.cloned();

    rsx! {
        div {
            style: "{section}",
            div { style: "{label}", "触摸板状态" }
            div { style: "font-size:20px;font-weight:600;margin-bottom:10px;", "{status_text}" }
            div {
                button { style: "{button}", onclick: move |_| spawn_toggle(true), "启用触摸板" }
                button { style: "{button}", onclick: move |_| spawn_toggle(false), "禁用触摸板" }
                button { style: "{button}", onclick: move |_| spawn_query(), "刷新检测" }
            }
            div {
                style: "font-size:12px;color:#9aa3b2;white-space:pre-wrap;font-family:Consolas,monospace;margin-top:8px;",
                "{device_text}"
            }
        }
    }
}

#[component]
fn signal_section(
    mut use_tcp: Signal<bool>,
    mut port_text: Signal<String>,
    mut layer_name: Signal<String>,
    mut indicator: Signal<bool>,
    mut feature: Signal<bool>,
) -> Element {
    let section = "background:#1c1f26;border-radius:12px;padding:14px 16px;margin-bottom:12px;";
    let label = "color:#9aa3b2;font-size:12px;margin-bottom:6px;";
    let button = "background:#2f6fd6;color:#fff;border:none;border-radius:8px;padding:7px 14px;margin-right:8px;cursor:pointer;font-size:13px;";
    let input = "background:#14161a;color:#e6e9ef;border:1px solid #343a46;border-radius:6px;padding:4px 8px;";
    let row = "display:flex;align-items:center;margin-bottom:8px;gap:8px;";

    rsx! {
        div {
            style: "{section}",
            div { style: "{label}", "信号源(来自 kanata)" }
            div {
                style: "{row}",
                input {
                    r#type: "radio",
                    name: "signal",
                    checked: use_tcp.cloned(),
                    onclick: move |_| use_tcp.set(true),
                }
                span { "TCP 模式 — 内嵌 kanata 的层广播 (127.0.0.1:{port_text},改端口需重启)" }
            }
            div {
                style: "{row}",
                input {
                    r#type: "radio",
                    name: "signal",
                    checked: !use_tcp.cloned(),
                    onclick: move |_| use_tcp.set(false),
                }
                span { "F24 按键模式 — 监听内嵌 kanata 输出的 Ctrl+Win+F24 按下/释放" }
            }
            div {
                style: "{row}",
                span { "端口: " }
                input {
                    r#type: "text",
                    value: "{port_text}",
                    style: "{input} width:80px;",
                    oninput: move |e| port_text.set(e.value()),
                }
                span { "层名: " }
                input {
                    r#type: "text",
                    value: "{layer_name}",
                    style: "{input} width:120px;",
                    oninput: move |e| layer_name.set(e.value()),
                }
            }
            div {
                style: "{row}",
                input {
                    r#type: "checkbox",
                    checked: feature.cloned(),
                    onclick: move |_| feature.set(!feature.cloned()),
                }
                span { "总开关: CapsLock 层功能(按住=启用触摸板,松开=禁用)" }
            }
            div {
                style: "{row}",
                input {
                    r#type: "checkbox",
                    checked: indicator.cloned(),
                    onclick: move |_| indicator.set(!indicator.cloned()),
                }
                span { "鼠标处显示层激活提示" }
                button {
                    style: "{button}",
                    onclick: move |_| preview_indicator(),
                    "预览提示"
                }
            }
            div {
                button {
                    style: "{button}",
                    onclick: move |_| {
                        let cfg = current_cfg(use_tcp, &port_text, &layer_name, indicator, feature);
                        config::shared().apply(&cfg);
                        push_log(format!(
                            "设置已应用: mode={}, port={}, layer={}",
                            if cfg.use_tcp { "TCP" } else { "F24" },
                            cfg.tcp_port,
                            cfg.layer_name
                        ));
                    },
                    "应用设置"
                }
                button {
                    style: "{button}",
                    onclick: move |_| {
                        let cfg = current_cfg(use_tcp, &port_text, &layer_name, indicator, feature);
                        match cfg.save() {
                            Ok(()) => push_log("配置已保存".into()),
                            Err(e) => push_log(format!("配置保存失败: {e}")),
                        }
                    },
                    "保存配置"
                }
            }
        }
    }
}

#[component]
fn log_section(logs: SyncSignal<Vec<String>>) -> Element {
    let section = "background:#1c1f26;border-radius:12px;padding:14px 16px;margin-bottom:12px;";
    let label = "color:#9aa3b2;font-size:12px;margin-bottom:6px;";
    let log_lines = logs.cloned();

    rsx! {
        div {
            style: "{section}",
            div { style: "{label}", "运行日志" }
            div {
                style: "font-size:12px;color:#9aa3b2;white-space:pre-wrap;font-family:Consolas,monospace;line-height:1.7;",
                "{log_text(&log_lines)}"
            }
        }
    }
}

fn current_cfg(
    use_tcp: Signal<bool>,
    port_text: &Signal<String>,
    layer_name: &Signal<String>,
    indicator: Signal<bool>,
    feature: Signal<bool>,
) -> AppConfig {
    AppConfig {
        use_tcp: use_tcp.cloned(),
        tcp_port: port_text
            .cloned()
            .parse()
            .unwrap_or(config::DEFAULT_TCP_PORT),
        layer_name: layer_name.cloned(),
        indicator_enabled: indicator.cloned(),
        feature_enabled: feature.cloned(),
    }
}

fn log_text(lines: &[String]) -> String {
    if lines.is_empty() {
        "(暂无)".to_string()
    } else {
        lines.join("\n")
    }
}

fn preview_indicator() {
    if let Some(tx) = INDICATOR_TX.get() {
        let _ = tx.send(WinCmd::Show);
        dioxus::core::spawn(async {
            sleep(Duration::from_millis(1600)).await;
            if let Some(tx) = INDICATOR_TX.get() {
                let _ = tx.send(WinCmd::Hide);
            }
        });
    } else {
        push_log("指示器窗口尚未就绪".into());
    }
}

/// Create the indicator window and keep it glued to the mouse while shown.
async fn indicator_manager() {
    let (tx, mut rx) = unbounded_channel::<WinCmd>();
    let _ = INDICATOR_TX.set(tx);

    let builder = WindowBuilder::new()
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_visible(false)
        .with_inner_size(LogicalSize::new(240.0, 56.0));
    let config = Config::new()
        .with_window(builder)
        .with_background_color((0x00, 0x00, 0x00, 0x00));

    let Ok(indicator) = window()
        .new_window(VirtualDom::new(indicator_view), config)
        .try_resolve()
        .await
    else {
        push_log("指示器窗口创建失败".into());
        return;
    };
    let _ = indicator.set_ignore_cursor_events(true);

    let mouse = DeviceState::new();
    let mut visible = false;
    let mut tick = tokio::time::interval(Duration::from_millis(33));
    loop {
        tokio::select! {
            command = rx.recv() => match command {
                None => break,
                Some(WinCmd::Show) => {
                    visible = true;
                    place_at_mouse(&indicator, &mouse);
                    indicator.set_visible(true);
                }
                Some(WinCmd::Hide) => {
                    visible = false;
                    indicator.set_visible(false);
                }
            },
            _ = tick.tick() => {
                if visible {
                    place_at_mouse(&indicator, &mouse);
                }
            }
        }
    }
}

fn place_at_mouse(indicator: &DesktopContext, mouse: &DeviceState) {
    let pos = mouse.query_pointer().coords;
    indicator.set_outer_position(PhysicalPosition::new(pos.0 + 18, pos.1 + 26));
}

fn indicator_view() -> Element {
    rsx! {
        div {
            style: "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;background:transparent;pointer-events:none;",
            div {
                style: "background:rgba(31,111,224,0.92);color:#fff;font:600 15px/1.4 'Segoe UI','Microsoft YaHei',system-ui,sans-serif;padding:10px 16px;border-radius:10px;white-space:nowrap;",
                "🖱️ 鼠标层已激活 (CapsLock)"
            }
        }
    }
}
