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

### Windows feasibility demo (branch `demo/windows-feasibility`, 2026-08-31)

- Goal: validate the enable-touchpad idea — hold CapsLock in a kanata layer
  → touchpad enabled + Q/W/E act as mouse buttons; release → restore and
  soft-disable. Built per the user's hand-drawn sketch.
- **v3 (current), after the user's device test rejected v2 choices**:
  - touchpad on/off is NOT app-controlled: the embedded kanata taps
    Ctrl+Win+F24 on CapsLock press AND release; the system / touchpad driver
    owns the soft toggle. The PowerShell PnP device-disable path was removed
    (user rejected hard device disable).
  - signal sources collapsed to the combo only: the TCP LayerChange reader
    and the rdev F24 watcher were removed ("no backward-compatible保留 of
    rejected features").
  - the mouse-following indicator was removed (user: useless + render cost).
  - the settings window is small (460×340), starts hidden, opens only from
    the tray right-click menu (打开设置 / 退出). UI v2: Gruvbox palette,
    system light/dark via `prefers-color-scheme`, native decorations AND
    menu bar removed (custom draggable title strip with minimize/hide
    buttons), scrollbar hidden, custom-styled select/checkbox.
  - the settings window is small, starts hidden, opens only from
    the tray right-click menu (打开设置 / 退出). UI v2: Gruvbox palette,
    system light/dark via `prefers-color-scheme`, native decorations AND
    menu bar removed (custom draggable title strip with minimize/hide
    buttons), scrollbar hidden, custom-styled select/checkbox. UI v3:
    config rows are action-first ("鼠标左键 → key"), matching the user's
    mental model; title-strip buttons stop mousedown propagation so
    drag_window doesn't swallow their clicks.
  - the settings page configures the layer bindings (action-first:
    left/middle/right click and CapsLock each pick their layer key) plus
    a master switch; saving regenerates the kanata
    config and hot-applies it via the TCP `Reload` command (protocol:
    `{"Reload":{"wait":true}}` on 127.0.0.1:5829, internal only).
  - detailed logging goes to `%APPDATA%\enable-touchpad\enable-touchpad.log`
    via simplelog (kanata's `log` output lands there too); the in-app log
    area was removed.
  - the static `demo/kanata/*.kbd` file was removed — the config is
    generated from code (single source of truth) and syntax-validated with
    `kanata --check` (switch-case form: `((input real caps)) <action> break`,
    three sibling forms per case).
- **v5 layer fix (verified with kanata's own simulation harness)**: the v4
  config put `layer-while-held` inside a `switch` case — kanata's sim
  (`simulated_output` feature, `simulate()` in `src/tests/sim_tests/mod.rs`
  of the kanata repo) proved the layer never activates in that position
  (the letter Q passed through, hitting Win+Ctrl+Q = Quick Assist dialog on
  the user's machine). The switch construct was replaced with the verified
  `(multi (layer-while-held mouse) (macro lctl lmeta f24))`: full combo tap
  at press, layer active while held, Q/W/E emit mouse buttons
  (`out🖰:↓Left` in sim). NOTE: the "release taps the combo again"
  (soft-disable) is NOT yet implemented — no clean kanata construct exists
  (`fork` third arg must be a key list, not an action); awaiting the user's
  driver-behaviour report from real-machine testing.
- **Single-exe architecture**: kanata v1.11 embedded as a library
  (`kanata_state_machine` from crates.io, default features minus zippychord
  = LL-hook capture + SendInput output — NO Interception driver and NO
  external kanata process). Startup mirrors kanata's own `win_gui.rs`:
  `ValidatedArgs` → `new_arc` → TcpServer → `start_processing_loop` →
  `start_notification_loop` → `event_loop` (blocking LL-hook thread).
- License note: embedding kanata pulls **LGPL-3.0-only** into the dependency
  graph (allowed in deny.toml with a comment). Private/internal use has no
  obligations, but DISTRIBUTING the exe requires the kanata source link plus
  object files / relink means per LGPL §4/§6.
- **v4 device-test fixes**: (a) tray "open settings" was a silent no-op — the
  v3 refactor dropped the `MAIN_WINDOW` registration; the window handle is
  now registered from a one-shot `use_future` in `ui_root`. (b) the exe is
  built as a GUI-subsystem binary (`windows_subsystem = "windows"`) so no
  console pops up on double-click. (c) single-instance guard: a sentinel
  listener on 127.0.0.1:58270; a second launch exits immediately (the user
  had two instances double-capturing keys). The loopback TCP server remains
  because it is kanata 1.11's only programmatic reload surface — internal
  IPC only, never UI-facing.
- `src/bin/enable-touchpad/`: Dioxus 0.7 desktop app — tray (`tray-icon`,
  leaked handle; `TrayIcon` is `!Send`), settings page, click-through
  indicator window (`DesktopContext::new_window` + tao
  `set_ignore_cursor_events`), touchpad toggle via PowerShell
  `Enable/Disable-PnpDevice` (needs admin; no `unsafe` anywhere).
- Key API findings (dioxus 0.7.10): default `Signal` is **Unsync** (thread-local)
  → cross-thread UI state uses `use_signal_sync` + statics;
  `use_hook` was removed in 0.7; `Signal::set` needs `&mut self`;
  `use_future` closure is `FnMut`; wry has no click-through — go through
  tao's `set_ignore_cursor_events`; rdev has no `Key::F24` (match
  `Key::Unknown(0x87)`); muda `MenuItem::with_id` takes
  `Option<Accelerator>`.
- Windows-only code lives under `cfg(windows)` + `[target.'cfg(windows)'.dependencies]`
  so Linux gates stay green; verification on Linux = `cargo check/clippy
  --target x86_64-pc-windows-msvc` (compiles, not run). Real-device testing
  still pending on a Windows machine.

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
- Demo branch: deny.toml gained (a) `MPL-2.0` in the license allowlist for
  the servo/HTML stack dioxus pulls in (cssparser/selectors/dtoa-short,
  option-ext) and (b) 14 advisory ignores for GTK3/glib/fxhash/paste/
  proc-macro-error/rand "unmaintained/unsound" notices — all present only
  because `[graph] all-features = true` collects every platform's metadata;
  none compile for the Windows target. Re-evaluate and drop the ignores if
  the demo graduates to a cross-platform product.

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
