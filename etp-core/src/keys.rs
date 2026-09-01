//! Bindable-key allowlist and display labels.

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

/// Whether a captured keyboard code may be assigned to a layer action.
#[must_use]
pub fn is_bindable(code: &str) -> bool {
    SUPPORTED_CODES.iter().any(|(c, _)| *c == code)
}

/// Display label for a binding value: W3C code, legacy short name, or raw
/// fallback so unknown values stay visible instead of disappearing.
#[must_use]
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

#[cfg(test)]
mod tests {
    use super::{KEY_NONE, SUPPORTED_CODES, is_bindable, key_label};

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
}
