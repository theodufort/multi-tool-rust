//! Trim leading and trailing whitespace.

pub fn trim(input: &str) -> String {
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_basic() {
        assert_eq!(trim("  hi  "), "hi");
    }
}
