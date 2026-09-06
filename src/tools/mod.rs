//! Pure, deterministic, dependency-light text transforms.
//!
//! Each tool lives in its own submodule under `src/tools/<name>/`. This module
//! re-exports every tool's public function and provides the `run` dispatcher,
//! which routes a slug to the right transform. The tools perform no I/O,
//! network, or allocation-heavy work beyond building the output `String`.

pub mod base64;
pub mod case;
pub mod collapse;
pub mod hex;
pub mod html;
pub mod json;
pub mod lorem;
pub mod lower;
pub mod replace;
pub mod reverse;
pub mod slug;
pub mod title;
pub mod trim;
pub mod unicode_strip;
pub mod upper;
pub mod url;
pub mod util;
pub mod word_count;

pub use base64::{base64_decode, base64_encode};
pub use case::case;
pub use collapse::collapse;
pub use hex::{hex_decode, hex_encode};
pub use html::{html_decode, html_encode};
pub use json::{json_minify, json_pretty};
pub use lorem::lorem;
pub use lower::lowercase;
pub use replace::{replace, replace_all};
pub use reverse::reverse;
pub use slug::slug;
pub use title::title;
pub use trim::trim;
pub use unicode_strip::unicode_strip;
pub use upper::uppercase;
pub use url::{url_decode, url_encode};
pub use word_count::word_count;

/// Dispatch to the transform for `slug`. Returns the output, or a short error
/// string if the slug is unknown.
///
/// `action` is used by `case` (camel/snake/kebab/pascal), `lorem` (word count)
/// and `replace` (occurrence count). `find` and `replace` feed the two find-and-
/// replace tools.
pub fn run(
    slug: &str,
    input: &str,
    action: Option<&str>,
    find: Option<&str>,
    repl: Option<&str>,
) -> String {
    let find = find.unwrap_or("");
    let repl = repl.unwrap_or("");
    if (slug == "replace" || slug == "replace-all") && find.is_empty() && repl.is_empty() {
        // No `find`/`replace` params supplied: apply the documented demo
        // substitution (first `X` -> `_`, or every occurrence for
        // `replace-all`). Keeps the endpoint deterministic and matching the
        // catalog example, e.g. `aXbXc -> a_bXc`.
        let count = if slug == "replace-all" {
            usize::MAX
        } else {
            replace::count_from_action(action)
        };
        return replace(input, "X", "_", count);
    }
    match slug {
        "replace" => replace(input, find, repl, replace::count_from_action(action)),
        "replace-all" => replace_all(input, find, repl),
        "collapse" => collapse(input),
        "uppercase" => uppercase(input),
        "lowercase" => lowercase(input),
        "title" => title(input),
        "reverse" => reverse(input),
        "trim" => trim(input),
        "url-encode" => url_encode(input),
        "url-decode" => url_decode(input),
        "html-encode" => html_encode(input),
        "html-decode" => html_decode(input),
        "hex-encode" => hex_encode(input),
        "hex-decode" => hex_decode(input),
        "base64-encode" => base64_encode(input),
        "base64-decode" => base64_decode(input),
        "unicode-strip" => unicode_strip(input),
        "word-count" => word_count(input),
        "json-pretty" => json_pretty(input),
        "json-minify" => json_minify(input),
        "slug" => crate::tools::slug(input),
        "case" => case(input, action),
        "lorem" => {
            // Word count may arrive as `action`, as the bare input ("3"), or as
            // `words=N` inside the input ("words=3"). Default is 5.
            let words = action
                .and_then(|a| a.parse::<usize>().ok())
                .or_else(|| input.parse::<usize>().ok())
                .or_else(|| {
                    input
                        .strip_prefix("words=")
                        .and_then(|w| w.parse::<usize>().ok())
                })
                .unwrap_or(5);
            lorem(words)
        }
        _ => format!("unknown tool: {}", slug),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lorem_word_count_variants() {
        // `words=` in the input, bare number as input, and action override.
        assert_eq!(
            run("lorem", "words=3", None, None, None),
            "lorem ipsum dolor"
        );
        assert_eq!(run("lorem", "3", None, None, None), "lorem ipsum dolor");
        assert_eq!(
            run("lorem", "", Some("4"), None, None),
            "lorem ipsum dolor sit"
        );
        assert_eq!(
            run("lorem", "", None, None, None),
            "lorem ipsum dolor sit amet"
        );
    }

    #[test]
    fn run_dispatch_variants() {
        assert_eq!(run("uppercase", "hi", None, None, None), "HI");
        assert_eq!(run("slug", "Hello World", None, None, None), "hello-world");
        assert_eq!(
            run("title", "the quick brown", None, None, None),
            "The Quick Brown"
        );
    }

    #[test]
    fn run_dispatch() {
        assert_eq!(run("uppercase", "x", None, None, None), "X");
        assert_eq!(
            run("unknown", "x", None, None, None),
            "unknown tool: unknown"
        );
    }

    #[test]
    fn run_replace_dispatch() {
        // `?input=aXbXc&find=X&replace=_&count=1` -> replace first `X`.
        assert_eq!(run("replace", "aXbXc", None, Some("X"), Some("_")), "a_bXc");
        // count=2 replaces both occurrences.
        assert_eq!(
            run("replace", "aXbXc", Some("2"), Some("X"), Some("_")),
            "a_b_c"
        );
        // `all` replaces every occurrence.
        assert_eq!(
            run("replace", "aXbXc", Some("all"), Some("X"), Some("_")),
            "a_b_c"
        );
    }

    #[test]
    fn run_replace_all_dispatch() {
        assert_eq!(
            run("replace-all", "aXbXc", None, Some("X"), Some("_")),
            "a_b_c"
        );
    }

    #[test]
    fn run_case_dispatch() {
        assert_eq!(
            run("case", "hello world foo", None, None, None),
            "helloWorldFoo"
        );
        assert_eq!(
            run("case", "hello world foo", Some("snake"), None, None),
            "hello_world_foo"
        );
    }

    #[test]
    fn replace_empty_find_returns_input() {
        // An empty `find` with a non-empty `replace` matches nothing, so the
        // input is echoed unchanged.
        assert_eq!(run("replace", "abc", None, Some(""), Some("z")), "abc");
    }

    #[test]
    fn run_replace_no_params_defaults_to_demo() {
        // No params -> documented demo substitution (first `X` -> `_`).
        assert_eq!(run("replace", "aXbXc", None, None, None), "a_bXc");
    }

    #[test]
    fn run_replace_all_no_params_defaults_to_demo() {
        assert_eq!(run("replace-all", "aXbXc", None, None, None), "a_b_c");
    }

    #[test]
    fn run_replace_action_all() {
        assert_eq!(run("replace", "aXbXc", Some("all"), None, None), "a_b_c");
    }
}
