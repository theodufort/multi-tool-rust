//! Lowercase the input (Unicode-aware).

pub fn lowercase(input: &str) -> String {
    input.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_basic() {
        assert_eq!(lowercase("HELLO"), "hello");
    }
}
