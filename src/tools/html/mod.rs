//! HTML entity encoding / decoding.

/// HTML-encode: escape `&`, `<`, `>`, `"`, `'`.
pub fn html_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// HTML-decode the common named entities.
pub fn html_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if let Some((entity, s)) = named_entity_at(input, i) {
            out.push_str(&entity);
            i += s;
        } else {
            out.push(input.chars().nth(i).unwrap_or('?'));
            i += 1;
        }
    }
    out
}

/// Look up a named HTML entity starting at `i`; returns (decoded, consumed).
fn named_entity_at(input: &str, i: usize) -> Option<(&str, usize)> {
    let rest = &input[i..];
    let table = [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
    ];
    for (entity, dec) in table.iter() {
        if rest.starts_with(entity) {
            return Some((dec, entity.len()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_encode_basic() {
        assert_eq!(html_encode("a<b>\""), "a&lt;b&gt;&quot;");
    }

    #[test]
    fn html_decode_basic() {
        assert_eq!(html_decode("a&lt;b&gt;\""), "a<b>\"");
    }
}
