# Windows feasibility demo

> English | [简体中文](README.zh.md)

A proof-of-concept for the `enable-touchpad` idea on Windows, shipped as a
**single executable**:

- **kanata runs embedded inside the app** (as a library, `kanata` v1.11): it
  captures the keyboard through a low-level hook (no kernel driver needed),
  activates a `mouse` layer while `CapsLock` is held (`Q`/`W`/`E` → mouse
  left/middle/right buttons, `Left Alt` → `CapsLock`), and broadcasts the
  layer state over a local TCP socket inside the same process.
- **the Dioxus UI** reacts to the layer signal, **enables the touchpad while
  CapsLock is held and disables it on release**, shows a click-through
  indicator at the mouse position, and provides a tray icon plus a minimal
  settings page.

```
enable-touchpad.exe (single file)
├── kanata (embedded lib, LL-hook capture + SendInput output)
│     ├── hold CapsLock → "mouse" layer + Q/W/E = mouse buttons
│     └── LayerChange → 127.0.0.1:<port> (in-process TCP self-connect)
├── Dioxus UI (tray + settings page + mouse-following indicator)
└── touchpad enable/disable (PowerShell PnP device toggle)
```

## Layout

| Path | Purpose |
|------|---------|
| `kanata/enable-touchpad.kbd` | kanata layer config, embedded into the binary at compile time |
| `../src/bin/enable-touchpad/` | the app: `main` / `app` (UI) / `config` / `kanata_embed` / `signal` / `touchpad` / `tray` |

The app compiles only for Windows; other targets build a stub so the
repository's Linux gates stay green.

## Windows setup

1. **Build or download** the exe (`cargo build --release` on Windows, or the
   `test-build` CI artifact).
2. **Run it as administrator** (device enable/disable is a system-level
   operation). The embedded kanata config is written to
   `%APPDATA%\enable-touchpad\kanata.kbd` on first launch.
3. Hold `CapsLock` → the touchpad turns on, a blue pill appears next to the
   mouse cursor, and `Q`/`W`/`E` act as mouse buttons. Release → everything
   is restored and the touchpad turns off.

No separate kanata installation and no kernel driver are needed — kanata's
default Windows mode uses a low-level keyboard hook. Do **not** run an
external kanata at the same time (double key capture and a TCP port clash).

## Settings page

- **触摸板状态** + manual enable/disable/refresh buttons (works without kanata —
  handy for smoke-testing permissions).
- **信号源**: TCP (`LayerChange` stream, recommended) or F24 key events.
- **总开关 / 指示器**: master switch for the feature and the mouse indicator,
  with a preview button.
- **应用设置 / 保存配置**: runtime apply (live) and persistence to
  `%APPDATA%\enable-touchpad\config.json`.

## Demo limitations

- Touchpad enable/disable shells out to PowerShell
  (`Disable-PnpDevice`/`Enable-PnpDevice` matching `touchpad|触摸板` friendly
  names) — demo-grade; a production build would use CfgMgr32 from a small
  elevated helper.
- The device must expose a friendly name containing "touchpad"/"触摸板";
  PS/2-touchpad OEM names may need the pattern extended.
- The UI renders in a system WebView (dioxus desktop); a pure-GPU native
  renderer (Blitz) is not production-ready yet.
- F24 in TCP mode is emitted but inert — nothing registers it as a hotkey.
