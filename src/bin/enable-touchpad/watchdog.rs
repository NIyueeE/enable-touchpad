//! Touchpad state watchdog (application layer).
//!
//! The platform layer knows *how* to read and toggle the touchpad; this
//! module knows *when*: the system flips the precision touchpad on/off on our
//! Ctrl+Win+F24 chord taps, so the watchdog reads the official state back
//! through [`Platform::touchpad_enabled`] and corrects drift — whenever the
//! layer key is **not** held, the touchpad must be off. Correction reuses the
//! same soft toggle ([`Platform::tap_toggle_chord`]), so no device is ever
//! disabled.
//!
//! Managed/unmanaged: when the master switch is off, the touchpad is left to
//! the system (expected state "unmanaged") and the watchdog does nothing.

use etp_platform::{Platform, PlatformError};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Expected state: the feature is off; the touchpad belongs to the system.
const EXPECTED_UNMANAGED: u8 = 0;
/// Expected state: no layer key is held, the touchpad must be off.
const EXPECTED_OFF: u8 = 1;
/// Expected state: the layer key is held, the touchpad must be on.
const EXPECTED_ON: u8 = 2;

/// `Platform::touchpad_enabled` usability: 0 unknown, 1 works, 2 unusable
/// (logged once).
const SPI_USABLE: u8 = 0;
/// The query works on this machine.
const SPI_WORKS: u8 = 1;
/// The query is unavailable on this machine (logged once).
const SPI_UNAVAILABLE: u8 = 2;

/// How often the watchdog samples the touchpad state.
const WATCHDOG_INTERVAL: Duration = Duration::from_millis(1200);
/// A tap flips the touchpad asynchronously; querying too soon after one can
/// read the pre-flip state and cause a double tap, so state checks back off
/// for this long after any tap (ours or a transition's).
const SETTLE_MS: u64 = 400;
/// After this many consecutive corrections without effect, pause retries —
/// tapping every cycle would only spam the log and the keyboard buffer.
/// The pause is a **timestamp**, never a sleep: the watchdog thread must
/// keep draining layer events (badge, transitions) while retries are off.
const FAILURE_BACKOFF_THRESHOLD: u8 = 3;
const FAILURE_BACKOFF_MS: u64 = 60_000;

/// Shared watchdog state, updated by the UI thread (master switch), the
/// layer-change channel (expected on/off), and the watchdog thread itself.
pub struct WatchdogState {
    platform: &'static dyn Platform,
    /// Expected touchpad state; see the `EXPECTED_*` constants.
    expected: AtomicU8,
    /// Millis since the last chord tap (ours, or the press/release taps kanata
    /// sent on a layer change) — corrections back off around them to avoid
    /// racing the state flip they cause.
    last_tap_ms: AtomicU64,
    /// Consecutive corrections that did not turn the touchpad off.
    consecutive_failures: AtomicU8,
    /// `Platform::touchpad_enabled` usability; see the `SPI_*` constants.
    query_usability: AtomicU8,
    /// Epoch-ms until which idle correction retries are paused (a timestamp,
    /// so correcting never sleeps the watchdog thread).
    backoff_until_ms: AtomicU64,
}

impl WatchdogState {
    /// Create the shared state for the process-wide [`Platform`] adapter.
    #[must_use]
    pub fn new(platform: &'static dyn Platform) -> Self {
        Self {
            platform,
            expected: AtomicU8::new(EXPECTED_UNMANAGED),
            last_tap_ms: AtomicU64::new(0),
            consecutive_failures: AtomicU8::new(0),
            query_usability: AtomicU8::new(SPI_USABLE),
            backoff_until_ms: AtomicU64::new(0),
        }
    }

    /// Record that a Ctrl+Win+F24 chord just flew (the state flip lags a bit).
    pub fn mark_tap_now(&self) {
        self.last_tap_ms.store(now_ms(), Ordering::Relaxed);
    }

    /// The platform engine reported a layer change: holding the layer key
    /// means the touchpad should be on, releasing it means off. The cursor
    /// badge mirrors the same signal.
    ///
    /// The desired state is applied **deterministically**: the official
    /// touchpad state is queried and the chord is tapped only on a mismatch.
    /// (The chord is a toggle — blind-firing it used to invert the state
    /// whenever the touchpad was already on.) Machines without the state
    /// query keep the legacy blind tap per transition.
    pub fn set_expected(&self, on: bool) {
        self.expected.store(
            if on { EXPECTED_ON } else { EXPECTED_OFF },
            Ordering::Relaxed,
        );
        crate::cursor_badge::set_visible(on);
        self.verify_now(on);
    }

    /// One immediate desired-state check for a layer transition. If a very
    /// recent tap may still be settling (stale read risk), skip — the
    /// periodic tick re-checks within ~1.2 s.
    fn verify_now(&self, desired: bool) {
        if now_ms().saturating_sub(self.last_tap_ms.load(Ordering::Relaxed)) < SETTLE_MS {
            return;
        }
        match self.platform.touchpad_enabled() {
            Ok(actual) => {
                self.query_usability.store(SPI_WORKS, Ordering::Relaxed);
                if actual == desired {
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                } else {
                    self.correct();
                }
            }
            Err(e) => {
                // No state query (pre-Win11 / no precision touchpad): keep
                // the legacy blind toggle per transition.
                self.report_unusable(&e);
                let _ = self.platform.tap_toggle_chord();
            }
        }
        self.mark_tap_now();
    }

