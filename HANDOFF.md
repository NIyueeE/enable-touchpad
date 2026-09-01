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
- **v6 toggle semantics (sim-verified; CONFIRMED WORKING on the user's
  device)**: the CapsLock hold key taps the real Ctrl+Win+F24 **chord**
  (`(macro C-M-f24)` = all three keys down, reverse-order release) on
  press, and a second tap on release via `(deffakekeys release-tap (macro
  C-M-f24))` + `(on-release-fakekey release-tap tap)`. The earlier
  sequential `(macro lctl lmeta f24)` emitted a lone Win tap and popped the
  Start menu on the device. Sim wave (v7 tests live in the kanata clone at
  `src/tests/sim_tests/etp_sim_tests.rs`, not in this repo):
  `↓LCtrl ↓LGui ↓F24 ↑F24 ↑LGui ↑LCtrl` at press and at release.
- **v7 key capture + state watchdog (current)**:
  - Bindings are **captured, not chosen from a list**: click a row's
    button, press any key. `AppConfig` stores W3C `KeyboardEvent.code`
    strings (`"KeyF"`, `"AltLeft"`) — kanata's parser accepts these
    verbatim (`str_to_oscode`), and legacy short names (`q`, `lalt`) stay
    valid, so old config files keep loading. The allowlist
    (`etp_core::SUPPORTED_CODES`, ~100 codes) was cross-checked against
    kanata v1.11.0's name table and Windows scancode conversion, and a
    config containing EVERY allowlisted key passed `kanata --check`.
    Reserved and rejected: `Escape` (capture cancel), `CapsLock` (fixed
    defsrc hold key), `PrintScreen`/media keys (no kanata name / name
    mismatch with web codes).
  - Generator de-duplicates layer keys (first claim wins) and skips the
    hold-key collision; pure logic lives in the cross-platform
    `etp_core.rs` (replaces `config.rs`) so unit tests run on Linux.
  - **State watchdog** (`touchpad_state.rs`): kanata broadcasts
    `LayerChange` to EVERY connected TCP client (no subscription needed),
    so a persistent monitor connection feeds expected state to the
    watchdog (mouse layer = touchpad ON, base = OFF). While idle it reads
    the official state via `SPI_GETTOUCHPADPARAMETERS` (0x00AE,
    `TOUCHPAD_PARAMETERS_V1`: 44 bytes, `touchpadEnabled` = bit 3 of the
    first C bit-field word — from `WinUser.h`; the constant AND struct are
    missing from windows-rs metadata) and corrects drift to OFF by firing
    `{"ActOnFakeKey":{"name":"release-tap","action":"Tap"}}` — the same
    soft chord, no device touching (the success path writes no response).
    1.2 s cadence, 1.5 s cooldown after any chord tap (a flip lags), 3
    failed corrections → 60 s backoff. Master switch off = unmanaged
    (stock behaviour preserved). Tray quit sends one best-effort tap if
    the layer was held, so the touchpad is not stranded ON.
  - `etp-ffi/` is a tiny path-dependency crate wrapping the two `unsafe`
    FFI calls: the main crate's `unsafe_code = "forbid"` cannot be relaxed
    locally (rustc E0453), so the FFI boundary moved into its own crate
    with a compile-time `size_of == 44` layout assert. Needs Windows 11 +
    a precision touchpad; anything else logs "SPI unavailable" once and
    the watchdog stays inert.
  - **Chord-isolation proofs (sim, 7 tests green)**: (a) inside the mouse
    layer the mapped layer keys emit ONLY their own actions — F24 appears
    exactly twice per hold (press + release chord); (b) unmapped keys pass
    straight through (`process-unmapped-keys no`) and never chord; (c)
    OS auto-repeat is safe: kanata's Windows LL hook converts repeated
    KEYDOWNs of a held key into `KeyValue::Repeat` (PRESSED_KEYS
    dedup, `src/kanata/windows/llhook.rs`), and `handle_repeat` only
    re-emits CURRENTLY-held output keys — after the macro finishes there
    is nothing held, so 5 injected repeats produced zero events in sim.
    NOTE: flooding sim events with no `t:` delays DOES interleave the
    macro steps — a sim artifact, not real behaviour (real repeats arrive
    no sooner than ~30 ms apart).
  - UI: capture buttons + per-row × (reset to none), Esc cancels, root
    `onkeydown` uses `ev.code().to_string()` (keyboard-types 0.7 has no
    `as_str`); unsupported keys show a red hint and stay in capture mode.
    Window grew to 440×400 — 360/384 clipped the footer line (verified in
    headless chromium renders of dark/light/capture states).
  - serde/serde_json moved to unconditional dependencies (the config
    model is cross-platform now).
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
