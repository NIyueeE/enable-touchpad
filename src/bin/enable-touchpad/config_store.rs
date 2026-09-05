//! Application-layer config persistence.
//!
//! The platform layer owns the data directory; this module only decides the
//! file name and delegates serialisation to `etp_core`.

use etp_core::AppConfig;
use etp_platform::{Platform, PlatformError};
use std::path::PathBuf;

/// Name of the JSON settings file inside the platform data directory.
const CONFIG_FILE_NAME: &str = "config.json";

/// Load the config, falling back to defaults on any problem.
pub fn load(platform: &dyn Platform) -> AppConfig {
    config_path(platform)
        .and_then(|p| std::fs::read_to_string(p).map_err(PlatformError::from))
        .and_then(|text| AppConfig::from_json(&text).map_err(PlatformError::new))
        .unwrap_or_default()
}

/// Persist the config to disk atomically: write a sibling temp file first and
/// rename it over `config.json`. A crash mid-write then cannot leave a
/// half-written config behind — [`load`] silently falls back to defaults on a
/// parse error, so a corrupt file would quietly discard the user's bindings.
pub fn save(platform: &dyn Platform, cfg: &AppConfig) -> Result<(), String> {
    let path = config_path(platform).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = cfg.to_json_pretty()?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    // `fs::rename` replaces the destination on both Unix and Windows.
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

fn config_path(platform: &dyn Platform) -> Result<PathBuf, PlatformError> {
    Ok(platform.app_data_dir()?.join(CONFIG_FILE_NAME))
}
