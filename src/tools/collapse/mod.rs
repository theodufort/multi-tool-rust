//! Collapse runs of whitespace into a single space.

/// Collapse runs of whitespace (space, tab, newline, CR, FF, VT) into a single
/// space and strip leading/trailing whitespace.
pub fn collapse(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_basic() {
        assert_eq!(collapse("a\n\n  b\tc"), "a b c");
    }

    #[test]
    fn collapse_multi() {
        assert_eq!(collapse("a\n\n\n\nb"), "a b");
    }

    #[test]
    fn collapse_multi_newlines() {
        assert_eq!(collapse("a\n\n\n\tb"), "a b");
    }
}
