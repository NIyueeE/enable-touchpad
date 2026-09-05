# HANDOFF.md — working state for agents

Agent-facing handoff notes. AGENTS.md owns the **rules**; this file owns the
**state**: where the project stands, which decisions were made and why, and
what is still open. Update this file (in the same commit) whenever you change
something the next session needs to know.

## Quick orientation

- What this repo is: `enable-touchpad` — a Windows 11 tray utility that
  soft-enables the touchpad while a configurable layer key is held and
  soft-disables it again afterwards. It embeds kanata as a library and is
  layered for future platforms.
  Live: https://github.com/NIyueeE/enable-touchpad
- Read order: AGENTS.md (rules) → this file (state) → docs/ (topic pages).
- Local loop: `just setup` (once) → `just check` (full chain) → commit/push.
- Tests: `just test` (cargo-nextest when installed, otherwise `cargo test`).

## Current state (2026-09-01, `main`)

- **Tray + settings-window architecture**: all tray/window mutations run
  on the main thread via a message-only "door" window
  (`etp_ffi::window`: `init(handler)` + `post(task, param)`, handler
  registered as `app::on_door_message`). tray-icon/muda state lives in
  main-thread `thread_local`s (their handles are `Rc<RefCell>` — never
  mutate from the forwarder thread). Left click = master toggle, menu =
  打开设置 + checkable 总开关 + 退出, icon dims when the master switch is off. The master switch's single write point is
  `WatchdogState::set_managed` (writes `main.rs::MASTER_SWITCH`); the UI
  checkbox lives in a `SyncStorage` signal
  (`app::FEATURE_SIGNAL` + `use_signal_sync`) so tray toggles update it
  live. Showing the settings window is a door task (plain
  a door task handled ON the main thread with tao's own
  `set_minimized/set_visible/set_focus` (tao's executor runs inline
  there) — do NOT call them from the forwarder thread (tao's
  thread-executor hop froze the app; device-reported) and do NOT show
  the window with raw ShowWindow (that desyncs tao's visibility state
  and made the ✕ close a no-op; device-reported).
- **Deterministic touchpad state sync**: kanata no longer blind-fires the
  Ctrl+Win+F24 toggle on layer entry/exit (a toggle inverts the state
  when the touchpad was already on). The watchdog queries
  `touchpad_enabled()` and taps the chord only on mismatch — at layer
  entry, layer exit, and idle, in both directions, with a 400 ms settle
  guard after any tap. SPI-less machines keep the legacy blind tap per
  transition; the tray-quit tap is query-guarded. Idle enforcement still
  reverts a manual "on" set in Windows Settings — flagged to the user as
  a design decision (adaptive baseline offered).
- **Device findings (build 8fd9ae4)**: the `ActOnFakeKey` fake-key-over-TCP
  delivery no-opped on real hardware and 3 failed corrections slept the
  watchdog thread 60 s → badge stuck on + transitions dead + touchpad
  would not enable. Fixes: chord injected directly via `etp_ffi::chord`
  (SendInput, scancodes + extended flag, mirrors kanata oskbd byte-for-
  byte; the config-macro chord is the device-proven mechanism), backoff
  is a timestamp instead of a sleep, transitions honour the settle guard,
  and logging falls back to the exe dir with the real path shown in the
  settings footer. If the toggle still fails on device, capture
  `%APPDATA%\enable-touchpad\enable-touchpad.log` (footer shows the real
  path) — it now logs every mismatch/tap/backoff decision.
- **Mouse-layer cursor badge**: a click-through layered overlay
  (`etp-ffi/src/cursor_badge.rs`, `UpdateLayeredWindow`, no webview)
  pins `assets/icon_16.png` to the cursor while the layer is held;
  the anchor mimics the native Help Select cursor — hugging the arrow
  glyph's right edge (sprite box × 45% − 2px), halfway down
  (SM_CXCURSOR/SM_CYCURSOR, DPI-aware) — with a 150 ms show delay,
  and visibility is driven by the watchdog's expected state
  (`set_expected`/`set_managed` → `cursor_badge::set_visible`). The
  owner thread polls at ~125 Hz only while visible; a process kill
  leaves nothing behind (no system cursor mutation). Win32 struct
  layouts are hand-mirrored — verified by cross-compile only, real
  device check pending.
- **Real icon assets**: the user-supplied touchpad artwork lives in
  `assets/` (sources in `assets/src/`). `build.rs` (winresource) embeds
  `assets/icon.ico` (16/24/32/48/64) as exe icon resource 1 — Explorer,
  taskbar, and the tray (`Icon::from_resource(1, Some((32, 32)))`,
  in-process disc as fallback) all use it; `assets/icon_32.png` is
  decoded at startup (png crate, windows dep) for the tao window icon.
  On hosts without a Win32 resource compiler (Linux cross checks) the
  embed degrades to a cargo warning; Windows CI runners embed it for
  real. Regenerate the ico with `python3 assets/make_icons.py`
  (needs Pillow; verified byte-identical to the committed assets).
- **UI v3 + robustness pass**: the settings window was redesigned
  (card sections, keycap-style capture buttons, a real toggle switch
  for the master switch, coloured save-status pills, an amber inline
  hint for duplicate bindings; 440×486, headless-render verified in
  dark/light/capture/conflict states with the real stylesheet).
  Hidden-bug fixes in the same pass: watchdog busy-spin once the
  engine's layer-event sender dies; touchpad stranded ON when the
  master switch went off while the layer was held (`set_managed(false)`
  now queries the official state and taps once); click-vs-CapsLock
  conflict resolution now follows the documented first-claim order
  with a regression test; 3 s control-channel timeouts so a hung
  kanata connection cannot freeze the UI thread; atomic config.json
  writes (temp + rename); tray left click opens settings; tray
  installation failures are logged; opening settings from the tray
  restores a minimized window; a second launch leaves a log entry.

- **Merged**: `demo/windows-feasibility` graduated into `main` with a
  `--no-ff` merge; the demo branch stays as an archive. The Windows 11
  adapter is marked **adapted** in the READMEs and the demo notes.
- **Renamed**: the root package is now `enable-touchpad` (was
  `rust-agents-template`). The template hello-world binary and its smoke test
  were removed; the real app binary is `src/bin/enable-touchpad/`.

- **Layered architecture (this branch)**: the app binary is the application
  layer (`src/bin/enable-touchpad/`: UI, tray, config store, watchdog),
  `etp-platform/` is the **single platform-adaptation layer** (`Platform`
  trait; `windows` adapter with the embedded kanata engine and touchpad query;
  non-Windows fallback), and `etp-core/` is the pure cross-platform domain
  layer (config model, key allowlist, config generator). The former
  `etp_core.rs` / `kanata_embed.rs` / `touchpad_state.rs` binary modules were
  dissolved into these crates. `etp-ffi/` remains a Windows-only FFI leaf
  (now `#![cfg(windows)]`, workspace member). `cargo test` runs all workspace
  members via `default-members`.
- **Project identity**: the repo was derived from a strict Rust template and
  has now adopted its real project identity; the old template release history
  and tags were reset earlier, and a clean baseline `v0.1.0` validated the
  CI/CD flow.
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
- Windows-only code now lives in `etp-platform/src/windows/` and in the
  root package's `[target.'cfg(windows)'.dependencies]` (dioxus, tray-icon,
  log, simplelog); Linux gates stay green. Verification on Linux:
  `cargo clippy --all-targets --all-features -- -D warnings` (host) plus
  `cargo clippy --target x86_64-pc-windows-msvc --all-targets --all-features
  -- -D warnings` (cross compile). Windows 11 real-device testing is done;
  the adapter is marked adapted.

