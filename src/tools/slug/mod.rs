//! Slugify: lowercase, replace runs of non-alphanumeric chars with `-`.

/// Slugify: lowercase, replace runs of non-alphanumeric chars with `-`, trim.
pub fn slug(input: &str) -> String {
    let mut out = String::new();
    let mut in_run = false;
    for c in input.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_basic() {
        assert_eq!(slug("Hello World & Friends!"), "hello-world-friends");
    }
}
