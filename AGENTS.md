# AGENTS.md — Repository Rules

This file governs AI coding agents (and, equally, human contributors) working
in this repository. Read it fully before making any change; when resuming an
interrupted session, treat it as a fresh entry and redo the §1 self-check.
HANDOFF.md records the current working state (decisions, open threads) — read
it right after this file. If this file contradicts the actual code, the code
wins — and §3 requires fixing the docs in the same change.

## 1. Entering the repository: routine self-check (every time)

Before touching anything, verify three things:

1. **pre-commit is enabled** — `git config core.hooksPath` must print
   `githooks`. If empty, run (prefer `just setup`, which also installs missing
   tools):

   ```bash
   git config core.hooksPath githooks
   ```

2. **hook dependencies are installed** — four external tools must be on PATH:

   ```bash
   command -v cargo-machete cargo-audit cargo-outdated cargo-deny
   ```

   Install whatever is missing (with `--locked`):

   ```bash
   cargo install cargo-machete cargo-audit cargo-outdated cargo-deny --locked
   ```

   Note: `cargo fmt` and `cargo clippy` are guaranteed by the components
   declared in `rust-toolchain.toml`; rustup installs them with the toolchain.

3. **toolchain** — `rust-toolchain.toml` declares `channel = "stable"`; rustup
   resolves the latest stable automatically. Never hardcode a version number
   and never bypass this file.

When in doubt about environment health, run `githooks/pre-commit` end to end
as a smoke test (the first run fetches the RustSec database; slowness is
normal).

## 2. Lint errors: waiver discipline

Principle: **fix the code first; a waiver is the last resort, and only
code-level.**

- Never "make errors disappear" by editing `Cargo.toml` `[lints]`,
  `githooks/pre-commit`, or any check command.
- When a waiver is truly needed, relax **in code only**:
  - prefer `#[expect(clippy::lint_name)]` (it starts producing a compile
    warning once the lint stops firing, preventing stale allows), fall back to
    `#[allow(clippy::lint_name)]`;
  - minimal scope: a single statement or one function; never function groups,
    module-level `#![allow(...)]`, or crate-level relaxation;
  - a one-line reason comment at the waiver point is mandatory (plus a linked
    issue, if any).
- Only two legitimate scenarios:
  1. **genuinely unavoidable** — the business need demands it and no equally
     reasonable alternative exists;
  2. **upstream problems** — false positives, macro/derive-generated code, or
     audit noise from dependencies themselves (e.g. RustSec unmaintained
     notices).
- All other audits and extra checks (machete, audit, deny, outdated,
  docs-sync, and anything added later) follow the **same discipline**: fix if
  fixable; waive only as above when truly unfixable. Never delete, comment
  out, or bypass a check.
- The chain has two layers: **fast gates** (`githooks/pre-commit`: fmt /
  secrets / machete / docs / clippy) run on commit, **heavy gates**
  (`githooks/pre-push`: audit / deny / outdated / test) run on push; CI runs
  the whole chain via `just check`. All three are "the checks" and bound by
  this discipline.

## 3. Before every commit: docs ↔ code alignment (every commit)

- Verify the docs still tell the truth about the code:
  - lint tables in docs/lint-policy.md / docs/lint-policy.zh.md ↔
    `[lints]` in `Cargo.toml`;
  - gate tables in docs/checks.md / docs/checks.zh.md ↔ the actual commands in
    both hooks (`githooks/pre-commit` and `githooks/pre-push`);
  - README.md / README.zh.md as landing pages: quick-start commands, docs
    index links, and feature claims still hold;
  - toolchain description ↔ `rust-toolchain.toml`; layout ↔
    docs/structure(.zh).md; command examples; version numbers;
  - source doc comments (`//!` / `///`) ↔ actual behavior.
- Docs are bilingual pairs (`*.md` + `*.zh.md`) and must change together;
  never update one language only.
- Changing lint config or the check chain requires syncing the affected docs
  pages, both READMEs, and this file **in the same commit**.
- The mechanical part is automated in `githooks/check-docs`, wired into the
  pre-commit chain. It only covers greppable invariants (hook commands ↔
  docs/checks, lint names ↔ docs/lint-policy, edition, channel, just recipes,
  README docs index, CI entry, CHANGELOG extraction, test-build entry,
  secret-scan gate).
  **Semantic alignment** (outdated prose, runnable examples, consistent tone)
  cannot be mechanized — it stays with the agent or a human reviewer.

## 4. Commit message convention

- **English only**, regardless of the author's language.
- Conventional Commits prefixes: `feat:`, `fix:`, `docs:`, `chore:`,
  `refactor:`, `test:`, `ci:`, `perf:`.
- Subject line: imperative mood ("add", not "added"), ≤ 72 characters, no
  trailing period.
- Body (optional): explain **why**, wrap long lines; breaking changes append
  `!` to the type and carry a `BREAKING CHANGE:` footer.
- Every commit must pass the pre-commit gate — it runs automatically; do not
  use `--no-verify`.

## 5. Releases: tag-driven, automated

- **Releases are tag-driven.** The only trigger of a release is pushing a
  `v*` tag; `.github/workflows/release.yml` owns the whole flow and no other
  path publishes a release.
