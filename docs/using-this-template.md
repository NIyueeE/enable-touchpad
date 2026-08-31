# Using this template

> English | [简体中文](using-this-template.zh.md)

You clicked **Use this template** — congrats, your new repo already has strict
lints, layered hooks, CI, and a release pipeline. Now rename things so the
project is *yours*. Every file that needs a change is listed below; grep for
the old name to be safe:

```bash
grep -rn "rust-agents-template" . --exclude-dir={.git,target}
```

## Rename checklist

| # | File | What to change |
|---|------|----------------|
| 1 | `Cargo.toml` | `name`, `description`, `repository`; reset `version` if you like |
| 2 | `Cargo.lock` | no manual edit — regenerate with `cargo build` (or delete it first) |
| 3 | `tests/cli.rs` | `env!("CARGO_BIN_EXE_rust-agents-template")` → your new binary name |
| 4 | `.github/workflows/release.yml` | `bin: rust-agents-template` → your new binary name |
| 5 | `justfile` | top comment (cosmetic) |
| 6 | `README.md` / `README.zh.md` | title, badge URLs, clone URL, intro text |
| 7 | `LICENSE` / `LICENSE-MIT` / `LICENSE-APACHE` | copyright holder and year |
| 8 | `SECURITY.md`, `CONTRIBUTING.md`, `AGENTS.md` | optional: adjust contact / wording |
| 9 | `src/main.rs` | crate-level doc comment (missing_docs is enforced) |

Files that need **no** change: `rust-toolchain.toml`, `deny.toml`,
`githooks/*`, `.editorconfig`, `docs/*` (relative links), `.github/dependabot.yml`.

## After renaming

```bash
just setup        # activate hooks + install tools
just check        # full chain — will catch anything you missed
git add -A && git commit -m "chore: rename project"   # pre-commit runs here
```

`just check` is your safety net: it re-verifies that the docs, hooks, and CI
still agree with the code you just renamed.

## Then develop as usual

- commits run the fast gates, pushes run the heavy gates (see
  [Checks](checks.md))
- when a gate blocks you: fix the code first — waivers are code-level and need
  a reason ([Lint policy](lint-policy.md))
- to ship binaries: push a `v*` tag ([Release](release.md))
