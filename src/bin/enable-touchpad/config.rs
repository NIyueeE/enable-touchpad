//! Persistent configuration: which keyboard key performs each mouse action
//! inside the `CapsLock` `mouse` layer, plus a master switch. Saving
//! regenerates the embedded kanata config and hot-applies it.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Keyboard keys that can be assigned to a mouse action inside the layer:
/// `(id, label)`. The physical `CapsLock` key is always the layer trigger.
pub const KEY_CHOICES: [(&str, &str); 5] = [
    ("q", "Q"),
    ("w", "W"),
    ("e", "E"),
    ("lalt", "Left Alt"),
    ("none", "无"),
];

/// Contents of `%APPDATA%\enable-touchpad\config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Master switch: `false` makes `CapsLock` behave stock.
    pub feature_enabled: bool,
    /// Layer key that performs the left mouse click.
    pub left_click_key: String,
    /// Layer key that performs the middle mouse click.
    pub middle_click_key: String,
    /// Layer key that performs the right mouse click.
    pub right_click_key: String,
    /// Layer key that acts as `CapsLock`.
    pub capslock_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            feature_enabled: true,
            left_click_key: "q".to_string(),
            middle_click_key: "w".to_string(),
            right_click_key: "e".to_string(),
            capslock_key: "lalt".to_string(),
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

/// The kanata action for a `mouse`-layer slot, given the action-to-key
/// mapping. Unassigned slots emit `XX` (blocked). First claim wins when two
/// actions share a key.
pub fn layer_slot_action(
    slot: &str,
    left_click_key: &str,
    middle_click_key: &str,
    right_click_key: &str,
    capslock_key: &str,
) -> &'static str {
    if slot == capslock_key {
        "caps"
    } else if slot == left_click_key {
        "mlft"
    } else if slot == middle_click_key {
        "mmid"
    } else if slot == right_click_key {
        "mrgt"
    } else {
        "XX"
    }
}
