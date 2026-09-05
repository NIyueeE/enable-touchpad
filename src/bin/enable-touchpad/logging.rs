//! Logging setup.
//!
//! Writes detailed logs to `enable-touchpad.log` in the platform data
//! directory. kanata logs through the `log` crate too, so its output lands in
//! the same file.

use etp_platform::Platform;

/// Configure file logging. Failing to open the log file is not fatal: the
/// app still runs, just without a log.
pub fn init(platform: &dyn Platform) {
    let Ok(dir) = platform.app_data_dir() else {
        return;
    };
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("enable-touchpad.log"))
    else {
        return;
    };
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        file,
    );
}