    /// Master switch moved (startup or settings apply): unmanaged means hands
    /// off. Any transition also hides the cursor badge.
    ///
    /// Turning the feature off **while the layer was held** is a special case:
    /// the engine reload removes the layer entirely, so the physical key
    /// release will never produce a layer-exit event (and its release chord)
    /// again — the touchpad would stay on forever with nobody correcting it.
    /// If the system still reports the touchpad enabled, fire one soft toggle
    /// before going unmanaged. The query guard keeps a blind tap from
    /// *switching the touchpad on* when it is already off.
    pub fn set_managed(&self, managed: bool) {
        // The master switch's single write point: UI, tray, and quit paths
        // all read this.
        crate::MASTER_SWITCH.store(managed, Ordering::Relaxed);
        // A master-switch transition always hides the badge: the layer is
        // either gone (feature off) or about to be idle-off.
        crate::cursor_badge::set_visible(false);
        if managed {
            self.expected.store(EXPECTED_OFF, Ordering::Relaxed);
            return;
        }
        let was_on = self.expected.swap(EXPECTED_UNMANAGED, Ordering::Relaxed) == EXPECTED_ON;
        if was_on && matches!(self.platform.touchpad_enabled(), Ok(true)) {
            log::info!("feature disabled while the layer was held; sending one release tap");
            self.mark_tap_now();
            if let Err(e) = self.platform.tap_toggle_chord() {
                log::error!("release tap after unmanaging failed: {e}");
            }
        }
    }

    /// Whether the watchdog currently believes the touchpad must be on (the
    /// tray quit path uses this to leave the system in the off state).
    #[must_use]
    pub fn expected_on(&self) -> bool {
        self.expected.load(Ordering::Relaxed) == EXPECTED_ON
    }

    /// Run the watchdog loop on the current thread: process layer-change
    /// events as they arrive and sample the touchpad state once per interval.
    pub fn run(self: &Arc<Self>, layer_events: &Receiver<bool>) {
        loop {
            match layer_events.recv_timeout(WATCHDOG_INTERVAL) {
                Ok(on) => {
                    self.set_expected(on);
                }
                Err(RecvTimeoutError::Timeout) => self.tick(),
                Err(RecvTimeoutError::Disconnected) => {
                    // The engine died without layer events; keep enforcing
                    // "off while idle" with the platform query alone. Note
                    // that `recv_timeout` returns `Disconnected` immediately
                    // without waiting once the sender is gone — sleep here so
                    // this branch cannot degenerate into a busy spin loop.
                    std::thread::sleep(WATCHDOG_INTERVAL);
                    self.tick();
                }
            }
        }
    }

    /// One watchdog cycle: enforce the expected state (off while idle, on
    /// while held) with at most one soft toggle.
    fn tick(&self) {
        let expected = self.expected.load(Ordering::Relaxed);
        if expected == EXPECTED_UNMANAGED {
            return;
        }
        if now_ms() < self.backoff_until_ms.load(Ordering::Relaxed) {
            return;
        }
        if now_ms().saturating_sub(self.last_tap_ms.load(Ordering::Relaxed)) < SETTLE_MS {
            return;
        }
        match self.platform.touchpad_enabled() {
            Err(e) => self.report_unusable(&e),
            Ok(actual) => {
                self.query_usability.store(SPI_WORKS, Ordering::Relaxed);
                let desired = expected == EXPECTED_ON;
                if actual == desired {
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                } else {
                    self.correct();
                }
            }
        }
    }

    /// State mismatch: the touchpad is not in the expected state. Send one
    /// soft toggle and log it.
    fn correct(&self) {
        let desired = self.expected.load(Ordering::Relaxed) == EXPECTED_ON;
        log::info!("touchpad state mismatch (expected on: {desired}); sending soft toggle");
        self.mark_tap_now();
        if let Err(e) = self.platform.tap_toggle_chord() {
            log::error!("state correction failed to reach kanata: {e}");
            return;
        }
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= FAILURE_BACKOFF_THRESHOLD {
            // Timestamp only — a sleeping watchdog would freeze layer-event
            // processing (badge stuck, transitions dead) for a whole minute.
            self.backoff_until_ms
                .store(now_ms() + FAILURE_BACKOFF_MS, Ordering::Relaxed);
            self.consecutive_failures.store(0, Ordering::Relaxed);
            log::warn!(
                "correction did not stick after {failures} attempts; pausing idle retries for 60s \
                 (layer transitions and the badge keep working)"
            );
        }
    }

    /// Log query unusability once (or again after it came back).
    fn report_unusable(&self, e: &PlatformError) {
        let previous = self
            .query_usability
            .swap(SPI_UNAVAILABLE, Ordering::Relaxed);
        if previous != SPI_UNAVAILABLE {
            log::warn!(
                "touchpad state query unavailable ({e}); state correction disabled \
                 (needs Windows 11 and a precision touchpad)"
            );
        }
    }
}

/// Spawn the watchdog thread (call once at startup).
pub fn spawn(state: Arc<WatchdogState>, layer_events: Receiver<bool>) {
    std::thread::spawn(move || state.run(&layer_events));
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or_default())
}
