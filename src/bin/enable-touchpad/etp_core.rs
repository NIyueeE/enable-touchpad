//! Pure, cross-platform core of enable-touchpad: the configuration model,
//! the keyboard-code allowlist for layer bindings, display labels, and the
//! kanata config generator.
//!
//! Compiled on every target so `cargo test` exercises the real logic on the
//! Linux host; on Windows the binary modules are its only consumers.
//!
//! Binding values are W3C `KeyboardEvent.code` strings (`"KeyQ"`,
//! `"AltLeft"`, `"F5"`). kanata's parser accepts these verbatim (see
//! `str_to_oscode` in kanata-parser v1.11.0, `parser/src/keys/mod.rs`), so
//! captured codes go straight into the generated config. The legacy short
//! forms from earlier versions (`"q"`, `"lalt"`) stay valid and loadable.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Binding value meaning "nothing assigned to this action".
pub const KEY_NONE: &str = "none";

/// The physical `CapsLock` key is hard-wired as the layer hold key, so it can
/// never be a binding inside the layer.
pub const HOLD_KEY: &str = "CapsLock";

/// `Escape` cancels an in-progress key capture in the UI and is not bindable.
pub const CANCEL_KEY: &str = "Escape";

/// Bindable keyboard codes with their display labels.
///
/// Every entry was cross-checked against kanata v1.11.0's
/// `str_to_oscode` name table and its Windows scancode conversion
/// (`scancode_to_usvk.rs`). Deliberately absent: `Escape` (capture cancel),
/// `CapsLock` (reserved as the hold key), and media/browser keys whose web
/// code names differ from kanata's.
pub const SUPPORTED_CODES: &[(&str, &str)] = &[
    ("KeyA", "A"),
    ("KeyB", "B"),
    ("KeyC", "C"),
    ("KeyD", "D"),
    ("KeyE", "E"),
    ("KeyF", "F"),
    ("KeyG", "G"),
    ("KeyH", "H"),
    ("KeyI", "I"),
    ("KeyJ", "J"),
    ("KeyK", "K"),
    ("KeyL", "L"),
    ("KeyM", "M"),
    ("KeyN", "N"),
    ("KeyO", "O"),
    ("KeyP", "P"),
    ("KeyQ", "Q"),
    ("KeyR", "R"),
    ("KeyS", "S"),
    ("KeyT", "T"),
    ("KeyU", "U"),
    ("KeyV", "V"),
    ("KeyW", "W"),
    ("KeyX", "X"),
    ("KeyY", "Y"),
    ("KeyZ", "Z"),
    ("Digit0", "0"),
    ("Digit1", "1"),
    ("Digit2", "2"),
    ("Digit3", "3"),
    ("Digit4", "4"),
    ("Digit5", "5"),
    ("Digit6", "6"),
    ("Digit7", "7"),
    ("Digit8", "8"),
    ("Digit9", "9"),
    ("F1", "F1"),
    ("F2", "F2"),
    ("F3", "F3"),
    ("F4", "F4"),
    ("F5", "F5"),
    ("F6", "F6"),
    ("F7", "F7"),
    ("F8", "F8"),
    ("F9", "F9"),
    ("F10", "F10"),
    ("F11", "F11"),
    ("F12", "F12"),
    ("F13", "F13"),
    ("F14", "F14"),
    ("F15", "F15"),
    ("F16", "F16"),
    ("F17", "F17"),
    ("F18", "F18"),
    ("F19", "F19"),
    ("F20", "F20"),
    ("F21", "F21"),
    ("F22", "F22"),
    ("F23", "F23"),
    ("F24", "F24"),
    ("Backquote", "`"),
    ("Minus", "-"),
    ("Equal", "="),
    ("BracketLeft", "["),
    ("BracketRight", "]"),
    ("Semicolon", ";"),
    ("Quote", "'"),
    ("Backslash", "\\"),
    ("Comma", ","),
    ("Period", "."),
    ("Slash", "/"),
    ("IntlBackslash", "\\ ISO"),
    ("Space", "Space"),
    ("Tab", "Tab"),
    ("Enter", "Enter"),
    ("Backspace", "Bksp"),
    ("Delete", "Del"),
    ("Insert", "Ins"),
    ("Home", "Home"),
    ("End", "End"),
    ("PageUp", "PgUp"),
    ("PageDown", "PgDn"),
    ("ArrowUp", "↑"),
    ("ArrowDown", "↓"),
    ("ArrowLeft", "←"),
    ("ArrowRight", "→"),
    ("ShiftLeft", "LShift"),
    ("ShiftRight", "RShift"),
    ("ControlLeft", "LCtrl"),
    ("ControlRight", "RCtrl"),
    ("AltLeft", "LAlt"),
    ("AltRight", "RAlt"),
    ("MetaLeft", "LWin"),
    ("MetaRight", "RWin"),
    ("ContextMenu", "Menu"),
    ("Numpad0", "Num0"),
    ("Numpad1", "Num1"),
    ("Numpad2", "Num2"),
    ("Numpad3", "Num3"),
    ("Numpad4", "Num4"),
    ("Numpad5", "Num5"),
    ("Numpad6", "Num6"),
    ("Numpad7", "Num7"),
    ("Numpad8", "Num8"),
    ("Numpad9", "Num9"),
    ("NumpadAdd", "Num+"),
    ("NumpadSubtract", "Num-"),
    ("NumpadMultiply", "Num*"),
    ("NumpadDivide", "Num/"),
    ("NumpadDecimal", "Num."),
    ("NumpadEnter", "NumEnter"),
    ("NumLock", "NumLock"),
    ("ScrollLock", "ScrLk"),
    ("Pause", "Pause"),
];

