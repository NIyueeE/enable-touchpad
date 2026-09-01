//! Embeds kanata as an in-process library so the demo ships as a single exe.
//!
//! Responsibilities:
//! - generate the kanata layer config from [`crate::etp_core::AppConfig`]
//!   (see [`crate::etp_core::generate_config_text`]) and write it to
//!   `%APPDATA%\enable-touchpad\kanata.kbd`;
//! - start the kanata stack on a dedicated thread (LL-hook capture, layer
//!   handling, and a loopback TCP server used only as the internal control
//!   channel);
//! - hot-apply config changes by sending the TCP `Reload` command, so saving
//!   in the settings page takes effect without a restart;
//! - watch layer-change notifications (kanata broadcasts them to every
//!   connected TCP client) and feed them to the touchpad state watchdog;
//! - expose [`tap_release_fakekey`], the soft Ctrl+Win+F24 toggle used for
//!   state corrections.
//!
//! The `CapsLock` mapping taps Ctrl+Win+F24 once on press and once on
//! release (a toggle for the system's touchpad driver). This app never
//! disables devices.

use crate::etp_core::{self, AppConfig};
use kanata_state_machine::{Kanata, TcpServer, ValidatedArgs};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::sync_channel;
use std::time::Duration;

/// Fixed loopback port of the embedded kanata TCP server. Internal control
/// channel only — never exposed in the UI.
const INTERNAL_PORT: u16 = 5829;

/// Delay before the layer monitor retries a failed connection.
const LAYER_MONITOR_RETRY: Duration = Duration::from_secs(2);

/// Spawn the embedded kanata stack and the layer monitor (call once).
pub fn start() {
    std::thread::spawn(|| {
        if let Err(e) = run() {
            log::error!("embedded kanata failed to start: {e}");
        }
    });
    std::thread::spawn(layer_monitor_loop);
}

/// Regenerate the kanata config file for `cfg` and hot-reload the running
/// embedded instance.
pub fn apply_config(cfg: &AppConfig) -> Result<(), String> {
    write_config_file(cfg)?;
    send_reload()?;
    log::info!("config regenerated and hot-applied: {cfg:?}");
    Ok(())
}

fn run() -> Result<(), String> {
    let cfg = AppConfig::load();
    let cfg_path = write_config_file(&cfg)?;
    let addr = format!("127.0.0.1:{INTERNAL_PORT}");
    let tcp_address: std::net::SocketAddr = addr.parse().map_err(|e| format!("{e}"))?;

    let args = ValidatedArgs {
        paths: vec![cfg_path],
        tcp_server_address: Some(addr.parse().map_err(|e| format!("{e}"))?),
        nodelay: true,
    };
    let kanata = Kanata::new_arc(&args).map_err(|e| format!("{e:?}"))?;

    let (tx, rx) = sync_channel(100);
    let mut server = TcpServer::new(tcp_address, tx.clone());
    server.start(kanata.clone());
    let (ntx, nrx) = sync_channel(100);
    Kanata::start_processing_loop(kanata.clone(), rx, Some(ntx), true);
    Kanata::start_notification_loop(nrx, server.connections);

    log::info!("embedded kanata started on {addr}");
    // Blocks this thread: installs the low-level keyboard hook and pumps
    // messages for it.
    Kanata::event_loop(kanata, tx).map_err(|e| format!("{e:?}"))
}

/// Connects to the embedded server and relays layer-change notifications to
/// the touchpad watchdog. Kanata broadcasts `LayerChange` to every connected
/// client, so a plain persistent connection is all it takes; reconnects
/// until the process exits.
fn layer_monitor_loop() {
    loop {
        let Ok(stream) = std::net::TcpStream::connect(("127.0.0.1", INTERNAL_PORT)) else {
            // Kanata is not up (yet); quiet retry.
            std::thread::sleep(LAYER_MONITOR_RETRY);
            continue;
        };
        log::info!("layer monitor connected");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        loop {
            line.clear();
            let Ok(count) = reader.read_line(&mut line) else {
                log::warn!("layer monitor disconnected; retrying");
                break;
            };
            if count == 0 {
                log::warn!("layer monitor disconnected; retrying");
                break;
            }
            if let Some(on) = parse_layer_change(&line) {
                log::info!(
                    "layer change: touchpad should be {}",
                    if on { "ON" } else { "OFF" }
                );
                // The press/release chord taps fly together with the
                // layer change; let them land before the watchdog
                // samples the state.
                crate::touchpad_state::mark_tap_now();
                crate::touchpad_state::set_expected(on);
            }
        }
        std::thread::sleep(LAYER_MONITOR_RETRY);
    }
}

/// Extract `{"LayerChange":{"new":"<layer>"}}` — `Some(true)` when the mouse
/// layer became active, `Some(false)` when it left, `None` otherwise.
fn parse_layer_change(line: &str) -> Option<bool> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let layer = value.get("LayerChange")?.get("new")?.as_str()?.to_string();
    Some(layer == "mouse")
}

/// Fire the `release-tap` fake key (one Ctrl+Win+F24 chord) through the
/// embedded kanata instance. The state watchdog uses this to force the
/// touchpad off without touching any device. Success produces no response,
/// so the connection is dropped right after sending.
pub fn tap_release_fakekey() -> Result<(), String> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", INTERNAL_PORT))
        .map_err(|e| format!("kanata 控制通道连接失败: {e}"))?;
    stream
        .write_all(br#"{"ActOnFakeKey":{"name":"release-tap","action":"Tap"}}"#)
        .and_then(|()| stream.write_all(b"\n"))
        .map_err(|e| format!("kanata fake-key 命令发送失败: {e}"))?;
    log::info!("sent release-tap (Ctrl+Win+F24) via fake key");
    Ok(())
}

/// Materialise the generated config for `cfg` at the canonical path.
fn write_config_file(cfg: &AppConfig) -> Result<std::path::PathBuf, String> {
    let path = config_path()?;
    std::fs::write(&path, etp_core::generate_config_text(cfg)).map_err(|e| e.to_string())?;
    Ok(path)
}

fn config_path() -> Result<std::path::PathBuf, String> {
    Ok(etp_core::app_dir()?.join("kanata.kbd"))
}

/// Send the `Reload` command to the embedded kanata TCP server.
fn send_reload() -> Result<(), String> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", INTERNAL_PORT))
        .map_err(|e| format!("kanata 控制通道连接失败: {e}"))?;
    stream
        .write_all(br#"{"Reload":{"wait":true}}"#)
        .and_then(|()| stream.write_all(b"\n"))
        .map_err(|e| format!("kanata reload 命令发送失败: {e}"))?;
    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|e| format!("kanata reload 响应读取失败: {e}"))?;
    log::info!("kanata reload response: {}", response.trim());
    Ok(())
}
