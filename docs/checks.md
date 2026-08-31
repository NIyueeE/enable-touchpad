# Checks

> English | [简体中文](checks.zh.md)

Fast gates run before every commit, heavyweight gates before every push, and CI
runs the whole chain on every push / pull request via `just check`.

## Tools

The gates use four external tools; `just setup` installs any that are missing
(and activates the git hooks):

```bash
cargo install cargo-machete cargo-audit cargo-outdated cargo-deny --locked
```

`cargo fmt` and `cargo clippy` come with the toolchain declared in
`rust-toolchain.toml`.

## On every commit — `githooks/pre-commit`

| # | Gate | Command | Purpose |
|---|------|---------|---------|
| 1 | fmt | `cargo fmt --all -- --check` | code style |
| 2 | secrets | `githooks/check-secrets` | secret scan on staged changes |
| 3 | machete | `cargo machete` | unused dependencies |
| 4 | docs | `githooks/check-docs` | docs ↔ code alignment |
| 5 | clippy | `cargo clippy --all-targets --all-features -- -D warnings` | strict lints |

Lines that must carry a secret-shaped string (e.g. key-format documentation)
take a `security-scan:allow` marker with a reason; `check-secrets` skips them.

## On every push — `githooks/pre-push`

| # | Gate | Command | Purpose |
|---|------|---------|---------|
| 6 | audit | `cargo audit` | RustSec security advisories |
| 7 | deny | `cargo deny check` | licenses / bans / advisories policy |
| 8 | outdated | `cargo outdated --root-deps-only` | outdated direct dependencies |
| 9 | test | `cargo test --quiet` | test suite |

## One-shot run

```bash
just check   # identical to hooks + CI
```

## When a gate blocks you

Fix the code first. A waiver is the last resort: code-level only
(`#[expect(...)]` preferred over `#[allow]`), minimal scope, with a reason
comment. Never weaken `[lints]`, the hooks, or CI. See
[Lint policy](lint-policy.md) and [AGENTS.md](../AGENTS.md).
