//! Windows platform adapter: embedded kanata input engine plus the Win32
//! precision-touchpad state query. UI/tray are NOT here — they live in the
//! application layer, which programs against the [`Platform`](super::Platform)
//! trait.

mod engine;

use super::{Platform, PlatformError};
use etp_core::AppConfig;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

/// Process-wide Windows adapter instance.
pub static WINDOWS_PLATFORM: WindowsPlatform = WindowsPlatform;

/// Windows implementation of the platform capability surface.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsPlatform;

impl Platform for WindowsPlatform {
    fn app_data_dir(&self) -> Result<PathBuf, PlatformError> {
        let base =
            std::env::var_os("APPDATA").ok_or_else(|| PlatformError::new("APPDATA is not set"))?;
        let dir = PathBuf::from(base).join("enable-touchpad");
        std::fs::create_dir_all(&dir).map_err(PlatformError::from)?;
        Ok(dir)
    }

    fn start_engine(
        &self,
        cfg: &AppConfig,
        layer_events: SyncSender<bool>,
    ) -> Result<(), PlatformError> {
        engine::start(cfg, layer_events)
    }

    fn apply_engine_config(&self, cfg: &AppConfig) -> Result<(), PlatformError> {
        engine::apply_config(cfg)
    }

    fn tap_toggle_chord(&self) -> Result<(), PlatformError> {
        engine::tap_release_fakekey()
    }

    fn touchpad_enabled(&self) -> Result<bool, PlatformError> {
        etp_ffi::touchpad_enabled().map_err(|e| PlatformError::new(format!("{e:?}")))
    }
}
