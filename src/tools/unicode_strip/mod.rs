//! Strip non-ASCII characters (Unicode -> ASCII).

pub fn unicode_strip(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii()).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_strip_basic() {
        assert_eq!(unicode_strip("Héllo Wörld"), "Hllo Wrld");
    }
}
