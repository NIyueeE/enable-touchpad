# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Windows feasibility demo (`demo/`, demo branch for now): a Dioxus desktop
  app (`src/bin/enable-touchpad`) with system tray, minimal settings page and
  a mouse-following layer indicator, plus a `kanata` CapsLock layer config
  that enables the touchpad while CapsLock is held and restores on release.
  See `demo/README.md`; the app compiles for `cfg(windows)` only (other
  targets build a stub).

## [0.1.0] - 2026-08-31

### Added

- Initial release of the **rust-agents-template**: an agent-facing Rust
  project template with strict lints, layered git hooks,
  changelog-driven releases, and CI/CD test builds.
