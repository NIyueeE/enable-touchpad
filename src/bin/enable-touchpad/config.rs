//! Persistent configuration: the mouse-layer key bindings plus a master
//! switch. Saving regenerates the embedded kanata config and hot-applies it.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bindings offered for the `Q`/`W`/`E` layer keys: `(id, label)`.
pub const MOUSE_ACTIONS: [(&str, &str); 4] = [
    ("left", "左键"),
    ("middle", "中键"),
    ("right", "右键"),
    ("none", "无"),
];

/// Bindings offered for the `Left Alt` layer key: `(id, label)`.
pub const LALT_ACTIONS: [(&str, &str); 2] = [("caps", "CapsLock"), ("none", "无")];

/// Contents of `%APPDATA%\enable-touchpad\config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Master switch: `false` makes `CapsLock` behave stock.
    pub feature_enabled: bool,
    /// Binding id for the `Q` layer key (see [`MOUSE_ACTIONS`]).
    pub key_q: String,
    /// Binding id for the `W` layer key.
    pub key_w: String,
    /// Binding id for the `E` layer key.
    pub key_e: String,
    /// Binding id for the `Left Alt` layer key (see [`LALT_ACTIONS`]).
    pub key_lalt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            feature_enabled: true,
            key_q: "left".to_string(),
            key_w: "middle".to_string(),
            key_e: "right".to_string(),
            key_lalt: "caps".to_string(),
        }
    }
}

impl AppConfig {
    /// Path of the config file, or `None` when `APPDATA` is unset.
    pub fn path() -> Option<PathBuf> {
        Some(app_dir().ok()?.join("config.json"))
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

/// Application data directory, created on demand.
pub fn app_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA").ok_or("APPDATA is not set")?;
    let dir = PathBuf::from(base).join("enable-touchpad");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Map a binding id to the kanata action emitted by a `Q`/`W`/`E` layer key.
pub fn mouse_action(id: &str) -> &'static str {
    match id {
        "left" => "mlft",
        "middle" => "mmid",
        "right" => "mrgt",
        _ => "XX",
    }
}

/// Map a binding id to the kanata action emitted by the `Left Alt` layer key.
pub fn lalt_action(id: &str) -> &'static str {
    match id {
        "caps" => "caps",
        _ => "XX",
    }
}
