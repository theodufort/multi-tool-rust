//! JSON pretty-print / minify.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
