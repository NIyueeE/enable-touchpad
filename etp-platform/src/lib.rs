//! Platform adaptation layer for enable-touchpad.
//!
//! This is the single layer where OS-specific integration lives. The
//! application crate programs against the [`Platform`] trait and never
//! touches `cfg(windows)` / Win32 / kanata directly; adding a new OS means
//! adding a new module behind `#[cfg(...)]` and returning it from
//! [`current`], without changing the application or the domain core.

mod error;
mod traits;

#[cfg(not(windows))]
mod fallback;
#[cfg(windows)]
mod windows;

pub use error::PlatformError;
pub use traits::Platform;

/// Returns the platform adapter for the current target.
///
/// The returned reference is `'static` and lives for the process lifetime, so
/// every thread (UI, tray, watchdog, engine) can share it.
#[must_use]
pub fn current() -> &'static dyn Platform {
    #[cfg(windows)]
    {
        &windows::WINDOWS_PLATFORM
    }
    #[cfg(not(windows))]
    {
        &fallback::FALLBACK_PLATFORM
    }
}
