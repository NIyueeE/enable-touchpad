//! Touchpad state watchdog.
//!
//! The system flips the precision touchpad on/off on our Ctrl+Win+F24 chord
//! taps; this module reads the official state back via `etp-ffi`
//! (`SPI_GETTOUCHPADPARAMETERS`) and corrects drift: whenever the layer key
//! is **not** held, the touchpad must be off. Correction reuses the same
//! soft toggle (the `release-tap` fake key through the embedded kanata), so
//! no device is ever disabled.
//!
//! Managed/unmanaged: when the master switch is off, the touchpad is left to
//! the system (expected state "unmanaged") and the watchdog does nothing.

use crate::etp_core::AppConfig;
use crate::kanata_embed;
use etp_ffi::TouchpadStateError;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Expected state: the feature is off; the touchpad belongs to the system.
const EXPECTED_UNMANAGED: u8 = 0;
/// Expected state: no layer key is held, the touchpad must be off.
const EXPECTED_OFF: u8 = 1;
/// Expected state: the layer key is held, the touchpad must be on.
const EXPECTED_ON: u8 = 2;

/// `etp-ffi` usability: 0 unknown, 1 works, 2 unusable (logged once).
static SPI_USABLE: AtomicU8 = AtomicU8::new(0);
/// Expected touchpad state; see the `EXPECTED_*` constants.
static EXPECTED: AtomicU8 = AtomicU8::new(EXPECTED_UNMANAGED);
/// Millis since the last chord tap (ours, or the press/release taps kanata
/// sent on a layer change) — corrections back off around them to avoid
/// racing the state flip they cause.
static LAST_TAP_MS: AtomicU64 = AtomicU64::new(0);
/// Consecutive corrections that did not turn the touchpad off.
static CONSECUTIVE_FAILURES: AtomicU8 = AtomicU8::new(0);

/// How often the watchdog samples the touchpad state.
const WATCHDOG_INTERVAL: Duration = Duration::from_millis(1200);
/// Minimum time after any chord tap before a correction may fire.
const TAP_COOLDOWN_MS: u64 = 1500;
/// After this many consecutive corrections without effect, pause a minute —
/// tapping every cycle would only spam the log and the keyboard buffer.
const FAILURE_BACKOFF_THRESHOLD: u8 = 3;
const FAILURE_BACKOFF: Duration = Duration::from_secs(60);

/// Record that a Ctrl+Win+F24 chord just flew (the state flip lags a bit).
pub fn mark_tap_now() {
    LAST_TAP_MS.store(now_ms(), Ordering::Relaxed);
}

/// The embedded kanata reported a layer change: holding the layer key means
/// the touchpad should be on, releasing it means off.
pub fn set_expected(on: bool) {
    EXPECTED.store(
        if on { EXPECTED_ON } else { EXPECTED_OFF },
        Ordering::Relaxed,
    );
}

/// Master switch moved (startup or settings apply): unmanaged means hands off.
pub fn set_managed(managed: bool) {
    if managed {
        EXPECTED.store(EXPECTED_OFF, Ordering::Relaxed);
    } else {
        EXPECTED.store(EXPECTED_UNMANAGED, Ordering::Relaxed);
    }
}

/// Whether the watchdog currently believes the touchpad must be on (the tray
/// quit path uses this to leave the system in the off state).
pub fn expected_on() -> bool {
    EXPECTED.load(Ordering::Relaxed) == EXPECTED_ON
}

/// Spawn the watchdog thread (call once at startup).
pub fn spawn_watchdog() {
    std::thread::spawn(|| {
        // The master switch decides whether we manage the touchpad at all;
        // settings applies update this through [`set_managed`].
        set_managed(AppConfig::load().feature_enabled);
        log::info!("touchpad watchdog started");
        loop {
            std::thread::sleep(WATCHDOG_INTERVAL);
            tick();
        }
    });
}

/// One watchdog cycle: enforce "off while idle" with at most one soft toggle.
fn tick() {
    if EXPECTED.load(Ordering::Relaxed) != EXPECTED_OFF {
        return;
    }
    if now_ms().saturating_sub(LAST_TAP_MS.load(Ordering::Relaxed)) < TAP_COOLDOWN_MS {
        return;
    }
    match etp_ffi::touchpad_enabled() {
        Err(e) => report_unusable(e),
        Ok(false) => CONSECUTIVE_FAILURES.store(0, Ordering::Relaxed),
        Ok(true) => correct(),
    }
}

/// Drift detected: the touchpad is on while it must be off. Send one soft
/// toggle and log it.
fn correct() {
    log::info!("state correction: touchpad on while idle, sending soft toggle");
    mark_tap_now();
    if let Err(e) = kanata_embed::tap_release_fakekey() {
        log::error!("state correction failed to reach kanata: {e}");
        return;
    }
    let failures = CONSECUTIVE_FAILURES.fetch_add(1, Ordering::Relaxed) + 1;
    if failures >= FAILURE_BACKOFF_THRESHOLD {
        log::warn!(
            "correction did not stick after {failures} attempts; pausing {FAILURE_BACKOFF:?} before retrying",
        );
        std::thread::sleep(FAILURE_BACKOFF);
        CONSECUTIVE_FAILURES.store(0, Ordering::Relaxed);
    }
}

/// Log SPI unusability once (or again after it came back).
fn report_unusable(e: TouchpadStateError) {
    let previous = SPI_USABLE.swap(2, Ordering::Relaxed);
    if previous != 2 {
        log::warn!(
            "touchpad state query unavailable ({e:?}); state correction disabled \
             (needs Windows 11 and a precision touchpad)"
        );
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or_default())
}
