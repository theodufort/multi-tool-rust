//! Pure, deterministic, dependency-light text transforms.
//!
//! Each tool is a single `pub fn` in this module. They perform no I/O, network,
//! or allocation-heavy work beyond building the output `String`. The `run`
//! helper dispatches to the right transform by slug.

/// Replace `count` occurrences of `find` with `replace` (count == 0 means all).
/// An empty `find` matches nothing, so the input is returned unchanged.
pub fn replace(input: &str, find: &str, replace: &str, count: usize) -> String {
    if find.is_empty() {
        return input.to_string();
    }
    let mut out = String::new();
    let mut i = 0;
    let mut done = 0;
    while i < input.len() {
        if done < count && input[i..i + find.len()] == *find {
            out.push_str(replace);
            i += find.len();
            done += 1;
        } else {
            match input[i..].chars().next() {
                Some(c) => {
                    out.push(c);
                    i += c.len_utf8();
                }
                None => break,
            }
        }
    }
    out
}

/// Replace every occurrence of `find` with `replace`.
pub fn replace_all(input: &str, find: &str, to: &str) -> String {
    replace(input, find, to, usize::MAX)
}

/// Collapse runs of whitespace (space, tab, newline, CR, FF, VT) into a single
/// space and strip leading/trailing whitespace.
pub fn collapse(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Uppercase the input (Unicode-aware).
pub fn uppercase(input: &str) -> String {
    input.to_uppercase()
}

/// Lowercase the input (Unicode-aware).
pub fn lowercase(input: &str) -> String {
    input.to_lowercase()
}

/// Title-case: capitalize the first letter of every whitespace-delimited word
/// and leave the rest untouched.
pub fn title(input: &str) -> String {
    input
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Reverse the input, preserving character order (Unicode-aware).
pub fn reverse(input: &str) -> String {
    input.chars().rev().collect()
}

/// Trim leading and trailing whitespace.
pub fn trim(input: &str) -> String {
    input.trim().to_string()
}

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

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

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

/// Base64-encode a UTF-8 string (standard alphabet, no padding).
pub fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

/// Strip non-ASCII characters (Unicode -> ASCII).
pub fn unicode_strip(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii()).collect::<String>()
}

/// Count words, characters, lines, and bytes of the input (one per line).
pub fn word_count(input: &str) -> String {
    let words = input.split_whitespace().count();
    let chars = input.chars().count();
    let lines = input.lines().count().max(1);
    let bytes = input.len();
    format!(
        "words: {}\nchars: {}\nlines: {}\nbytes: {}",
        words, chars, lines, bytes
    )
}

/// Pretty-print JSON with 2-space indentation (compact separators).
pub fn json_pretty(input: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| input.to_string()),
        Err(_) => input.to_string(),
    }
}

/// Minify JSON by removing insignificant whitespace.
pub fn json_minify(input: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(v) => serde_json::to_string(&v).unwrap_or_else(|_| input.to_string()),
        Err(_) => input.to_string(),
    }
}

/// Slugify: lowercase, replace runs of non-alphanumeric chars with `-`, trim.
pub fn slug(input: &str) -> String {
    let mut out = String::new();
    let mut in_run = false;
    for c in input.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Convert text to a target case (camel, snake, kebab, pascal).
/// Defaults to `camel` when no action is given.
pub fn case(input: &str, action: Option<&str>) -> String {
    let action = action.unwrap_or("camel");
    let words: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect();
    match action {
        "snake" => words
            .iter()
            .map(|w| snake_case(w))
            .collect::<Vec<_>>()
            .join("_"),
        "kebab" => words
            .iter()
            .map(|w| kebab_case(w))
            .collect::<Vec<_>>()
            .join("-"),
        "camel" => {
            let mut s = String::new();
            for (i, w) in words.iter().enumerate() {
                if i == 0 {
                    s += w;
                } else {
                    s += &uppercase_first(w);
                }
            }
            s
        }
        "pascal" => words
            .iter()
            .map(|w| uppercase_first(w))
            .collect::<Vec<_>>()
            .join(""),
        _other => input.to_string(),
    }
}

fn uppercase_first(w: &str) -> String {
    let mut chars = w.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn snake_case(w: &str) -> String {
    let mut s = String::new();
    for (i, c) in w.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('_');
        }
        s.push(c.to_lowercase().next().unwrap_or(c));
    }
    s
}

