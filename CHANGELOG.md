# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Windows feasibility demo (`demo/`, demo branch for now): a single-exe
  Dioxus desktop app with a tray icon and a small settings window (hidden by
  default) for the mouse-layer key bindings; **kanata v1.11 is embedded as a
  library** (LL-hook keyboard capture, no kernel driver, no separate
  process). Holding CapsLock activates the configurable layer and taps
  Ctrl+Win+F24 on press and on release — the touchpad soft on/off is owned
  by the operating system, the app never disables devices. Config changes
  hot-apply over kanata's TCP `Reload` command; detailed logs go to
  `%APPDATA%\enable-touchpad\enable-touchpad.log`. Compiles for
  `cfg(windows)` only (other targets build a stub).

## [0.1.0] - 2026-08-31

### Added

- Initial release of the **rust-agents-template**: an agent-facing Rust
  project template with strict lints, layered git hooks,
  changelog-driven releases, and CI/CD test builds.