/// Short kanata key names from earlier config versions, for display only.
const LEGACY_LABELS: &[(&str, &str)] = &[
    ("q", "Q"),
    ("w", "W"),
    ("e", "E"),
    ("r", "R"),
    ("f", "F"),
    ("v", "V"),
    ("lalt", "LAlt"),
    ("ralt", "RAlt"),
    ("lctl", "LCtrl"),
    ("rctl", "RCtrl"),
    ("lshift", "LShift"),
    ("rshift", "RShift"),
    ("lmeta", "LWin"),
    ("rmeta", "RWin"),
    ("caps", "Caps"),
    ("spc", "Space"),
    ("tab", "Tab"),
    ("enter", "Enter"),
    ("bspc", "Bksp"),
    ("del", "Del"),
    ("ins", "Ins"),
    ("home", "Home"),
    ("end", "End"),
    ("pgup", "PgUp"),
    ("pgdn", "PgDn"),
    ("up", "↑"),
    ("down", "↓"),
    ("left", "←"),
    ("right", "→"),
    ("grave", "`"),
    ("min", "-"),
    ("eql", "="),
    ("lbrc", "["),
    ("rbrc", "]"),
    ("scln", ";"),
    ("apo", "'"),
    ("bksl", "\\"),
    ("comm", ","),
    ("dot", "."),
    ("slash", "/"),
    ("nlck", "NumLock"),
    ("scrlck", "ScrLk"),
    ("pause", "Pause"),
];

/// Contents of `%APPDATA%\enable-touchpad\config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Master switch: `false` makes `CapsLock` behave stock.
    pub feature_enabled: bool,
    /// Layer key that performs the left mouse click.
    pub left_click_key: String,
    /// Layer key that performs the middle mouse click.
    pub middle_click_key: String,
    /// Layer key that performs the right mouse click.
    pub right_click_key: String,
    /// Layer key that acts as `CapsLock`.
    pub capslock_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            feature_enabled: true,
            left_click_key: "q".to_string(),
            middle_click_key: "w".to_string(),
            right_click_key: "e".to_string(),
            capslock_key: "lalt".to_string(),
        }
    }
}

impl AppConfig {
    /// Path of the config file, or `None` when `APPDATA` is unset.
    pub fn path() -> Option<PathBuf> {
        Some(app_dir().ok()?.join("config.json"))
    }