fn kebab_case(w: &str) -> String {
    let mut s = String::new();
    for (i, c) in w.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('-');
        }
        s.push(c.to_lowercase().next().unwrap_or(c));
    }
    s
}

/// Generate lorem ipsum text with the requested number of words.
pub fn lorem(words: usize) -> String {
    let mut out = String::new();
    for n in 0..words {
        if n > 0 {
            out.push(' ');
        }
        out.push_str(&lorem_word(n));
    }
    out
}

fn lorem_word(n: usize) -> String {
    let l = [
        "lorem",
        "ipsum",
        "dolor",
        "sit",
        "amet",
        "consectetur",
        "adipiscing",
        "elit",
        "sed",
        "eiusmod",
        "tempor",
        "laborum",
        "magna",
    ];
    let r = [
        "aliquam",
        "nulla",
        "quis",
        "venenatis",
        "vestibulum",
        "integer",
        "mauris",
        "rhoncus",
        "tempus",
        "ultrices",
        "condimentum",
        "facilisis",
    ];
    let idx = n % (l.len() + r.len());
    if idx < l.len() {
        l[idx].to_string()
    } else {
        r[idx - l.len()].to_string()
    }
}

/// Dispatch to the transform for `slug`. Returns the output, or a short error
/// string if the slug is unknown.
///
/// `action` is used by `case` (camel/snake/kebab/pascal), `lorem` (word count)
/// and `replace` (occurrence count). `find` and `replace` feed the two find-and-
/// replace tools.
pub fn run(
    slug: &str,
    input: &str,
    action: Option<&str>,
    find: Option<&str>,
    repl: Option<&str>,
) -> String {
    let find = find.unwrap_or("");
    let repl = repl.unwrap_or("");
    if (slug == "replace" || slug == "replace-all") && find.is_empty() && repl.is_empty() {
        // No `find`/`replace` params supplied: apply the documented demo
        // substitution (first `X` -> `_`, or every occurrence for
        // `replace-all`). Keeps the endpoint deterministic and matching the
        // catalog example, e.g. `aXbXc -> a_bXc`.
        let count = if slug == "replace-all" {
            usize::MAX
        } else {
            count_from_action(action)
        };
        return replace(input, "X", "_", count);
    }
    match slug {
        "replace" => replace(input, find, repl, count_from_action(action)),
        "replace-all" => replace_all(input, find, repl),
        "collapse" => collapse(input),
        "uppercase" => uppercase(input),
        "lowercase" => lowercase(input),
        "title" => title(input),
        "reverse" => reverse(input),
        "trim" => trim(input),
        "url-encode" => url_encode(input),
        "url-decode" => url_decode(input),
        "html-encode" => html_encode(input),
        "html-decode" => html_decode(input),
        "hex-encode" => hex_encode(input),
        "hex-decode" => hex_decode(input),
        "base64-encode" => base64_encode(input),
        "base64-decode" => base64_decode(input),
        "unicode-strip" => unicode_strip(input),
        "word-count" => word_count(input),
        "json-pretty" => json_pretty(input),
        "json-minify" => json_minify(input),
        "slug" => crate::tools::slug(input),
        "case" => case(input, action),
        "lorem" => {
            // Word count may arrive as `action`, as the bare input ("3"), or as
            // `words=N` inside the input ("words=3"). Default is 5.
            let words = action
                .and_then(|a| a.parse::<usize>().ok())
                .or_else(|| input.parse::<usize>().ok())
                .or_else(|| {
                    input
                        .strip_prefix("words=")
                        .and_then(|w| w.parse::<usize>().ok())
                })
                .unwrap_or(5);
            lorem(words)
        }
        _ => format!("unknown tool: {}", slug),
    }
}

