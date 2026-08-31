//! Persistent demo configuration plus runtime settings shared with the
//! signal threads (so changes from the settings page apply live).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

/// Signal source: watch kanata's TCP `LayerChange` stream.
pub const MODE_TCP: u8 = 0;
/// Signal source: watch the raw F24 press/release that kanata emits.
pub const MODE_F24: u8 = 1;
/// Port kanata listens on when started with `-p`.
pub const DEFAULT_TCP_PORT: u16 = 5829;

/// Contents of `%APPDATA%\enable-touchpad\config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// `true` -> TCP mode, `false` -> F24 key mode.
    pub use_tcp: bool,
    /// TCP port of the kanata server.
    pub tcp_port: u16,
    /// Layer name whose activation toggles the touchpad.
    pub layer_name: String,
    /// Show the click-through indicator at the mouse position.
    pub indicator_enabled: bool,
    /// Master switch for the whole CapsLock-layer feature.
    pub feature_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            use_tcp: true,
            tcp_port: DEFAULT_TCP_PORT,
            layer_name: "mouse".to_string(),
            indicator_enabled: true,
            feature_enabled: true,
        }
    }
}

impl AppConfig {
    /// Path of the config file, or `None` when `APPDATA` is unset.
    pub fn path() -> Option<PathBuf> {
        let base = std::env::var_os("APPDATA")?;
        Some(
            PathBuf::from(base)
                .join("enable-touchpad")
                .join("config.json"),
        )
    }

    /// Load the config, falling back to defaults on any problem.
    pub fn load() -> Self {
        let parsed = Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| serde_json::from_str(&text).ok());
        parsed.unwrap_or_default()
    }

    /// Persist the config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path().ok_or_else(|| "APPDATA is not set".to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// Runtime copy of [`AppConfig`] readable from any thread.
pub struct Shared {
    mode: AtomicU8,
    port: Mutex<u16>,
    layer_name: Mutex<String>,
    enabled: AtomicBool,
    indicator: AtomicBool,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

/// Process-wide shared settings, initialised from the persisted config.
pub fn shared() -> &'static Shared {
    SHARED.get_or_init(|| Shared::from_cfg(&AppConfig::load()))
}

impl Shared {
    fn from_cfg(cfg: &AppConfig) -> Self {
        Self {
            mode: AtomicU8::new(if cfg.use_tcp { MODE_TCP } else { MODE_F24 }),
            port: Mutex::new(cfg.tcp_port),
            layer_name: Mutex::new(cfg.layer_name.clone()),
            enabled: AtomicBool::new(cfg.feature_enabled),
            indicator: AtomicBool::new(cfg.indicator_enabled),
        }
    }

    /// Copy current values out of the settings page into the runtime.
    pub fn apply(&self, cfg: &AppConfig) {
        self.mode.store(
            if cfg.use_tcp { MODE_TCP } else { MODE_F24 },
            Ordering::Relaxed,
        );
        if let Ok(mut port) = self.port.lock() {
            *port = cfg.tcp_port;
        }
        if let Ok(mut name) = self.layer_name.lock() {
            (*name).clone_from(&cfg.layer_name);
        }
        self.enabled.store(cfg.feature_enabled, Ordering::Relaxed);
        self.indicator
            .store(cfg.indicator_enabled, Ordering::Relaxed);
    }

    /// `true` when the TCP signal source is selected.
    pub fn is_tcp(&self) -> bool {
        self.mode.load(Ordering::Relaxed) == MODE_TCP
    }

    /// `true` when the F24 key signal source is selected.
    pub fn is_f24(&self) -> bool {
        !self.is_tcp()
    }

    /// Master switch for the CapsLock-layer feature.
    pub fn feature_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Whether the on-screen layer indicator should be shown.
    pub fn indicator_enabled(&self) -> bool {
        self.indicator.load(Ordering::Relaxed)
    }

    /// Current kanata TCP port.
    pub fn port(&self) -> u16 {
        self.port.lock().map_or(DEFAULT_TCP_PORT, |port| *port)
    }

    /// Current layer name to watch for.
    pub fn layer_name(&self) -> String {
        self.layer_name
            .lock()
            .map(|n| n.clone())
            .unwrap_or_default()
    }
}
