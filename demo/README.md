# Windows feasibility demo

> English | [简体中文](README.zh.md)

A proof-of-concept for the `enable-touchpad` idea on Windows, shipped as a
**single executable**:

- **kanata runs embedded inside the app** (as a library, `kanata` v1.11,
  low-level-hook capture — no kernel driver, no separate process). While
  `CapsLock` is held, a `mouse` layer activates (`Q`/`W`/`E` → mouse
  buttons, `Left Alt` → `CapsLock`) and **Ctrl+Win+F24 is tapped once on
  press and once on release**. The touchpad soft on/off is performed by
  whatever the operating system / touchpad driver binds that combo to —
  this app never enables or disables devices itself.
- **the Dioxus UI** is a small settings window for the layer key bindings
  (hidden by default, opened from the tray), with changes hot-applied to the
  embedded kanata over its local TCP `Reload` command.

## Layout

| Path | Purpose |
|------|---------|
| `../src/bin/enable-touchpad/` | the app: `main` / `app` (UI + logging) / `config` / `kanata_embed` (config generation + embedded kanata) / `tray` |

The app compiles only for Windows; other targets build a stub so the
repository's Linux gates stay green.

## Windows setup

1. **Build or download** the exe (`cargo build --release` on Windows, or the
   `test-build` CI artifact).
2. **Run it as administrator.** The generated kanata config lives at
   `%APPDATA%\enable-touchpad\kanata.kbd`; the log at
   `%APPDATA%\enable-touchpad\enable-touchpad.log`.
3. Hold `CapsLock` → the `mouse` layer activates (configurable `Q`/`W`/`E`
   bindings) and the system performs the touchpad soft on/off.
4. Settings: right-click the tray icon → `打开设置`. Saving regenerates the
   config and hot-reloads the embedded kanata — no restart needed.

Do **not** run an external kanata at the same time (double key capture).

## Demo limitations

- The Ctrl+Win+F24 tap semantics (activate on press, deactivate on release)
  depend on how the system's touchpad driver handles the combo; if your
  driver toggles differently, the layer config in
  `%APPDATA%\enable-touchpad\kanata.kbd` is the single place to adjust.
- Touchpad enable/disable is entirely system-owned — the app has no fallback
  for machines where the combo is unbound.
- The UI renders in a system WebView (dioxus desktop); a pure-GPU native
  renderer (Blitz) is not production-ready yet.