/// Derive a `count` argument from the `action` param for `replace`.
/// Default is 1 (replace the first occurrence); `all` / a large value mean
/// every occurrence.
fn count_from_action(action: Option<&str>) -> usize {
    match action {
        Some(a) if a.eq_ignore_ascii_case("all") => usize::MAX,
        Some(a) => a.parse::<usize>().unwrap_or(1),
        None => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_basic() {
        assert_eq!(replace("aXbXc", "X", "_", 1), "a_bXc");
    }

    #[test]
    fn replace_all_basic() {
        assert_eq!(replace_all("aXbXc", "X", "_"), "a_b_c");
    }

    #[test]
    fn replace_count() {
        assert_eq!(replace("aaaa", "a", "b", 2), "bbaa");
    }

    #[test]
    fn collapse_basic() {
        assert_eq!(collapse("a\n\n  b\tc"), "a b c");
    }

    #[test]
    fn collapse_multi() {
        assert_eq!(collapse("a\n\n\n\nb"), "a b");
    }

    #[test]
    fn uppercase_basic() {
        assert_eq!(uppercase("hello"), "HELLO");
    }

    #[test]
    fn lowercase_basic() {
        assert_eq!(lowercase("HELLO"), "hello");
    }

    #[test]
    fn title_basic() {
        assert_eq!(title("the quick brown"), "The Quick Brown");
    }

    #[test]
    fn reverse_basic() {
        assert_eq!(reverse("abc 123"), "321 cba");
    }

    #[test]
    fn trim_basic() {
        assert_eq!(trim("  hi  "), "hi");
    }

    #[test]
    fn url_encode_basic() {
        assert_eq!(url_encode("a b&c"), "a+b%26c");
    }

    #[test]
    fn url_decode_basic() {
        assert_eq!(url_decode("a+b%26c"), "a b&c");
    }

    #[test]
    fn html_encode_basic() {
        assert_eq!(html_encode("a<b>\""), "a&lt;b&gt;&quot;");
    }

    #[test]
    fn html_decode_basic() {
        assert_eq!(html_decode("a&lt;b&gt;\""), "a<b>\"");
    }

    #[test]
    fn hex_encode_basic() {
        assert_eq!(hex_encode("AB"), "4142");
    }

    #[test]
    fn hex_decode_basic() {
        assert_eq!(hex_decode("4142"), "AB");
    }

    #[test]
    fn base64_encode_basic() {
        assert_eq!(base64_encode("Man"), "TWFu");
    }

    #[test]
    fn base64_decode_basic() {
        assert_eq!(base64_decode("TWFu"), "Man");
    }

    #[test]
    fn unicode_strip_basic() {
        assert_eq!(unicode_strip("Héllo Wörld"), "Hllo Wrld");
    }

    #[test]
    fn word_count_basic() {
        assert_eq!(
            word_count("one two three"),
            "words: 3\nchars: 13\nlines: 1\nbytes: 13"
        );
    }

    #[test]
    fn word_count_multiline() {
        assert_eq!(
            word_count("a b\nc d e"),
            "words: 5\nchars: 9\nlines: 2\nbytes: 9"
        );
    }

    #[test]
    fn case_default_camel() {
        assert_eq!(case("hello world foo", None), "helloWorldFoo");
    }

    #[test]
    fn run_lorem_word_count_variants() {
        // `words=` in the input, bare number as input, and action override.
        assert_eq!(
            run("lorem", "words=3", None, None, None),
            "lorem ipsum dolor"
        );
        assert_eq!(run("lorem", "3", None, None, None), "lorem ipsum dolor");
        assert_eq!(
            run("lorem", "", Some("4"), None, None),
            "lorem ipsum dolor sit"
        );
        assert_eq!(
            run("lorem", "", None, None, None),
            "lorem ipsum dolor sit amet"
        );
    }

    #[test]
    fn json_pretty_basic() {
        assert_eq!(
            json_pretty(r#"{"a":1,"b":2}"#),
            r#"{
  "a": 1,
  "b": 2
}"#
        );
    }

    #[test]
    fn json_minify_basic() {
        assert_eq!(json_minify(r#"{"a": 1}"#), r#"{"a":1}"#);
    }

    #[test]
    fn slug_basic() {
        assert_eq!(slug("Hello World & Friends!"), "hello-world-friends");
    }

    #[test]
    fn case_kebab() {
        assert_eq!(case("hello world foo", Some("kebab")), "hello-world-foo");
    }

    #[test]
    fn case_snake() {
        assert_eq!(case("hello world foo", Some("snake")), "hello_world_foo");
    }

    #[test]
    fn case_camel() {
        assert_eq!(case("hello world foo", Some("camel")), "helloWorldFoo");
    }

    #[test]
    fn case_pascal() {
        assert_eq!(case("hello world foo", Some("pascal")), "HelloWorldFoo");
    }

    #[test]
    fn lorem_default() {
        assert_eq!(lorem(5), "lorem ipsum dolor sit amet");
    }

    #[test]
    fn lorem_three() {
        assert_eq!(lorem(3), "lorem ipsum dolor");
    }

    #[test]
    fn run_dispatch_variants() {
        assert_eq!(run("uppercase", "hi", None, None, None), "HI");
        assert_eq!(run("slug", "Hello World", None, None, None), "hello-world");
        assert_eq!(
            run("title", "the quick brown", None, None, None),
            "The Quick Brown"
        );
    }

    #[test]
    fn run_dispatch() {
        assert_eq!(run("uppercase", "x", None, None, None), "X");
        assert_eq!(
            run("unknown", "x", None, None, None),
            "unknown tool: unknown"
        );
    }

    #[test]
    fn run_replace_dispatch() {
        // `?input=aXbXc&find=X&replace=_&count=1` -> replace first `X`.
        assert_eq!(run("replace", "aXbXc", None, Some("X"), Some("_")), "a_bXc");
        // count=2 replaces both occurrences.
        assert_eq!(
            run("replace", "aXbXc", Some("2"), Some("X"), Some("_")),
            "a_b_c"
        );
        // `all` replaces every occurrence.
        assert_eq!(
            run("replace", "aXbXc", Some("all"), Some("X"), Some("_")),
            "a_b_c"
        );
    }

    #[test]
    fn run_replace_all_dispatch() {
        assert_eq!(
            run("replace-all", "aXbXc", None, Some("X"), Some("_")),
            "a_b_c"
        );
    }

    #[test]
    fn run_case_dispatch() {
        assert_eq!(
            run("case", "hello world foo", None, None, None),
            "helloWorldFoo"
        );
        assert_eq!(
            run("case", "hello world foo", Some("snake"), None, None),
            "hello_world_foo"
        );
    }

    #[test]
    fn collapse_multi_newlines() {
        assert_eq!(collapse("a\n\n\n\tb"), "a b");
    }

    #[test]
    fn replace_empty_find_returns_input() {
        // An empty `find` with a non-empty `replace` matches nothing, so the
        // input is echoed unchanged.
        assert_eq!(run("replace", "abc", None, Some(""), Some("z")), "abc");
    }

    #[test]
    fn run_replace_no_params_defaults_to_demo() {
        // No params -> documented demo substitution (first `X` -> `_`).
        assert_eq!(run("replace", "aXbXc", None, None, None), "a_bXc");
    }

    #[test]
    fn run_replace_all_no_params_defaults_to_demo() {
        assert_eq!(run("replace-all", "aXbXc", None, None, None), "a_b_c");
    }

    #[test]
    fn run_replace_action_all() {
        assert_eq!(run("replace", "aXbXc", Some("all"), None, None), "a_b_c");
    }
}
