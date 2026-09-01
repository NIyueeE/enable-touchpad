//! The platform capability surface used by the application layer.

use crate::error::PlatformError;
use etp_core::AppConfig;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

/// OS-specific services that the application layer needs.
///
/// Every method is called from multiple threads; implementors must be cheap
/// to share (`&'static`) and internally synchronised where needed.
pub trait Platform: Send + Sync {
    /// Directory for config, generated kanata config, and logs. The directory
    /// is created on first use.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError`] when the OS-specific data directory cannot
    /// be determined or created.
    fn app_data_dir(&self) -> Result<PathBuf, PlatformError>;

    /// Start the input engine with `cfg`. Engine startup is asynchronous: the
    /// implementation spawns its own threads and returns once they are
    /// started (or immediately if startup is deliberately deferred).
    ///
    /// `layer_events` receives `true` when the mouse layer becomes active and
    /// `false` when it is left. The engine owns a sender clone; the caller
    /// keeps the receiver.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError`] when the engine configuration cannot be
    /// materialised or the engine threads cannot be spawned.
    fn start_engine(
        &self,
        cfg: &AppConfig,
        layer_events: SyncSender<bool>,
    ) -> Result<(), PlatformError>;

    /// Regenerate the engine configuration for `cfg` and hot-apply it to the
    /// running engine instance.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError`] when the config file cannot be written or
    /// the running engine cannot be reached for reload.
    fn apply_engine_config(&self, cfg: &AppConfig) -> Result<(), PlatformError>;

    /// Fire one soft touchpad toggle (on Windows: the Ctrl+Win+F24 chord via
    /// the embedded kanata fake key). Used by the watchdog and the quit path.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError`] when the input engine control channel is
    /// unreachable.
    fn tap_toggle_chord(&self) -> Result<(), PlatformError>;

    /// Read whether the system currently considers the touchpad enabled.
    /// Unsupported platforms return [`PlatformError`] so the watchdog can log
    /// and disable itself.
    ///
    /// # Errors
    ///
    /// Returns [`PlatformError`] when the OS API is unavailable or the
    /// machine has no supported precision touchpad.
    fn touchpad_enabled(&self) -> Result<bool, PlatformError>;
}