- `CHANGELOG.md` is the **single source of release notes**, maintained in
  [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format and
  following [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
- During development, record notable changes under `## [Unreleased]`.
- Before tagging, move that content into a dated section:
  `## [x.y.z] - YYYY-MM-DD` (the git tag is the same version with a `v`
  prefix, e.g. `v0.2.0`).
- Pushing a `v*` tag triggers `.github/workflows/release.yml`, which:
  1. extracts the section matching the tag from `CHANGELOG.md` and uses it as
     the release notes,
  2. creates the GitHub Release,
  3. builds and attaches archives for three targets (x86_64-linux-gnu,
     aarch64-macOS, x86_64-windows-msvc).
- A missing or empty changelog section **fails the release**. Fix: add the
  section, delete and re-push the tag. Never hand-edit release notes on
  GitHub; the changelog is the source.
- **Tag-push policy: no casual release pushes.** Commits are always allowed —
  the fast gates guard them and they trigger nothing public. Pushing a `v*`
  tag is a deliberate release act; **all** of the following must hold before
  pushing one:
  1. an explicit human request (agents must never create release tags on
     their own initiative);
  2. `version` in `Cargo.toml` equals the tag version;
  3. a dated `## [x.y.z] - YYYY-MM-DD` section exists in `CHANGELOG.md`;
  4. `just check` is green on the tagged commit.
  Re-tagging is allowed only to fix a failed release (delete the tag, fix,
  re-push). For verifying a commit without releasing, use CD test builds (§6).

## 6. CD test builds: per-commit, per-platform artifacts

- `.github/workflows/test-build.yml` builds **test artifacts** from any
  commit without creating a release: dispatch it manually from the Actions
  tab, choose a `ref` (commit SHA, branch, or tag) and `targets`
  (`linux`, `macos`, `windows`).
- Artifacts are ephemeral (7-day retention) and are never a Release — do not
  hand out release links for them, and do not reference them in the
  changelog.
- Typical uses: verifying that a specific commit compiles on all platforms
  before tagging (§5), and reproducing platform-specific issues on an exact
  commit.

## 7. Deriving a new project from this template

- Click **Use this template**, then follow the rename checklist in
  docs/using-this-template.md. The three traps that break automation if
  missed:
  1. `Cargo.toml` — `name` / `repository`;
  2. `tests/cli.rs` — `env!("CARGO_BIN_EXE_rust-agents-template")`;
  3. `.github/workflows/release.yml` — `bin: rust-agents-template`.
- After renaming: `just setup` → `just check` → first commit (the pre-commit
  gate runs automatically and `just check` is the safety net for anything
  missed).
- Start the project changelog in `CHANGELOG.md` under `## [Unreleased]` (§5).

## 8. Day-to-day operations

- commit → fast gates; push to a branch → heavy gates; **push of a `v*` tag →
  release (§5, deliberate)**; PR or push to `main` → CI runs the identical
  chain; branch protection on `main` requires the `full check chain` check,
  forbids force-pushes, and auto-deletes merged branches.
- Formatting: `just fmt` auto-fixes; `just check` rehearses the whole chain
  before committing.
- Dependencies: add or remove them only through cargo — `cargo add` (add
  `--dev` for dev-dependencies, `--package` in workspaces) and
  `cargo remove`. Never hand-edit the `[dependencies]` / `[dev-dependencies]`
  tables in `Cargo.toml`: `cargo add` resolves a compatible version
  requirement and updates `Cargo.lock` in the same step, avoiding
  hand-written specs that drift from the lock or trip the dependency gates.
- Maintenance: Dependabot opens weekly updates for GitHub Actions and cargo
  dependencies; they merge only with CI green.
- Security reports go through GitHub's private vulnerability reporting
  (SECURITY.md), never public issues.

## 9. Working discipline (daily rules)

- **Stage with eyes open.** Review `git status` and stage selectively
  (`git add -p`); never blanket `git add -A` while the worktree holds
  unrelated changes. One commit = one logical change: features, refactors,
  and fixes do not share a commit.
- **main stays releasable.** Direct pushes to main are allowed, so CI red on
  main is the top priority — fix it before starting new work; experiments go
  to a branch.
- **No drive-by dependency upgrades.** Upgrades are Dependabot's job (or a
  dedicated commit); never bundle them into feature work — keep bisect clean.
- **CHANGELOG as you go.** A user-visible change and its `## [Unreleased]`
  entry land in the same commit; never backfill at release time (§5).
- **Prove it, don't assume it.** Every "it works" claim must be backed by
  real command output from this session; no output, no claim.
- **No corpses.** Commented-out code and `todo!()` stubs get removed, not
  accumulated (the `todo` lint already watches).
- **End-of-session ritual.** A session ends with `just fmt` + `just check`,
  everything committed and pushed — never a dirty tree, never unpushed
  commits.
- **Timebox rabbit holes.** Three failed attempts on the same problem: stop,
  write the findings into HANDOFF.md, and ask the human.
- **Clear → act; ambiguous or irreversible → ask.** Renames, deletions,
  settings changes, and anything touching releases need the human's go.
- **Secrets never enter the repository.** Tokens, keys, and credentials live
  in repo settings / environment only — never in code, docs, or commits.
  Enforced mechanically by `githooks/check-secrets` in the pre-commit chain;
  a line that must carry a secret-shaped string takes a
  `security-scan:allow` marker with a reason.

## 10. Documentation map

| Question | Where |
|----------|-------|
| How to derive and rename a new project | docs/using-this-template.md |
| What each gate runs, how to handle a block | docs/checks.md |
| Lint levels and waiver rules | docs/lint-policy.md |
| Release mechanics, test builds | docs/release.md |
| What every file in this repo is for | docs/structure.md |
| Current working state, decisions, open threads | HANDOFF.md |

Every page has a `*.zh.md` counterpart; §3 governs their sync.

## 11. One-line summary

> Self-check the environment on entry; when a check blocks you, fix the code —
> waive only as a last resort, locally, with a named reason; keep docs and
> code in the same commit; write commit messages in english; commits are free,
> release tags are deliberate; let releases speak through CHANGELOG.md; prove
> every claim with real output; end sessions clean; secrets never enter the
> repo.
