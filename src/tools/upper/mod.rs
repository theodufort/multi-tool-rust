//! Uppercase the input (Unicode-aware).

pub fn uppercase(input: &str) -> String {
    input.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uppercase_basic() {
        assert_eq!(uppercase("hello"), "HELLO");
    }
}
