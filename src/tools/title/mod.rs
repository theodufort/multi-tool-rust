//! Title-case: capitalize the first letter of every word.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_basic() {
        assert_eq!(title("the quick brown"), "The Quick Brown");
    }
}
