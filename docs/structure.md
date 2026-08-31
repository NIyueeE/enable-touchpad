# Project structure

> English | [简体中文](structure.zh.md)

| Path | Purpose |
|------|---------|
| `src/main.rs` | binary entry point |
| `src/bin/enable-touchpad/` | Windows demo binary: tray + settings UI + touchpad toggle (stub on other targets) |
| `demo/` | Windows feasibility demo assets: kanata layer config, bilingual README |
| `tests/cli.rs` | smoke test for the binary |
| `Cargo.toml` | manifest: strict `[lints]`, package metadata |
| `Cargo.lock` | committed (binary template convention) |
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
