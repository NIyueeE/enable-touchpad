//! Non-Windows fallback adapter.
//!
//! The demo currently only ships a Windows adapter. This fallback keeps the
//! crate compiling and the host-side Linux gates green; it is the seed for
//! future Linux/macOS adapters.

use super::{Platform, PlatformError};
use etp_core::AppConfig;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

/// Process-wide fallback adapter instance.
pub static FALLBACK_PLATFORM: UnsupportedPlatform = UnsupportedPlatform;

/// Adapter for platforms that do not have an enable-touchpad backend yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnsupportedPlatform;

impl Platform for UnsupportedPlatform {
    fn app_data_dir(&self) -> Result<PathBuf, PlatformError> {
        Err(PlatformError::new(
            "enable-touchpad does not have a platform backend for this OS yet",
        ))
    }

    fn start_engine(
        &self,
        _cfg: &AppConfig,
        _layer_events: SyncSender<bool>,
    ) -> Result<(), PlatformError> {
        Err(PlatformError::new(
            "enable-touchpad does not have a platform backend for this OS yet",
        ))
    }

    fn apply_engine_config(&self, _cfg: &AppConfig) -> Result<(), PlatformError> {
        Err(PlatformError::new(
            "enable-touchpad does not have a platform backend for this OS yet",
        ))
    }

    fn tap_toggle_chord(&self) -> Result<(), PlatformError> {
        Err(PlatformError::new(
            "enable-touchpad does not have a platform backend for this OS yet",
        ))
    }

    fn touchpad_enabled(&self) -> Result<bool, PlatformError> {
        Err(PlatformError::new(
            "enable-touchpad does not have a platform backend for this OS yet",
        ))
    }
}
