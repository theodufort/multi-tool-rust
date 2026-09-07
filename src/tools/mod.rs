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
pub mod registry;
pub mod replace;
pub mod reverse;
pub mod slug;
pub mod title;
pub mod trim;
pub mod unicode_strip;
pub mod upper;
pub mod url;
pub mod word_count;

pub use registry::build_registry;

/// Dispatch to the transform for `slug`. Returns the full [`ToolOutput`]
/// (result, error, execution time, slug); for an unknown slug the result holds
/// a short `"unknown tool: <slug>"` message and the time is `0.0`.
///
/// `action` is used by `case` (camel/snake/kebab/pascal), `lorem` (word count)
/// and `replace` (occurrence count). `find` and `replace` feed the two find-and-
/// replace tools.
///
/// This is a thin wrapper over the object-oriented [`ToolRegistry`]: it builds
/// the registry once (lazily), converts the positional args into a [`ToolInput`],
/// and returns the [`ToolOutput`] directly, so callers can read both the
/// [`ToolOutput::result`] and its timing/error metadata.
pub fn run(
    url_slug: &str,
    input: &str,
    action: Option<&str>,
    find: Option<&str>,
    repl: Option<&str>,
) -> crate::types::tools::ToolOutput {
    let tool_input = crate::types::tools::ToolInput {
        input: input.to_string(),
        action: action.map(str::to_string),
        find: find.map(str::to_string),
        replace: repl.map(str::to_string),
    };
    registry().run(url_slug, &tool_input).unwrap_or_else(|| {
        crate::types::tools::ToolOutput::ok(url_slug, format!("unknown tool: {}", url_slug), 0.0)
    })
}

/// Lazily-built, process-wide [`ToolRegistry`] with every tool registered.
///
/// Built once on first use and reused thereafter, so dispatch is a single
/// lookup. Use this directly when you need [`ToolOutput`] metadata (timing,
/// errors) or want to benchmark tools.
pub fn registry() -> &'static crate::types::tools::ToolRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<crate::types::tools::ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lorem_word_count_variants() {
        // `words=` in the input, bare number as input, and action override.
        assert_eq!(
            run("lorem", "words=3", None, None, None).result,
            "lorem ipsum dolor"
        );
        assert_eq!(run("lorem", "3", None, None, None).result, "lorem ipsum dolor");
        assert_eq!(
            run("lorem", "", Some("4"), None, None).result,
            "lorem ipsum dolor sit"
        );
        assert_eq!(
            run("lorem", "", None, None, None).result,
            "lorem ipsum dolor sit amet"
        );
    }

    #[test]
    fn run_dispatch_variants() {
        assert_eq!(run("uppercase", "hi", None, None, None).result, "HI");
        assert_eq!(run("slug", "Hello World", None, None, None).result, "hello-world");
        assert_eq!(
            run("title", "the quick brown", None, None, None).result,
            "The Quick Brown"
        );
    }

    #[test]
    fn run_dispatch() {
        assert_eq!(run("uppercase", "x", None, None, None).result, "X");
        assert_eq!(
            run("unknown", "x", None, None, None).result,
            "unknown tool: unknown"
        );
        // Unknown slug keeps an empty error and zero time.
        let unknown = run("unknown", "x", None, None, None);
        assert!(unknown.is_ok());
        assert_eq!(unknown.execution_time, 0.0);
    }

    #[test]
    fn run_replace_dispatch() {
        // `?input=aXbXc&find=X&replace=_&count=1` -> replace first `X`.
        assert_eq!(
            run("replace", "aXbXc", None, Some("X"), Some("_")).result,
            "a_bXc"
        );
        // count=2 replaces both occurrences.
        assert_eq!(
            run("replace", "aXbXc", Some("2"), Some("X"), Some("_")).result,
            "a_b_c"
        );
        // `all` replaces every occurrence.
        assert_eq!(
            run("replace", "aXbXc", Some("all"), Some("X"), Some("_")).result,
            "a_b_c"
        );
    }

    #[test]
    fn run_replace_all_dispatch() {
        assert_eq!(
            run("replace-all", "aXbXc", None, Some("X"), Some("_")).result,
            "a_b_c"
        );
    }

    #[test]
    fn run_case_dispatch() {
        assert_eq!(
            run("case", "hello world foo", None, None, None).result,
            "helloWorldFoo"
        );
        assert_eq!(
            run("case", "hello world foo", Some("snake"), None, None).result,
            "hello_world_foo"
        );
    }

    #[test]
    fn replace_empty_find_returns_input() {
        // An empty `find` with a non-empty `replace` matches nothing, so the
        // input is echoed unchanged.
        assert_eq!(
            run("replace", "abc", None, Some(""), Some("z")).result,
            "abc"
        );
    }

    #[test]
    fn run_replace_no_params_defaults_to_demo() {
        // No params -> documented demo substitution (first `X` -> `_`).
        assert_eq!(
            run("replace", "aXbXc", None, None, None).result,
            "a_bXc"
        );
    }

    #[test]
    fn run_replace_all_no_params_defaults_to_demo() {
        assert_eq!(
            run("replace-all", "aXbXc", None, None, None).result,
            "a_b_c"
        );
    }

    #[test]
    fn run_replace_action_all() {
        assert_eq!(
            run("replace", "aXbXc", Some("all"), None, None).result,
            "a_b_c"
        );
    }
}
