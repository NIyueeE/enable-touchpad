//! Minimal Win32 FFI surface for enable-touchpad: the precision-touchpad
//! state query and the mouse-layer cursor badge overlay.
#![cfg(windows)]
//!
//! This lives in its own tiny crate because the main application crate sets
//! the crate-level lint `unsafe_code = "forbid"`, which cannot be relaxed
//! per-module (rustc E0453). Only [`touchpad_enabled`] in this file contains
//! `unsafe`, and each use carries its own safety argument.
//!
//! Source of truth for the constant and the struct layout: the Windows SDK
//! header `WinUser.h`, which guards both behind
//! `NTDDI_VERSION >= NTDDI_WIN11_GE`. The API therefore does not exist on
//! Windows 10 or on machines without a precision touchpad; callers must treat
//! [`TouchpadStateError`] as "state unknown" and never guess.

/// `SystemParametersInfoW` action code `SPI_GETTOUCHPADPARAMETERS`.
///
/// Declared in `WinUser.h` as `0x00AE`; redeclared here because the
/// windows-rs metadata does not generate this constant.
const SPI_GETTOUCHPADPARAMETERS: u32 = 0x00AE;

/// Struct version requested from the OS (`TOUCHPAD_PARAMETERS_VERSION_1`).
const TOUCHPAD_PARAMETERS_VERSION_1: u32 = 1;

/// Bit 3 of the first bit-field word: `touchpadEnabled`.
const BIT_TOUCHPAD_ENABLED: u32 = 1 << 3;

/// Bit 0 of the first bit-field word: `touchpadPresent`.
const BIT_TOUCHPAD_PRESENT: u32 = 1 << 0;

/// Mirror of `TOUCHPAD_PARAMETERS_V1` from `WinUser.h`.
///
/// Eleven 4-byte fields: three scalars, two raw C bit-field words (MSVC
/// allocates bit-fields least-significant-bit first), and six trailing
/// scalars. The compile-time size assert below fails the build if the
/// layout ever drifts from the 44-byte SDK contract.
#[repr(C)]
#[derive(Clone, Copy)]
struct TouchpadParametersV1 {
    version_number: u32,
    max_supported_contacts: u32,
    legacy_touchpad_features: u32,
    /// Bit-field word 1: touchpadPresent, legacyTouchpadPresent,
    /// externalMousePresent, touchpadEnabled, touchpadActive,
    /// feedbackSupported, clickForceSupported, then 25 reserved bits.
    status_bits: u32,
    /// Bit-field word 2: ten user-setting flags, then 22 reserved bits.
    setting_bits: u32,
    sensitivity_level: u32,
    cursor_speed: u32,
    feedback_intensity: u32,
    click_force_sensitivity: u32,
    right_click_zone_width: u32,
    right_click_zone_height: u32,
}

const _: () = assert!(
    size_of::<TouchpadParametersV1>() == 44,
    "TOUCHPAD_PARAMETERS_V1 no longer matches the 44-byte WinUser.h layout"
);

/// Why the touchpad state could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchpadStateError {
    /// The OS rejected the query: pre-Windows 11, or no precision touchpad
    /// stack is present.
    SpiUnavailable,
    /// The machine reports no precision touchpad (`touchpadPresent` is 0),
    /// so "enabled" would be meaningless.
    NoTouchpad,
}

/// Mouse-layer cursor badge overlay (click-through layered window pinned to
/// the cursor). See the module docs for the design.
pub mod cursor_badge;

/// Direct Ctrl+Win+F24 chord injection (mirrors kanata's proven SendInput
/// output path; bypasses the unproven ActOnFakeKey TCP command).
pub mod chord;

// This crate is the designated unsafe boundary of enable-touchpad (the main
// crate forbids `unsafe_code` and cannot relax it locally); each use below is
// a deliberate, documented Win32 call.
#[allow(unsafe_code)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(
        action: u32,
        uiparam: u32,
        pvparam: *mut std::os::raw::c_void,
        fwinini: u32,
    ) -> i32;
}

/// Reads whether the precision touchpad is currently enabled.
///
/// This is the same OS state the Settings app and the system's
/// Ctrl+Win+F24 soft toggle act on — no device is touched.
#[allow(unsafe_code)]
pub fn touchpad_enabled() -> Result<bool, TouchpadStateError> {
    let mut params = TouchpadParametersV1 {
        version_number: TOUCHPAD_PARAMETERS_VERSION_1,
        max_supported_contacts: 0,
        legacy_touchpad_features: 0,
        status_bits: 0,
        setting_bits: 0,
        sensitivity_level: 0,
        cursor_speed: 0,
        feedback_intensity: 0,
        click_force_sensitivity: 0,
        right_click_zone_width: 0,
        right_click_zone_height: 0,
    };
    let size = u32::try_from(size_of::<TouchpadParametersV1>()).unwrap_or_default();
    // SAFETY: `params` is a fully initialised 44-byte struct and the API only
    // writes into it (the pointer is exclusively for the OUT value). On
    // failure the API leaves the struct untouched and returns 0.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETTOUCHPADPARAMETERS,
            size,
            std::ptr::from_mut(&mut params).cast::<std::os::raw::c_void>(),
            0,
        )
    };
    if ok == 0 {
        return Err(TouchpadStateError::SpiUnavailable);
    }
    if params.status_bits & BIT_TOUCHPAD_PRESENT == 0 {
        return Err(TouchpadStateError::NoTouchpad);
    }
    Ok(params.status_bits & BIT_TOUCHPAD_ENABLED != 0)
}
