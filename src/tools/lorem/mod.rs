//! Lorem ipsum generator.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lorem_default() {
        assert_eq!(lorem(5), "lorem ipsum dolor sit amet");
    }

    #[test]
    fn lorem_three() {
        assert_eq!(lorem(3), "lorem ipsum dolor");
    }
}
