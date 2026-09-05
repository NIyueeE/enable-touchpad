# Project structure

> English | [简体中文](structure.zh.md)

| Path | Purpose |
|------|---------|
| `src/bin/enable-touchpad/` | application binary (application layer): tray + settings UI + config store + watchdog, written against the platform trait |
| `build.rs` | embeds `assets/icon.ico` as the exe's icon resource (skipped with a warning where no Win32 resource compiler exists) |
| `assets/` | icon assets: multi-size `icon.ico` (exe + tray resource), `icon_32.png` (window icon), `src/` original artwork, `make_icons.py` regeneration script |
| `etp-core/` | cross-platform domain layer: config model, key allowlist, kanata config generator (unit-tested on Linux) |
| `etp-platform/` | single platform-adaptation layer: `Platform` trait + Windows adapter (embedded kanata engine, touchpad state) and non-Windows fallback |
| `etp-ffi/` | Windows-only FFI leaf crate for the precision-touchpad state query (isolates `unsafe`) |
| `demo/` | Windows feasibility demo notes: bilingual README |
| `Cargo.toml` | manifest: strict `[lints]`, package metadata |
| `Cargo.lock` | committed lockfile for reproducible binary builds |
| `rust-toolchain.toml` | `channel = "stable"` + clippy/rustfmt components |
| `justfile` | `just setup` (hooks + tools) / `just check` (full chain) |
| `deny.toml` | cargo-deny policy: licenses / bans / advisories / sources |
| `githooks/pre-commit` | fast gates: fmt, secrets, machete, docs, clippy |
| `githooks/pre-push` | heavy gates: audit, deny, outdated, test |
| `githooks/check-docs` | docs ↔ code alignment gate |
| `githooks/check-secrets` | secret scan on staged changes |
| `.github/workflows/ci.yml` | CI: `just check` on push / PR |
| `.github/workflows/release.yml` | tag push (`v*`) → multi-platform binaries |
| `.github/workflows/test-build.yml` | manual test builds for chosen platforms at any commit |
| `.github/dependabot.yml` | weekly actions + cargo dependency updates |
| `AGENTS.md` | rules for AI coding agents (and humans) |
| `CONTRIBUTING.md` | how to contribute |
| `SECURITY.md` | vulnerability reporting policy |
| `LICENSE` (+ `LICENSE-MIT` / `LICENSE-APACHE`) | MIT OR Apache-2.0 |
| `.editorconfig` | cross-editor formatting basics |
| `docs/` | modular documentation (this directory) |

See also: [Checks](checks.md) · [Lint policy](lint-policy.md) · [Release](release.md)
