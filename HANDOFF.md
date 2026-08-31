# HANDOFF.md — working state for agents

Agent-facing handoff notes. AGENTS.md owns the **rules**; this file owns the
**state**: where the project stands, which decisions were made and why, and
what is still open. Update this file (in the same commit) whenever you change
something the next session needs to know.

## Quick orientation

- What this repo is: `rust-agents-template` — a public GitHub template repo
  for Rust binary projects with strict lints, layered git hooks,
  changelog-driven releases, and agent-facing rules.
  Live: https://github.com/NIyueeE/rust-agents-template
- Read order: AGENTS.md (rules) → this file (state) → docs/ (topic pages).
- Local loop: `just setup` (once) → `just check` (full chain) → commit/push.
- Tests: `just test` (cargo-nextest when installed, otherwise `cargo test`).

## Current state (2026-08-31)

- **Template finalized** at user request: release history and tags were reset,
  and a clean baseline release `v0.1.0` was re-created to validate the whole
  CI/CD flow after finalization.
- Toolchain: floating stable (1.98.0 at the time of writing).
- Gates: pre-commit fast gates (incl. secret scan) / pre-push heavy gates /
  CI identical; `githooks/check-docs` carries the docs↔code invariants
  (AGENTS.md §3).
- Releases: tag-driven, notes extracted from CHANGELOG.md; the baseline
  release is `v0.1.0` (placeholder notes, three platform assets).
- CD: `test-build.yml` verified on all three platforms.
- Everything currently green: CI on main, pre-commit, pre-push.

## Decision log (why things are the way they are)

- Bilingual docs (`*.md` + `*.zh.md`) — user preference; mechanically enforced
  by `check-docs` where greppable.
- clippy `pedantic` at `deny` plus `-D warnings` in the hook — the user chose
  the strict variant over a pedantic-warn baseline.
- cargo-deny runs **alongside** cargo audit on purpose (both were requested);
  do not deduplicate them without asking.
- Release notes come only from CHANGELOG.md; a missing section fails the
  release by design.
- Commits are free, `v*` tags are deliberate (AGENTS.md §5) — agents never
  tag without an explicit human request.
- Pre-commit secret scan (`githooks/check-secrets`) added at user request;
  waiver marker `security-scan:allow` with a reason; placeholder-looking
  values (example/dummy/{{ }}) are filtered to keep false positives low.
- The README roadmap section was removed at user request; open items live in
  this file.

## Open threads

- MSRV (`rust-version`): deliberately undecided — it tensions with the
  floating stable toolchain; ask before adding one.
- Action pins: SHAs with `# vX` comments; Dependabot tracks those comments —
  merge its bumps to stay current.
- cargo-nextest evaluation (2026-08-31): works, but the suite is a single
  smoke test, so there is nothing to speed up yet. Adopted as optional via
  `just test`; pre-push and CI still run `cargo test`. Revisit when the suite
  grows past ~10 tests.
- Consider tag-protection rulesets (restrict `v*` creation) if the repo gains
  collaborators.
- Only the linux leg of `test-build.yml` has been exercised (see current
  state).

## Gotchas

- `actions/checkout` etc. are SHA-pinned with `# vX` comments; bump via
  Dependabot PRs, or update the SHA and the comment together.
- Renaming anything? Follow docs/using-this-template.md and re-grep for the
  old name afterwards. Note: renaming the package can leave a stale
  `target/` cache that makes `cargo test` fail with
  `Os { code: 2, NotFound }` (the cached test binary still points at the old
  bin path). `cargo clean -p <name>` — or touching the test — fixes it
  locally; fresh checkouts and CI are unaffected.
- Private vulnerability reporting must stay enabled in repo settings.
- Never use `--no-verify` (AGENTS.md §4) and never bypass a gate (§2).
