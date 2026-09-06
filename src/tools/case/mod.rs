//! Case conversion (camel, snake, kebab, pascal).

use crate::utils::uppercase_first;

/// Convert text to a target case (camel, snake, kebab, pascal).
/// Defaults to `camel` when no action is given.
pub fn case(input: &str, action: Option<&str>) -> String {
    let action = action.unwrap_or("camel");
    let words: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect();
    match action {
        "snake" => words
            .iter()
            .map(|w| snake_case(w))
            .collect::<Vec<_>>()
            .join("_"),
        "kebab" => words
            .iter()
            .map(|w| kebab_case(w))
            .collect::<Vec<_>>()
            .join("-"),
        "camel" => {
            let mut s = String::new();
            for (i, w) in words.iter().enumerate() {
                if i == 0 {
                    s += w;
                } else {
                    s += &uppercase_first(w);
                }
            }
            s
        }
        "pascal" => words
            .iter()
            .map(|w| uppercase_first(w))
            .collect::<Vec<_>>()
            .join(""),
        _other => input.to_string(),
    }
}

fn snake_case(w: &str) -> String {
    let mut s = String::new();
    for (i, c) in w.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('_');
        }
        s.push(c.to_lowercase().next().unwrap_or(c));
    }
    s
}

fn kebab_case(w: &str) -> String {
    let mut s = String::new();
    for (i, c) in w.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('-');
        }
        s.push(c.to_lowercase().next().unwrap_or(c));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_default_camel() {
        assert_eq!(case("hello world foo", None), "helloWorldFoo");
    }

    #[test]
    fn case_kebab() {
        assert_eq!(case("hello world foo", Some("kebab")), "hello-world-foo");
    }

    #[test]
    fn case_snake() {
        assert_eq!(case("hello world foo", Some("snake")), "hello_world_foo");
    }

    #[test]
    fn case_camel() {
        assert_eq!(case("hello world foo", Some("camel")), "helloWorldFoo");
    }

    #[test]
    fn case_pascal() {
        assert_eq!(case("hello world foo", Some("pascal")), "HelloWorldFoo");
    }
}
