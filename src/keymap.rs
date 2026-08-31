use eframe::egui::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeySpec {
    pub id: &'static str,
    pub label: &'static str,
    pub speech: &'static str,
}

const fn spec(id: &'static str, label: &'static str, speech: &'static str) -> KeySpec {
    KeySpec { id, label, speech }
}

macro_rules! lookup {
    ($key:expr; $(($variant:ident, $id:literal, $label:literal, $speech:literal)),+ $(,)?) => {{
        match $key {
            $(Key::$variant => Some(spec($id, $label, $speech)),)+
            _ => None,
        }
    }};
}

pub fn spec_for_key(key: Key) -> Option<KeySpec> {
    if let Some(spec) = lookup!(
        key;
        (Num0, "num0", "0", "Zero"),
        (Num1, "num1", "1", "One"),
        (Num2, "num2", "2", "Two"),
        (Num3, "num3", "3", "Three"),
        (Num4, "num4", "4", "Four"),
        (Num5, "num5", "5", "Five"),
        (Num6, "num6", "6", "Six"),
        (Num7, "num7", "7", "Seven"),
        (Num8, "num8", "8", "Eight"),
        (Num9, "num9", "9", "Nine"),
    ) {
        return Some(spec);
    }
    if let Some(spec) = lookup!(
        key;
        (A, "a", "A", "A"),
        (B, "b", "B", "B"),
        (C, "c", "C", "C"),
        (D, "d", "D", "D"),
        (E, "e", "E", "E"),
        (F, "f", "F", "F"),
        (G, "g", "G", "G"),
        (H, "h", "H", "H"),
        (I, "i", "I", "I"),
        (J, "j", "J", "J"),
        (K, "k", "K", "K"),
        (L, "l", "L", "L"),
        (M, "m", "M", "M"),
        (N, "n", "N", "N"),
        (O, "o", "O", "O"),
        (P, "p", "P", "P"),
        (Q, "q", "Q", "Q"),
        (R, "r", "R", "R"),
        (S, "s", "S", "S"),
        (T, "t", "T", "T"),
        (U, "u", "U", "U"),
        (V, "v", "V", "V"),
        (W, "w", "W", "W"),
        (X, "x", "X", "X"),
        (Y, "y", "Y", "Y"),
        (Z, "z", "Z", "Z"),
    ) {
        return Some(spec);
    }
    if let Some(spec) = lookup!(
        key;
        (F1, "f1", "F1", "F one"),
        (F2, "f2", "F2", "F two"),
        (F3, "f3", "F3", "F three"),
        (F4, "f4", "F4", "F four"),
        (F5, "f5", "F5", "F five"),
        (F6, "f6", "F6", "F six"),
        (F7, "f7", "F7", "F seven"),
        (F8, "f8", "F8", "F eight"),
        (F9, "f9", "F9", "F nine"),
        (F10, "f10", "F10", "F ten"),
        (F11, "f11", "F11", "F eleven"),
        (F12, "f12", "F12", "F twelve"),
        (F13, "f13", "F13", "F thirteen"),
        (F14, "f14", "F14", "F fourteen"),
        (F15, "f15", "F15", "F fifteen"),
        (F16, "f16", "F16", "F sixteen"),
        (F17, "f17", "F17", "F seventeen"),
        (F18, "f18", "F18", "F eighteen"),
        (F19, "f19", "F19", "F nineteen"),
        (F20, "f20", "F20", "F twenty"),
        (F21, "f21", "F21", "F twenty one"),
        (F22, "f22", "F22", "F twenty two"),
        (F23, "f23", "F23", "F twenty three"),
        (F24, "f24", "F24", "F twenty four"),
        (F25, "f25", "F25", "F twenty five"),
        (F26, "f26", "F26", "F twenty six"),
        (F27, "f27", "F27", "F twenty seven"),
        (F28, "f28", "F28", "F twenty eight"),
        (F29, "f29", "F29", "F twenty nine"),
        (F30, "f30", "F30", "F thirty"),
        (F31, "f31", "F31", "F thirty one"),
        (F32, "f32", "F32", "F thirty two"),
        (F33, "f33", "F33", "F thirty three"),
        (F34, "f34", "F34", "F thirty four"),
        (F35, "f35", "F35", "F thirty five"),
    ) {
        return Some(spec);
    }

    Some(match key {
        Key::Escape => spec("escape", "Esc", "Escape"),
        Key::Tab => spec("tab", "Tab", "Tab"),
        Key::Backspace => spec("backspace", "Backspace", "Backspace"),
        Key::Enter => spec("enter", "Enter", "Enter"),
        Key::Space => spec("space", "Space", "Space"),
        Key::Insert => spec("insert", "Insert", "Insert"),
        Key::Delete => spec("delete", "Delete", "Delete"),
        Key::Home => spec("home", "Home", "Home"),
        Key::End => spec("end", "End", "End"),
        Key::PageUp => spec("page_up", "Page Up", "Page up"),
        Key::PageDown => spec("page_down", "Page Down", "Page down"),
        Key::Copy => spec("copy", "Copy", "Copy"),
        Key::Cut => spec("cut", "Cut", "Cut"),
        Key::Paste => spec("paste", "Paste", "Paste"),
        Key::Colon => spec("colon", ":", "Colon"),
        Key::Comma => spec("comma", ",", "Comma"),
        Key::Backslash => spec("backslash", "\\", "Backslash"),
        Key::Slash => spec("slash", "/", "Slash"),
        Key::OpenBracket => spec("open_bracket", "[", "Open bracket"),
        Key::CloseBracket => spec("close_bracket", "]", "Close bracket"),
        Key::Pipe => spec("pipe", "|", "Pipe"),
        Key::Questionmark => spec("questionmark", "?", "Question mark"),
        Key::Semicolon => spec("semicolon", ";", "Semicolon"),
        Key::Quote => spec("quote", "'", "Quote"),
        Key::Backtick => spec("backtick", "`", "Backtick"),
        Key::Minus => spec("minus", "-", "Minus"),
        Key::Period => spec("period", ".", "Period"),
        Key::Exclamationmark => spec("exclamationmark", "!", "Exclamation mark"),
        Key::OpenCurlyBracket => spec("open_curly_bracket", "{", "Open curly bracket"),
        Key::CloseCurlyBracket => spec("close_curly_bracket", "}", "Close curly bracket"),
        Key::Plus => spec("plus", "+", "Plus"),
        Key::Equals => spec("equals", "=", "Equals"),
        Key::ArrowUp => spec("arrow_up", "Arrow Up", "Arrow up"),
        Key::ArrowDown => spec("arrow_down", "Arrow Down", "Arrow down"),
        Key::ArrowLeft => spec("arrow_left", "Arrow Left", "Arrow left"),
        Key::ArrowRight => spec("arrow_right", "Arrow Right", "Arrow right"),
        Key::BrowserBack => spec("browser_back", "Back", "Back"),
        _ => return None,
    })
}

