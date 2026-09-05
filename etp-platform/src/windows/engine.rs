//! Embedded kanata input engine.
//!
//! Responsibilities:
//! - materialise the generated kanata config at the canonical path;
//! - start the kanata stack on a dedicated thread (LL-hook capture, layer
//!   handling, and a loopback TCP server used only as the internal control
//!   channel);
//! - hot-apply config changes by sending the TCP `Reload` command;
//! - watch layer-change notifications (kanata broadcasts them to every
//!   connected TCP client) and forward them to the watchdog through the
//!   channel passed to [`start`];
//! - expose [`tap_release_fakekey`], the soft Ctrl+Win+F24 toggle used for
//!   state corrections.
//!
//! The `CapsLock` mapping taps Ctrl+Win+F24 once on press and once on release
//! (a toggle for the system's touchpad driver). This app never disables
//! devices.

use super::WindowsPlatform;
use crate::{Platform, PlatformError};
use etp_core::{AppConfig, generate_config_text};
use kanata_state_machine::{Kanata, TcpServer, ValidatedArgs};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

/// Fixed loopback port of the embedded kanata TCP server. Internal control
/// channel only — never exposed in the UI.
const INTERNAL_PORT: u16 = 5829;

/// Bound on the control-channel waits. The settings UI calls `apply_config`
/// synchronously on the UI thread, so a hung connection must fail fast
/// instead of freezing the window forever.
const CONTROL_TIMEOUT: Duration = Duration::from_secs(3);

/// Delay before the layer monitor retries a failed connection.
const LAYER_MONITOR_RETRY: Duration = Duration::from_secs(2);

/// Spawn the embedded kanata stack and the layer monitor (call once).
pub fn start(cfg: &AppConfig, layer_events: SyncSender<bool>) -> Result<(), PlatformError> {
    let cfg_path = write_config_file(cfg)?;
    std::thread::spawn(move || {
        if let Err(e) = run(cfg_path) {
            log::error!("embedded kanata failed to start: {e}");
        }
    });
    std::thread::spawn(move || layer_monitor_loop(&layer_events));
    Ok(())
}

/// Regenerate the kanata config file for `cfg` and hot-reload the running
/// embedded instance.
pub fn apply_config(cfg: &AppConfig) -> Result<(), PlatformError> {
    write_config_file(cfg)?;
    send_reload()?;
    log::info!("config regenerated and hot-applied: {cfg:?}");
    Ok(())
}

fn run(cfg_path: PathBuf) -> Result<(), PlatformError> {
    let addr = format!("127.0.0.1:{INTERNAL_PORT}");
    let tcp_address: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| PlatformError::new(format!("{e}")))?;

    let args = ValidatedArgs {
        paths: vec![cfg_path],
        tcp_server_address: Some(
            addr.parse()
                .map_err(|e| PlatformError::new(format!("{e}")))?,
        ),
        nodelay: true,
    };
    let kanata = Kanata::new_arc(&args).map_err(|e| PlatformError::new(format!("{e:?}")))?;

    let (tx, rx) = sync_channel(100);
    let mut server = TcpServer::new(tcp_address, tx.clone());
    server.start(kanata.clone());
    let (ntx, nrx) = sync_channel(100);
    Kanata::start_processing_loop(kanata.clone(), rx, Some(ntx), true);
    Kanata::start_notification_loop(nrx, server.connections);

    log::info!("embedded kanata started on {addr}");
    // Blocks this thread: installs the low-level keyboard hook and pumps
    // messages for it.
    Kanata::event_loop(kanata, tx).map_err(|e| PlatformError::new(format!("{e:?}")))
}

/// Connects to the embedded server and relays layer-change notifications to
/// the watchdog. Kanata broadcasts `LayerChange` to every connected client,
/// so a plain persistent connection is all it takes; reconnects until the
/// process exits.
fn layer_monitor_loop(layer_events: &SyncSender<bool>) {
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
            if let Some(on) = parse_layer_change(&line)
                && layer_events.send(on).is_err()
            {
                log::warn!("layer monitor sink closed; exiting");
                return;
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
pub fn tap_release_fakekey() -> Result<(), PlatformError> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", INTERNAL_PORT))
        .map_err(|e| PlatformError::new(format!("kanata 控制通道连接失败: {e}")))?;
    stream
        .set_write_timeout(Some(CONTROL_TIMEOUT))
        .map_err(|e| PlatformError::new(format!("kanata 控制通道超时设置失败: {e}")))?;
    stream
        .write_all(br#"{"ActOnFakeKey":{"name":"release-tap","action":"Tap"}}"#)
        .and_then(|()| stream.write_all(b"\n"))
        .map_err(|e| PlatformError::new(format!("kanata fake-key 命令发送失败: {e}")))?;
    log::info!("sent release-tap (Ctrl+Win+F24) via fake key");
    Ok(())
}

/// Materialise the generated config for `cfg` at the canonical path.
fn write_config_file(cfg: &AppConfig) -> Result<PathBuf, PlatformError> {
    let path = config_path()?;
    std::fs::write(&path, generate_config_text(cfg)).map_err(PlatformError::from)?;
    Ok(path)
}

fn config_path() -> Result<PathBuf, PlatformError> {
    Ok(WindowsPlatform.app_data_dir()?.join("kanata.kbd"))
}

/// Send the `Reload` command to the embedded kanata TCP server.
fn send_reload() -> Result<(), PlatformError> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", INTERNAL_PORT))
        .map_err(|e| PlatformError::new(format!("kanata 控制通道连接失败: {e}")))?;
    stream
        .set_write_timeout(Some(CONTROL_TIMEOUT))
        .and_then(|()| stream.set_read_timeout(Some(CONTROL_TIMEOUT)))
        .map_err(|e| PlatformError::new(format!("kanata 控制通道超时设置失败: {e}")))?;
    stream
        .write_all(br#"{"Reload":{"wait":true}}"#)
        .and_then(|()| stream.write_all(b"\n"))
        .map_err(|e| PlatformError::new(format!("kanata reload 命令发送失败: {e}")))?;
    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|e| PlatformError::new(format!("kanata reload 响应读取失败: {e}")))?;
    log::info!("kanata reload response: {}", response.trim());
    Ok(())
}
