# Windows feasibility demo

> English | [简体中文](README.zh.md)

A proof-of-concept for the `enable-touchpad` idea on Windows, shipped as a
**single executable**:

- **kanata runs embedded inside the app** (as a library, `kanata` v1.11,
  low-level-hook capture — no kernel driver, no separate process). While
  `CapsLock` is held, a `mouse` layer activates and **Ctrl+Win+F24 is
  tapped once on press and once on release**. The touchpad soft on/off is
  performed by whatever the operating system / touchpad driver binds that
  combo to — this app never enables or disables devices itself.
- **layer keys are captured, not picked from a list**: click a row's
  button in the settings window and press any supported key (letters,
  digits, F-keys, modifiers, numpad, …); each of the mouse buttons and the
  CapsLock action gets its own key. `Escape` cancels a capture, `×`
  resets a row to "none", and `CapsLock` itself stays the fixed layer
  hold key.
- **a state watchdog keeps the touchpad off while the layer key is not
  held**: the official precision-touchpad state
  (`SPI_GETTOUCHPADPARAMETERS`, Windows 11+) is sampled every ~1.2 s and
  drift is corrected with the same soft Ctrl+Win+F24 chord via kanata's
  fake-key channel. With the master switch off the touchpad belongs to
  the system again.
- **the Dioxus UI** is a small settings window (hidden by default, opened
  from the tray), with changes hot-applied to the embedded kanata over its
  local TCP `Reload` command.

## Layout

| Path | Purpose |
|------|---------|
| `../src/bin/enable-touchpad/` | the app: `main` / `app` (UI + logging) / `etp_core` (config model, key allowlist, kanata config generator) / `kanata_embed` (embedded kanata, layer monitor, control channel) / `touchpad_state` (state watchdog) / `tray` |
| `../etp-ffi/` | tiny FFI crate: `SPI_GETTOUCHPADPARAMETERS` touchpad state query (isolates `unsafe`; the app crate forbids it) |

The app compiles only for Windows; other targets build a stub so the
repository's Linux gates stay green.

## Windows setup

1. **Build or download** the exe (`cargo build --release` on Windows, or the
   `test-build` CI artifact).
2. **Run it as administrator.** The generated kanata config lives at
   `%APPDATA%\enable-touchpad\kanata.kbd`; the log at
   `%APPDATA%\enable-touchpad\enable-touchpad.log`.
3. Hold `CapsLock` → the `mouse` layer activates (captured key bindings)
   and the system performs the touchpad soft on/off.
4. Settings: right-click the tray icon → `打开设置`. Saving regenerates the
   config and hot-reloads the embedded kanata — no restart needed.

Do **not** run an external kanata at the same time (double key capture).

## Demo limitations

- The Ctrl+Win+F24 tap semantics (activate on press, deactivate on release)
  depend on how the system's touchpad driver handles the combo; if your
  driver toggles differently, the layer config in
  `%APPDATA%\enable-touchpad\kanata.kbd` is the single place to adjust.
- Touchpad enable/disable is entirely system-owned — the app has no fallback
  for machines where the combo is unbound. The state watchdog needs
  Windows 11 and a precision touchpad (`SPI_GETTOUCHPADPARAMETERS`);
  elsewhere it logs "SPI unavailable" once and stays inert.
- Duplicate bindings collapse to the first claim (e.g. one key bound to two
  actions acts for the first only).
- The UI renders in a system WebView (dioxus desktop); a pure-GPU native
  renderer (Blitz) is not production-ready yet.