/// Resolves text produced by a keyboard layout, including symbols that require
/// Shift (for example `<` from Shift+Comma). Text events represent the final
/// character, so they take precedence over the physical modifier/key events.
pub fn spec_for_text(text: &str) -> Option<KeySpec> {
    let mut chars = text.chars();
    let character = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(match character {
        '0' => spec("num0", "0", "Zero"),
        '1' => spec("num1", "1", "One"),
        '2' => spec("num2", "2", "Two"),
        '3' => spec("num3", "3", "Three"),
        '4' => spec("num4", "4", "Four"),
        '5' => spec("num5", "5", "Five"),
        '6' => spec("num6", "6", "Six"),
        '7' => spec("num7", "7", "Seven"),
        '8' => spec("num8", "8", "Eight"),
        '9' => spec("num9", "9", "Nine"),
        'a' | 'A' => spec("a", "A", "A"),
        'b' | 'B' => spec("b", "B", "B"),
        'c' | 'C' => spec("c", "C", "C"),
        'd' | 'D' => spec("d", "D", "D"),
        'e' | 'E' => spec("e", "E", "E"),
        'f' | 'F' => spec("f", "F", "F"),
        'g' | 'G' => spec("g", "G", "G"),
        'h' | 'H' => spec("h", "H", "H"),
        'i' | 'I' => spec("i", "I", "I"),
        'j' | 'J' => spec("j", "J", "J"),
        'k' | 'K' => spec("k", "K", "K"),
        'l' | 'L' => spec("l", "L", "L"),
        'm' | 'M' => spec("m", "M", "M"),
        'n' | 'N' => spec("n", "N", "N"),
        'o' | 'O' => spec("o", "O", "O"),
        'p' | 'P' => spec("p", "P", "P"),
        'q' | 'Q' => spec("q", "Q", "Q"),
        'r' | 'R' => spec("r", "R", "R"),
        's' | 'S' => spec("s", "S", "S"),
        't' | 'T' => spec("t", "T", "T"),
        'u' | 'U' => spec("u", "U", "U"),
        'v' | 'V' => spec("v", "V", "V"),
        'w' | 'W' => spec("w", "W", "W"),
        'x' | 'X' => spec("x", "X", "X"),
        'y' | 'Y' => spec("y", "Y", "Y"),
        'z' | 'Z' => spec("z", "Z", "Z"),
        '!' => spec("exclamationmark", "!", "Exclamation mark"),
        '@' => spec("at", "@", "At sign"),
        '#' => spec("hash", "#", "Hash"),
        '$' => spec("dollar", "$", "Dollar sign"),
        '%' => spec("percent", "%", "Percent"),
        '^' => spec("caret", "^", "Caret"),
        '&' => spec("ampersand", "&", "Ampersand"),
        '*' => spec("asterisk", "*", "Asterisk"),
        '(' => spec("open_paren", "(", "Left parenthesis"),
        ')' => spec("close_paren", ")", "Right parenthesis"),
        '_' => spec("underscore", "_", "Underscore"),
        '+' => spec("plus", "+", "Plus"),
        '{' => spec("open_curly_bracket", "{", "Open curly bracket"),
        '}' => spec("close_curly_bracket", "}", "Close curly bracket"),
        '|' => spec("pipe", "|", "Pipe"),
        ':' => spec("colon", ":", "Colon"),
        '"' => spec("quote", "\"", "Quote"),
        '~' => spec("tilde", "~", "Tilde"),
        '<' => spec("less_than", "<", "Less than"),
        '>' => spec("greater_than", ">", "Greater than"),
        '?' => spec("questionmark", "?", "Question mark"),
        _ => return None,
    })
}

