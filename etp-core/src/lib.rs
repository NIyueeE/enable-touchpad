//! Cross-platform domain logic for enable-touchpad.
//!
//! This crate is intentionally free of OS and UI dependencies: it owns the
//! configuration model, the bindable-key allowlist, display labels, and the
//! kanata configuration generator. Every target compiles and tests it, so the
//! real logic is verified on the Linux host even while the application shell
//! is Windows-only.

pub mod config;
pub mod generator;
pub mod keys;

pub use config::AppConfig;
pub use generator::generate_config_text;
pub use keys::{
    CANCEL_KEY, HOLD_KEY, KEY_NONE, SUPPORTED_CODES, code_to_vk, is_bindable, key_label,
};
