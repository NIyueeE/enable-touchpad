//! Error type shared by every platform adapter.

use std::fmt;

/// A platform operation failed (path lookup, kanata startup, TCP control
/// channel, touchpad state query, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformError {
    message: String,
}

impl PlatformError {
    /// Wrap a human-readable message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PlatformError {}

impl From<String> for PlatformError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<std::io::Error> for PlatformError {
    fn from(e: std::io::Error) -> Self {
        Self::new(e.to_string())
    }
}
