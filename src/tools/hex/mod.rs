//! Hex encoding / decoding.

use crate::utils::hex_val;

/// Lowercase hex-encode a UTF-8 string.
pub fn hex_encode(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Lowercase hex-decode a string.
pub fn hex_decode(input: &str) -> String {
    let bytes: Vec<u8> = input
        .as_bytes()
        .chunks_exact(2)
        .filter(|c| c.len() == 2)
        .map(|c| {
            let h = hex_val(c[0]).unwrap_or(0);
            let l = hex_val(c[1]).unwrap_or(0);
            (h << 4) | l
        })
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode_basic() {
        assert_eq!(hex_encode("AB"), "4142");
    }

    #[test]
    fn hex_decode_basic() {
        assert_eq!(hex_decode("4142"), "AB");
    }
}