    /// Load the config, falling back to defaults on any problem.
    pub fn load() -> Self {
        let parsed = Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|text| serde_json::from_str(&text).ok());
        parsed.unwrap_or_default()
    }

    /// Persist the config to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path().ok_or_else(|| "APPDATA is not set".to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// Application data directory, created on demand.
pub fn app_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("APPDATA").ok_or("APPDATA is not set")?;
    let dir = PathBuf::from(base).join("enable-touchpad");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Whether a captured keyboard code may be assigned to a layer action.
pub fn is_bindable(code: &str) -> bool {
    SUPPORTED_CODES.iter().any(|(c, _)| *c == code)
}

/// Display label for a binding value: W3C code, legacy short name, or raw
/// fallback so unknown values stay visible instead of disappearing.
pub fn key_label(value: &str) -> String {
    if value == KEY_NONE {
        return "无".to_string();
    }
    if let Some((_, label)) = SUPPORTED_CODES.iter().find(|(c, _)| *c == value) {
        return (*label).to_string();
    }
    if let Some((_, label)) = LEGACY_LABELS.iter().find(|(c, _)| *c == value) {
        return (*label).to_string();
    }
    value.to_string()
}

/// The kanata action for a `mouse`-layer slot, given the action-to-key
/// mapping. Unassigned slots emit `XX` (blocked). First claim wins when two
/// actions share a key.
pub fn layer_slot_action(
    slot: &str,
    left_click_key: &str,
    middle_click_key: &str,
    right_click_key: &str,
    capslock_key: &str,
) -> &'static str {
    if slot == capslock_key {
        "caps"
    } else if slot == left_click_key {
        "mlft"
    } else if slot == middle_click_key {
        "mmid"
    } else if slot == right_click_key {
        "mrgt"
    } else {
        "XX"
    }
}

/// Ordered, de-duplicated list of layer keys to place in `defsrc`.
///
/// `CapsLock` never appears: the physical key owns that defsrc slot as the
/// layer hold key, so binding it to an action would be a silent no-op.
fn layer_keys(cfg: &AppConfig) -> Vec<&str> {
    let mut keys: Vec<&str> = Vec::new();
    for k in [
        cfg.left_click_key.as_str(),
        cfg.middle_click_key.as_str(),
        cfg.right_click_key.as_str(),
        cfg.capslock_key.as_str(),
    ] {
        if k != KEY_NONE && k != HOLD_KEY && !keys.contains(&k) {
            keys.push(k);
        }
    }
    keys
}

