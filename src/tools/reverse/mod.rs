//! Reverse the input, preserving character order (Unicode-aware).

pub fn reverse(input: &str) -> String {
    input.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_basic() {
        assert_eq!(reverse("abc 123"), "321 cba");
    }
}