## Platform roadmap (planned, not scheduled)

- The only multi-platform adaptation point is `etp-platform/` (`Platform`
  trait). Application code must stay platform-neutral.
- Current backends: `windows` (adapted, verified on Windows 11 with a
  precision touchpad) and `fallback` (non-Windows placeholder).
- **macOS (planned)**: add `etp-platform/src/macos/`. The toggle mechanism
  must be researched first (CGEventTap / IOHID / a soft-toggle equivalent);
  do not copy the Windows Ctrl+Win+F24 chord or `etp-ffi` FFI.
- **Linux (planned)**: add `etp-platform/src/linux/`. Candidate directions
  are libinput / uinput / xinput; X11 vs Wayland differences must be
  validated before committing to one.
- Before the first non-Windows backend: generalize the `Platform` trait —
  `start_engine`, `apply_engine_config`, `tap_toggle_chord`, and
  `touchpad_enabled` are currently Windows-shaped. Keep Windows specifics
  inside the `windows` adapter; other platforms get their own FFI leaf
  crates only if they need `unsafe`.
- Each new platform must be verified on real hardware before its README
  status changes to "adapted"; Linux host gates must stay green throughout.

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
- cargo-nextest evaluation (2026-08-31): works, but the suite was then a
  single smoke test. The template smoke test is gone; unit tests now live in
  `etp-core`. Adopted as optional via `just test`; pre-push and CI still run
  `cargo test`. Revisit when the suite grows past ~10 tests.
- Consider tag-protection rulesets (restrict `v*` creation) if the repo gains
  collaborators.
- Only the linux leg of `test-build.yml` has been exercised (see current
  state).

## Gotchas

- On this dev container `RUSTUP_HOME` points at the read-only
  `/opt/rust/rustup`, so every cargo/rustup call dies with
  `could not create temp file ... Permission denied` (rustup tries to
  sync the `stable` channel from rust-toolchain.toml). A writable copy
  of the toolchain home with the Windows cross target already added
  lives at `~/.rustup`: prefix cargo and hook invocations with
  `export RUSTUP_HOME=$HOME/.rustup`.
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
