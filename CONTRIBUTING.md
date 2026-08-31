# Contributing

Thanks for contributing! This repository runs a layered check pipeline; please
make sure it stays green.

## Setup

```bash
just setup        # activate git hooks + install missing check tools
just check        # run the full chain (same as CI)
```

## Check gates

Details and the full gate tables: [docs/checks.md](docs/checks.md) (简体中文:
[docs/checks.zh.md](docs/checks.zh.md)).

- **pre-commit (fast)**: `cargo fmt --check`, `cargo machete`, docs↔code
  alignment (`githooks/check-docs`), strict clippy
- **pre-push (heavy)**: `cargo audit`, `cargo deny check`, `cargo outdated`,
  `cargo test`
- **CI**: the whole chain via `just check`

The full discipline — including when a lint waiver is acceptable — lives in
[AGENTS.md](AGENTS.md). In short: fix code first; waivers are code-level,
minimal-scope, and must carry a reason comment.

## Docs must stay in sync

Documentation is modular and bilingual (`docs/*.md` + `docs/*.zh.md`, plus the
two README landing pages). Changing lint config or the check chain requires
updating the affected pages **in both languages** in the same change.
`githooks/check-docs` enforces the greppable parts automatically.

## Pull requests

- Keep CI green; a failing `full check chain` job blocks merge.
- Small, focused PRs are easier to review.