/// Generate the kanata layer config from the app configuration.
///
/// Toggle semantics (per the user's driver): on `CapsLock` press the
/// Ctrl+Win+F24 chord is tapped (all three keys down, released in reverse —
/// a real chord, never a lone Win tap) and the `mouse` layer activates; on
/// `CapsLock` release the chord is tapped again via `on-release-fakekey`.
/// The chord tap is what makes the system's touchpad driver flip the
/// touchpad on and off; this app never touches devices.
///
/// Layer keys are emitted in `defsrc` once each (duplicate action-to-key
/// assignments collapse to the first claim), and every slot passes its own
/// key through unchanged on the base layer.
pub fn generate_config_text(cfg: &AppConfig) -> String {
    let keys = layer_keys(cfg);
    let caps_slot = if cfg.feature_enabled {
        r"(multi
    (layer-while-held mouse)
    (macro C-M-f24)
    (on-release-fakekey release-tap tap))"
            .to_string()
    } else {
        "caps".to_string()
    };
    let slot = |k: &str| {
        layer_slot_action(
            k,
            &cfg.left_click_key,
            &cfg.middle_click_key,
            &cfg.right_click_key,
            &cfg.capslock_key,
        )
    };
    let base_slots: Vec<String> = std::iter::once(caps_slot)
        .chain(keys.iter().map(|k| (*k).to_string()))
        .collect();
    let mouse_slots: Vec<String> = std::iter::once("XX".to_string())
        .chain(keys.iter().map(|k| slot(k).to_string()))
        .collect();
    let defsrc = std::iter::once("caps")
        .chain(keys.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    let enumerated = |slots: &[String]| {
        slots
            .iter()
            .map(|s| format!("  {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        ";; generated by enable-touchpad — edits are overwritten by the app\n\
         (defcfg process-unmapped-keys no)\n\
         \n\
         (defsrc {defsrc})\n\
         \n\
         (deffakekeys\n  release-tap (macro C-M-f24))\n\
         \n\
         (deflayer base\n{}\n)\n\
         \n\
         (deflayer mouse\n{}\n)\n",
        enumerated(&base_slots),
        enumerated(&mouse_slots)
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, KEY_NONE, SUPPORTED_CODES, generate_config_text, is_bindable, key_label,
    };

    fn cfg(feature_enabled: bool, left: &str, middle: &str, right: &str, caps: &str) -> AppConfig {
        AppConfig {
            feature_enabled,
            left_click_key: left.to_string(),
            middle_click_key: middle.to_string(),
            right_click_key: right.to_string(),
            capslock_key: caps.to_string(),
        }
    }

    #[test]
    fn every_supported_code_has_a_label() {
        for (code, label) in SUPPORTED_CODES {
            assert!(!label.is_empty(), "empty label for {code}");
            assert_eq!(key_label(code), *label, "label mismatch for {code}");
        }
    }

    #[test]
    fn legacy_values_keep_labels() {
        assert_eq!(key_label("q"), "Q");
        assert_eq!(key_label("lalt"), "LAlt");
        assert_eq!(key_label(KEY_NONE), "无");
        assert_eq!(key_label("KeyQ"), "Q");
        assert_eq!(key_label("AltLeft"), "LAlt");
        assert_eq!(key_label("ArrowUp"), "↑");
    }

    #[test]
    fn reserved_keys_are_not_bindable() {
        assert!(!is_bindable("CapsLock"));
        assert!(!is_bindable("Escape"));
        assert!(!is_bindable("PrintScreen"));
        assert!(is_bindable("KeyQ"));
        assert!(is_bindable("AltLeft"));
        assert!(is_bindable("F5"));
        assert!(is_bindable("NumpadEnter"));
    }

    #[test]
    fn generator_dedups_keys_and_keeps_first_claim() {
        let cfg = cfg(true, "KeyF", "KeyF", KEY_NONE, "KeyJ");
        let text = generate_config_text(&cfg);
        assert!(text.contains("(defsrc caps KeyF KeyJ)"), "{text}");
        assert!(text.contains("(deflayer base\n  (multi\n    (layer-while-held mouse)\n    (macro C-M-f24)\n    (on-release-fakekey release-tap tap))\n  KeyF\n  KeyJ\n)"), "{text}");
        assert!(
            text.contains("(deflayer mouse\n  XX\n  mlft\n  caps\n)"),
            "{text}"
        );
    }

    #[test]
    fn generator_ignores_hold_key_collision() {
        let cfg = cfg(true, "CapsLock", "KeyW", "KeyE", "lalt");
        let text = generate_config_text(&cfg);
        assert!(text.contains("(defsrc caps KeyW KeyE lalt)"), "{text}");
        assert!(
            text.contains("(deflayer mouse\n  XX\n  mmid\n  mrgt\n  caps\n)"),
            "{text}"
        );
    }

    #[test]
    fn generator_none_only_emits_caps() {
        let cfg = cfg(true, KEY_NONE, KEY_NONE, KEY_NONE, KEY_NONE);
        let text = generate_config_text(&cfg);
        assert!(text.contains("(defsrc caps)"), "{text}");
        assert!(text.contains("(deflayer mouse\n  XX\n)"), "{text}");
    }

    #[test]
    fn generator_disabled_feature_passes_caps_through() {
        let cfg = cfg(false, "q", "w", "e", "lalt");
        let text = generate_config_text(&cfg);
        assert!(
            text.contains("(deflayer base\n  caps\n  q\n  w\n  e\n  lalt\n)"),
            "{text}"
        );
    }
}
