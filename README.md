# enable-touchpad

> Hold one key to bring your touchpad back for a moment. Release it, and the
> touchpad soft-disables again.

[![CI](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | [简体中文](README.zh.md)

`enable-touchpad` is a small Windows tray utility for people who keep the
touchpad disabled most of the time, but still want it back for a few seconds
when they actually need it.

Press and hold the layer key (CapsLock by default): the touchpad becomes
usable. Release the key: the touchpad goes back to its disabled state. There
is no permanent mode to remember and nothing to uninstall — it behaves like a
momentary switch for the touchpad.

> **Platform: Windows 11 (adapted).** A precision touchpad is required; other
> platforms are planned.

## Why this exists

Some users disable the touchpad to avoid accidental cursor jumps while typing
or using a mouse. The annoyance is that re-enabling it normally means digging
through Windows settings every time.

This tool turns that into a single held key:

- hold the key → touchpad on
- release the key → touchpad off again
- close the tray app → the touchpad belongs to the system again

## How it works

The app watches a configured layer key. While that key is held, it activates a
mouse layer; when released, it sends the soft-disable action again. It also
watches the official precision-touchpad state and corrects drift while the
key is not held.

It does **not** disable or uninstall any device. The touchpad is only
soft-toggled while the tool is running.

## Usage

1. Download or build the Windows executable.
2. Run it **as administrator**.
3. Look for the tray icon.
4. Hold the configured layer key (default: `CapsLock`) to enable the touchpad.
5. Release it to soft-disable the touchpad again.
6. Right-click the tray icon → `打开设置` to change bindings or turn the
   master switch off.

Settings are hot-applied; you do not need to restart the app after saving.

## Configuration

- Bind your own keys for left / middle / right mouse actions and the
  CapsLock-layer action.
- Keys are captured directly from the keyboard instead of picked from a
  dropdown.
- Turn the master switch off to leave the touchpad entirely to Windows.
- Config lives under `%APPDATA%\enable-touchpad\`.
- Logs also live under `%APPDATA%\enable-touchpad\`.

## Requirements and limits

- Windows 11 and a precision touchpad.
- The tool relies on the system/driver behavior bound to its soft toggle.
- If the soft toggle is not available on your machine, the app logs the
  condition and does not pretend to work.
- Other platforms are not supported yet.

## Project notes

Built with the Rust 2024 edition on the stable toolchain
(`rust-toolchain.toml` declares `channel = "stable"`). For local development,
hooks are enabled with `git config core.hooksPath githooks` (or `just setup`),
and the full chain runs with `just check`. Project docs:
`docs/using-this-template.md`, `docs/checks.md`, `docs/lint-policy.md`,
`docs/release.md`, `docs/structure.md`, and `HANDOFF.md`.

## License

Distributed under the MIT OR Apache-2.0 license. See [`LICENSE`](LICENSE) for details.

© 2026 NIyueeE (100502009+NIyueeE@users.noreply.github.com)
