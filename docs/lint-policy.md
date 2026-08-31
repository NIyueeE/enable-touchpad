# Lint policy

> English | [简体中文](lint-policy.zh.md)

Declared in the `[lints]` table of `Cargo.toml`:

| Lint | Level |
|------|-------|
| `unsafe_code` | forbid |
| `missing_docs` | warn |
| clippy `all` | deny |
| clippy `pedantic` | deny |
| clippy `unwrap_used` / `expect_used` / `dbg_macro` | deny |
| clippy `todo` | warn |

The pre-commit hook additionally passes `-D warnings`, so every warning above —
including `missing_docs` and `todo` — becomes a hard error at commit time.

## Waivers

**Fix code first; a waiver is the last resort, and only code-level.**

- Never "make errors disappear" by editing `[lints]`, the hooks, or CI.
- Allowed relaxation: `#[expect(clippy::lint_name)]` (preferred — it starts
  warning once the lint stops firing) or `#[allow(clippy::lint_name)]`, at
  statement or single-function scope, with a one-line reason comment.
- Never module-level `#![allow(...)]` or crate-level relaxation.
- Legitimate scenarios only: (1) genuinely unavoidable usage, (2) upstream
  false positives (macros, generated code, dependency audit noise).

The same discipline applies to every other gate (machete, audit, deny,
outdated, docs-sync, and anything added later). Full rules: [AGENTS.md](../AGENTS.md).
