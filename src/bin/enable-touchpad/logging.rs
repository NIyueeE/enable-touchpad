//! Logging setup.
//!
//! Writes detailed logs to `enable-touchpad.log`, preferring the platform
//! data directory (next to `config.json`) and falling back to the
//! executable's directory if that is unavailable — some launch contexts
//! (e.g. running elevated as a different account) resolve `%APPDATA%` to a
//! different profile, which used to make the log appear to vanish. The
//! active file path is published via [`log_file`] so the UI can show where
//! logs really live.

use etp_platform::Platform;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Full path of the active log file; set by [`init`] on success.
static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

/// The active log file path, when logging could be initialised.
#[must_use]
pub fn log_file() -> Option<&'static PathBuf> {
    LOG_FILE.get()
}

/// Configure file logging. Failing to open the log file is not fatal: the
/// app still runs, just without a log.
pub fn init(platform: &dyn Platform) {
    let dir = platform.app_data_dir().ok().or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(PathBuf::from))
    });
    let Some(dir) = dir else {
        return;
    };
    let path = dir.join("enable-touchpad.log");
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        file,
    );
    let _ = LOG_FILE.set(path);
}
