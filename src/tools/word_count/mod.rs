//! Count words, characters, lines, and bytes.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
