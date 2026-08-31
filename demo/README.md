# Windows feasibility demo

> English | [简体中文](README.zh.md)

A proof-of-concept for the `enable-touchpad` idea on Windows:

- **kanata** owns the keyboard: holding `CapsLock` activates a `mouse` layer
  (`Q`/`W`/`E` → mouse left/middle/right buttons, `Left Alt` → `CapsLock`)
  and holds the inert `Ctrl+Win+F24` combo for as long as CapsLock is held.
- **the companion app** (`src/bin/enable-touchpad`, built with Dioxus) reacts
  to the layer signal, **enables the touchpad while CapsLock is held and
  disables it on release**, shows a click-through indicator at the mouse
  position, and provides a tray icon plus a minimal settings page.

```
hold CapsLock ──▶ kanata layer "mouse" ──▶ signal ──▶ Dioxus app
   (Q/W/E = mouse buttons)                 TCP LayerChange      │
                                           or F24 press/release ├─▶ enable touchpad
                                                                ├─▶ indicator at mouse
release CapsLock ──▶ layer restored ────────────────────────────┴─▶ disable touchpad
```

## Layout

| Path | Purpose |
|------|---------|
| `kanata/enable-touchpad.kbd` | kanata layer config (validated with `kanata --check`) |
| `../src/bin/enable-touchpad/` | the app: `main` / `app` (UI) / `config` / `signal` / `touchpad` / `tray` |

The app compiles only for Windows; other targets build a stub so the
repository's Linux gates stay green.

## Windows setup

1. **Install kanata and the Interception driver** — grab a kanata release from
   <https://github.com/jtroo/kanata/releases>, then install the
   [Interception driver](https://github.com/oblitum/Interception) it requires
   on Windows and reboot.
2. **Start kanata** (pick one signal source):
   - TCP mode: `kanata -c enable-touchpad.kbd -p 5829`
   - F24 mode: `kanata -c enable-touchpad.kbd`
3. **Build and run the app as administrator** (device enable/disable is a
   system-level operation):

   ```powershell
   cargo run --bin enable-touchpad
   ```

4. Hold `CapsLock` → the touchpad turns on, a blue pill appears next to the
   mouse cursor, and `Q`/`W`/`E` act as mouse buttons. Release → everything
   is restored and the touchpad turns off.

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
