//! Find-and-replace tools.

/// Replace `count` occurrences of `find` with `replace` (count == 0 means all).
/// An empty `find` matches nothing, so the input is returned unchanged.
pub fn replace(input: &str, find: &str, replace: &str, count: usize) -> String {
    if find.is_empty() {
        return input.to_string();
    }
    let mut out = String::new();
    let mut i = 0;
    let mut done = 0;
    while i < input.len() {
        if done < count && input[i..i + find.len()] == *find {
            out.push_str(replace);
            i += find.len();
            done += 1;
        } else {
            match input[i..].chars().next() {
                Some(c) => {
                    out.push(c);
                    i += c.len_utf8();
                }
                None => break,
            }
        }
    }
    out
}

/// Replace every occurrence of `find` with `replace`.
pub fn replace_all(input: &str, find: &str, to: &str) -> String {
    replace(input, find, to, usize::MAX)
}

/// Derive a `count` argument from the `action` param for `replace`.
/// Default is 1 (replace the first occurrence); `all` / a large value mean
/// every occurrence.
pub fn count_from_action(action: Option<&str>) -> usize {
    match action {
        Some(a) if a.eq_ignore_ascii_case("all") => usize::MAX,
        Some(a) => a.parse::<usize>().unwrap_or(1),
        None => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_basic() {
        assert_eq!(replace("aXbXc", "X", "_", 1), "a_bXc");
    }

    #[test]
    fn replace_all_basic() {
        assert_eq!(replace_all("aXbXc", "X", "_"), "a_b_c");
    }

    #[test]
    fn replace_count() {
        assert_eq!(replace("aaaa", "a", "b", 2), "bbaa");
    }

    #[test]
    fn replace_empty_find_returns_input() {
        // An empty `find` with a non-empty `replace` matches nothing, so the
        // input is echoed unchanged.
        assert_eq!(replace("abc", "", "z", 1), "abc");
    }
}
