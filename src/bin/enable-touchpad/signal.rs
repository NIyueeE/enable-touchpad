//! Signal sources for the demo: kanata's TCP `LayerChange` stream and the
//! raw F24 press/release that kanata emits. Both report straight into
//! [`crate::app::handle_layer`].

use crate::app;
use crate::config;
use rdev::{Event, EventType, Key};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Windows virtual-key code for F24 (rdev reports unmapped VKs as
/// `Key::Unknown(vk)`; kanata emits `f24` = VK 0x87).
const VK_F24: u32 = 0x87;

/// Start every background signal source (call once from `main`).
pub fn spawn_all() {
    spawn_tcp();
    spawn_f24();
}

/// TCP reader: connects to kanata, parses `{"LayerChange":{"new":…}}` lines.
fn spawn_tcp() {
    std::thread::spawn(|| {
        let mut failures = 0u32;
        loop {
            let shared = config::shared();
            if shared.is_f24() || !shared.feature_enabled() {
                std::thread::sleep(Duration::from_millis(400));
                continue;
            }
            let port = shared.port();
            let layer = shared.layer_name();
            if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
                failures = 0;
                app::push_log(format!("已连接 kanata TCP 127.0.0.1:{port}"));
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let mut last: Option<bool> = None;
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if let Some(active) = parse_layer_change(&line, &layer)
                                && last != Some(active)
                            {
                                last = Some(active);
                                app::handle_layer(active);
                            }
                        }
                    }
                }
                app::push_log("kanata TCP 断开,2s 后重连".into());
            } else {
                failures += 1;
                if failures <= 1 {
                    app::push_log(format!(
                        "连接 kanata 127.0.0.1:{port} 失败(未运行?),每 2s 重试"
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(2000));
        }
    });
}

/// Extract the layer-change state from one kanata TCP line.
fn parse_layer_change(line: &str, layer: &str) -> Option<bool> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let new = value.get("LayerChange")?.get("new")?.as_str()?;
    Some(new == layer)
}

/// F24 watcher: kanata holds/releases F24 while `CapsLock` is held, so the
/// press enables the layer and the release restores it. Auto-repeat is
/// collapsed with an atomic flag.
fn spawn_f24() {
    std::thread::spawn(|| {
        let held = AtomicBool::new(false);
        let callback = move |event: Event| {
            let shared = config::shared();
            if shared.is_tcp() || !shared.feature_enabled() {
                return;
            }
            match event.event_type {
                EventType::KeyPress(Key::Unknown(VK_F24)) => {
                    if !held.swap(true, Ordering::SeqCst) {
                        app::handle_layer(true);
                    }
                }
                EventType::KeyRelease(Key::Unknown(VK_F24))
                    if held.swap(false, Ordering::SeqCst) =>
                {
                    app::handle_layer(false);
                }
                _ => {}
            }
        };
        if let Err(e) = rdev::listen(callback) {
            app::push_log(format!("F24 监听线程退出: {e:?}"));
        }
    });
}
