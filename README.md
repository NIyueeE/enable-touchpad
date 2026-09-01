# enable-touchpad

> Hold a layer key to reclaim your touchpad: the touchpad comes back while the
> layer is held and soft-disables again afterwards.

[![CI](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml/badge.svg)](https://github.com/NIyueeE/enable-touchpad/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

[English](README.md) | [简体中文](README.zh.md)

> **Platform: Windows 11 (adapted).** A precision touchpad is required; other
> platforms are planned behind the platform-adaptation layer.

A small Windows tray application that embeds the kanata keyboard engine as a
library. While the configurable layer key (CapsLock by default) is held, a
`mouse` layer activates and the system performs the touchpad soft on/off; the
app never disables devices itself.

## Features

- **Windows 11 adapted** — the precision-touchpad state query and the soft
  toggle path were verified on Windows 11 (see [demo/README.md](demo/README.md)).
- **Embedded kanata engine** — no kernel driver, no separate process; kanata
  v1.11 runs inside the single executable (LL-hook capture + SendInput output).
- **Captured bindings, not dropdowns** — click a row, press any supported key:
  letters, digits, F-keys, modifiers, numpad, and more. `Escape` cancels and
  `CapsLock` stays the fixed layer hold key.
- **State watchdog** — while the layer key is not held, the official
  precision-touchpad state is sampled and drift is corrected with the same soft
  chord; with the master switch off, the touchpad belongs to the system again.
- **Layered architecture** — `etp-core` (domain), `etp-platform` (the single
  platform-adaptation layer), `etp-ffi` (Windows-only FFI leaf), and the
  application binary written against the `Platform` trait.
- **Strict check pipeline** — `rust-toolchain.toml` declares
  `channel = "stable"` with `clippy` and `rustfmt` bundled, `unsafe_code = "forbid"`,
  clippy `all` + `pedantic` at `deny` (see [Lint policy](docs/lint-policy.md)),
  fast gates before every commit, heavyweight gates before every push, and CI
  enforcing the same chain (see [Checks](docs/checks.md)).
- **One-tag releases** — multi-platform binaries built on `v*` tags
  (see [Release](docs/release.md)).
- **Rust 2024 edition**.

## Quick start

```bash
git clone https://github.com/NIyueeE/enable-touchpad.git
cd enable-touchpad

# development: one-time setup per clone — activate hooks + install missing tools
just setup   # (or manually: git config core.hooksPath githooks)

cargo run    # Linux/macOS print the stub; run the real app on Windows

# run the full check chain any time — identical to hooks + CI
just check
```

On Windows 11, build or download the exe and run it as administrator: hold the
layer key to enable the touchpad, release to soft-disable it again. Right-click
the tray icon to open the settings window; saving regenerates the kanata
config and hot-applies it. Details and limitations live in
[demo/README.md](demo/README.md).

## Documentation

| Document | Content |
|----------|---------|
| [demo/README.md](demo/README.md) | Windows 11 usage, setup, and demo limitations |
| [docs/using-this-template.md](docs/using-this-template.md) | renaming a fork of this project: the rename checklist |
| [docs/checks.md](docs/checks.md) | the eight gates, layered hooks, CI |
| [docs/lint-policy.md](docs/lint-policy.md) | every lint and its level, waiver rules |
| [docs/release.md](docs/release.md) | tagging → multi-platform binaries |
| [docs/structure.md](docs/structure.md) | what every file in this repo is for |
| [HANDOFF.md](HANDOFF.md) | agent handoff: current state, decisions, open threads |
| [CONTRIBUTING.md](CONTRIBUTING.md) | how to contribute |
| [SECURITY.md](SECURITY.md) | reporting vulnerabilities |
| [AGENTS.md](AGENTS.md) | rules for AI coding agents (and humans) |

Each document has a `*.zh.md` 简体中文 counterpart.

## Contributing

PRs welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Distributed under the MIT OR Apache-2.0 license. See [`LICENSE`](LICENSE) for details.

© 2026 NIyueeE (100502009+NIyueeE@users.noreply.github.com)
