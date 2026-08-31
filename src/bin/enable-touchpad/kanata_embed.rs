//! Embeds kanata as an in-process library so the demo ships as a single exe.
//!
//! Startup mirrors kanata's own `win_gui.rs`: build `ValidatedArgs`, create
//! the state machine, start the TCP server (the existing `signal.rs` TCP
//! reader connects to it for `LayerChange` events), start the processing and
//! notification loops, then block this thread on the LL-hook event loop.
//! The keyboard config is embedded into the binary at compile time.

use kanata_state_machine::{Kanata, TcpServer, ValidatedArgs};
use std::sync::mpsc::sync_channel;

/// The keyboard layer config, compiled into the binary.
pub const CFG_TEXT: &str = include_str!("../../../demo/kanata/enable-touchpad.kbd");

/// Spawn the embedded kanata stack on a dedicated thread (call once).
pub fn start() {
    std::thread::spawn(|| {
        if let Err(e) = run() {
            crate::app::push_log(format!("内嵌 kanata 启动失败: {e}"));
        }
    });
}

fn run() -> Result<(), String> {
    let port = crate::config::shared().port();
    let addr = format!("127.0.0.1:{port}");

    // Materialise the embedded config so kanata's file-based API can read it
    // (and so users can tweak the file for experiments).
    let cfg_path = config_path()?;
    if let Some(parent) = cfg_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&cfg_path, CFG_TEXT).map_err(|e| e.to_string())?;

    let tcp_address: std::net::SocketAddr = addr.parse().map_err(|e| format!("{e}"))?;
    let args = ValidatedArgs {
        paths: vec![cfg_path],
        tcp_server_address: Some(addr.parse().map_err(|e| format!("tcp 地址解析失败: {e}"))?),
        nodelay: true,
    };

    let kanata = Kanata::new_arc(&args).map_err(|e| format!("{e:?}"))?;

    let (tx, rx) = sync_channel(100);
    let mut server = TcpServer::new(tcp_address, tx.clone());
    server.start(kanata.clone());
    let (ntx, nrx) = sync_channel(100);
    Kanata::start_processing_loop(kanata.clone(), rx, Some(ntx), true);
    Kanata::start_notification_loop(nrx, server.connections);

    crate::app::push_log(format!(
        "内嵌 kanata 已启动:层捕获运行中,TCP 127.0.0.1:{port}"
    ));
    // Blocks this thread: installs the LL keyboard hook and pumps messages.
    Kanata::event_loop(kanata, tx).map_err(|e| format!("{e:?}"))
}

fn config_path() -> Result<std::path::PathBuf, String> {
    let base = std::env::var_os("APPDATA").ok_or("APPDATA is not set")?;
    Ok(std::path::PathBuf::from(base)
        .join("enable-touchpad")
        .join("kanata.kbd"))
}