/// Fallback for integrations that report the physical key and modifiers but
/// omit the corresponding text event.
pub fn spec_for_shifted_key(key: Key) -> Option<KeySpec> {
    let text = match key {
        Key::Num0 => ")",
        Key::Num1 => "!",
        Key::Num2 => "@",
        Key::Num3 => "#",
        Key::Num4 => "$",
        Key::Num5 => "%",
        Key::Num6 => "^",
        Key::Num7 => "&",
        Key::Num8 => "*",
        Key::Num9 => "(",
        Key::Minus => "_",
        Key::Equals => "+",
        Key::OpenBracket => "{",
        Key::CloseBracket => "}",
        Key::Backslash => "|",
        Key::Semicolon => ":",
        Key::Quote => "\"",
        Key::Backtick => "~",
        Key::Comma => "<",
        Key::Period => ">",
        Key::Slash => "?",
        _ => return None,
    };
    spec_for_text(text)
}

pub fn modifier_spec(modifier: Modifier) -> KeySpec {
    match modifier {
        Modifier::Shift => spec("shift", "Shift", "Shift"),
        Modifier::Control => spec("control", "Ctrl", "Control"),
        Modifier::Alt => spec("alt", "Alt", "Alt"),
        Modifier::Meta => spec("meta", "Command", "Command"),
    }
}

#[derive(Clone, Copy)]
pub enum Modifier {
    Shift,
    Control,
    Alt,
    Meta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_escape_to_spoken_escape() {
        let key = spec_for_key(Key::Escape).unwrap();
        assert_eq!(key.label, "Esc");
        assert_eq!(key.speech, "Escape");
    }

    #[test]
    fn maps_number_and_modifier() {
        assert_eq!(spec_for_key(Key::Num7).unwrap().speech, "Seven");
        assert_eq!(modifier_spec(Modifier::Control).speech, "Control");
        assert_eq!(modifier_spec(Modifier::Meta).label, "Command");
        assert_eq!(modifier_spec(Modifier::Meta).speech, "Command");
    }

    #[test]
    fn every_egui_key_has_a_display_spec() {
        for key in Key::ALL {
            assert!(
                spec_for_key(*key).is_some(),
                "missing key mapping for {key:?}"
            );
        }
    }

    #[test]
    fn resolves_shifted_symbol_to_final_character() {
        let key = spec_for_text("<").unwrap();
        assert_eq!(key.label, "<");
        assert_eq!(key.speech, "Less than");
        assert_eq!(
            spec_for_shifted_key(Key::Comma).unwrap().speech,
            "Less than"
        );
    }
}
