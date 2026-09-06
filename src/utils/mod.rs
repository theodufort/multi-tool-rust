//! Small helpers shared by more than one tool.

/// Hex value of a single ASCII hex digit (`0-9`, `a-f`, `A-F`).
pub fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Uppercase the first character of `w`, leaving the rest untouched.
pub fn uppercase_first(w: &str) -> String {
    let mut chars = w.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
