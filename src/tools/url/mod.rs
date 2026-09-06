//! URL percent-encoding / decoding.

use crate::tools::util::hex_val;

/// Percent-encode spaces as `+`; every non-ASCII byte becomes `%XX`.
pub fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if c == ' ' {
            out.push('+');
        } else {
            let mut buf = vec![0u8; c.len_utf8()];
            c.encode_utf8(&mut buf);
            for &b in buf.as_slice() {
                if is_unreserved(b) {
                    out.push(b as char);
                } else {
                    out.push('%');
                    out.push(((b >> 4) as u8 + b'0') as char);
                    out.push(((b & 0x0f) as u8 + b'0') as char);
                }
            }
        }
    }
    out
}

/// True for URL "unreserved" bytes (RFC 3986): A-Z a-z 0-9 - _ . ~.
fn is_unreserved(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b)
        || (b'a'..=b'z').contains(&b)
        || (b'0'..=b'9').contains(&b)
        || matches!(b, b'-' | b'_' | b'.' | b'~')
}

/// Percent-decode `+` and `%XX` sequences.
pub fn url_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        match input.as_bytes()[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < input.len() => {
                let h = hex_val(input.as_bytes()[i + 1]);
                let l = hex_val(input.as_bytes()[i + 2]);
                if let (Some(h), Some(l)) = (h, l) {
                    out.push((h << 4) | l);
                    i += 3;
                } else {
                    out.push(input.as_bytes()[i]);
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_basic() {
        assert_eq!(url_encode("a b&c"), "a+b%26c");
    }

    #[test]
    fn url_decode_basic() {
        assert_eq!(url_decode("a+b%26c"), "a b&c");
    }
}
