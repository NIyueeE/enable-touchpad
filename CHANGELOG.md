# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Windows feasibility demo (merged to `main`): a single-exe
  Dioxus desktop app with a tray icon and a small settings window (hidden by
  default) for the mouse-layer key bindings; **kanata v1.11 is embedded as a
  library** (LL-hook keyboard capture, no kernel driver, no separate
  process). Holding CapsLock activates the configurable layer and taps
  Ctrl+Win+F24 on press and on release — the touchpad soft on/off is owned
  by the operating system, the app never disables devices. Config changes
  hot-apply over kanata's TCP `Reload` command; detailed logs go to
  `%APPDATA%\enable-touchpad\enable-touchpad.log`. Compiles for
  `cfg(windows)` only (other targets build a stub).
  **Windows 11 is adapted**: verified on a real Windows 11 device with a
  precision touchpad.
- Layer key bindings are **captured from the keyboard** instead of a fixed
  dropdown: click a row, press any supported key (W3C `KeyboardEvent.code`
  names pass straight into the generated kanata config; `Escape` cancels,
  `CapsLock` remains the fixed layer hold key, duplicates collapse to the
  first claim).
- Touchpad **state watchdog**: while the layer key is not held, the
  official precision-touchpad state (`SPI_GETTOUCHPADPARAMETERS`, Win11+,
  wrapped in the new `etp-ffi` FFI crate) is sampled every ~1.2 s and any
  drift back to "enabled" is corrected with the same soft Ctrl+Win+F24
  chord via kanata's fake-key channel — devices are never touched. With
  the master switch off, the touchpad is left to the system.

### Changed

- Layered architecture for multi-platform porting: `etp-core` (pure
  cross-platform domain logic, unit-tested on every host), `etp-platform`
  (the single platform-adaptation layer: `Platform` trait + Windows adapter
  and non-Windows fallback), and the application binary (UI, tray, watchdog)
  programmed against the platform trait. The former `etp_core` /
  `kanata_embed` / `touchpad_state` binary modules were dissolved into these
  crates. The settings window grew to 440×400 to fit the capture hints and
  footer.
- Project identity: the root package is now **enable-touchpad**; the
  template hello-world binary, its smoke test, and the remaining template
  placeholders were removed or updated.

## [0.1.0] - 2026-08-31

### Added

- Initial release of the **rust-agents-template**: an agent-facing Rust
  project template with strict lints, layered git hooks,
  changelog-driven releases, and CI/CD test builds.
