//! Base64 encoding / decoding (standard alphabet, no padding).

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base64-encode a UTF-8 string (standard alphabet, no padding).
pub fn base64_encode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (if chunk.len() > 1 { chunk[1] as u32 } else { 0 }) << 8
            | (if chunk.len() > 2 { chunk[2] as u32 } else { 0 });
        out.push(ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 0x3f) as usize] as char);
        }
    }
    out
}

/// Base64-decode a string (standard alphabet, no padding).
pub fn base64_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0u32;
    for c in input.chars() {
        let v = val(c).unwrap_or(0);
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            let byte = (acc >> (bits - 8)) & 0xFF;
            out.push(byte as u8);
            bits -= 8;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn val(c: char) -> Option<u32> {
    match c {
        'A'..='Z' => Some((c as u8 - b'A') as u32),
        'a'..='z' => Some((c as u8 - b'a' + 26) as u32),
        '0'..='9' => Some((c as u8 - b'0' + 52) as u32),
        '+' => Some(62),
        '/' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_basic() {
        assert_eq!(base64_encode("Man"), "TWFu");
    }

    #[test]
    fn base64_decode_basic() {
        assert_eq!(base64_decode("TWFu"), "Man");
    }
}
