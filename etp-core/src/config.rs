//! Configuration model for the enable-touchpad settings page.
//!
//! Binding values are W3C `KeyboardEvent.code` strings (`"KeyQ"`, `"AltLeft"`,
//! `"F5"`). kanata's parser accepts these verbatim (see `str_to_oscode` in
//! kanata-parser v1.11.0, `parser/src/keys/mod.rs`), so captured codes go
//! straight into the generated config. Legacy short forms from earlier
//! versions (`"q"`, `"lalt"`) stay valid and loadable.

use serde::{Deserialize, Serialize};

/// Contents of `%APPDATA%\enable-touchpad\config.json` on Windows. The path is
/// platform-specific and lives in the platform layer; this struct only owns
/// the data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// Parse a config file that was previously written by
    /// [`AppConfig::to_json_pretty`].
    ///
    /// # Errors
    ///
    /// Returns the [`serde_json`] error message if `text` is not valid JSON
    /// or does not match the [`AppConfig`] shape.
    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    /// Serialize the config to pretty-printed JSON for storage.
    ///
    /// # Errors
    ///
    /// Returns the [`serde_json`] error message if serialisation fails.
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn default_values_round_trip_through_json() {
        let cfg = AppConfig::default();
        let Ok(json) = cfg.to_json_pretty() else {
            panic!("failed to serialize default config");
        };
        let Ok(parsed) = AppConfig::from_json(&json) else {
            panic!("failed to parse serialized default config");
        };
        assert_eq!(parsed, cfg);
    }
}
