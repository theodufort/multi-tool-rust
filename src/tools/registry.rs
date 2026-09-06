//! Concrete [`Tool`] implementations and the shared [`ToolRegistry`].
//!
//! Every tool in `src/tools/<name>/` is wrapped in a small struct that
//! implements the [`Tool`] trait from `crate::types::tools`. This gives every
//! tool a standardized input/output and free benchmarking, while keeping the
//! pure transform functions untouched.
//!
//! [`build_registry`] registers every tool once; callers dispatch through
//! [`ToolRegistry::run`] / [`ToolRegistry::benchmark`].

use crate::tools::{
    base64, case, collapse, hex, html, json, lorem, lower, replace, reverse, slug, title, trim,
    unicode_strip, upper, url, word_count,
};
use crate::types::tools::{Tool, ToolInput, ToolRegistry};

// ---------------------------------------------------------------------------
// Simple single-function tools
// ---------------------------------------------------------------------------

macro_rules! simple_tool {
    ($name:ident, $slug:literal, $label:literal, $f:path) => {
        pub struct $name;

        impl Tool for $name {
            fn slug(&self) -> &str {
                $slug
            }
            fn label(&self) -> &str {
                $label
            }
            fn run(&self, input: &ToolInput) -> String {
                $f(&input.input)
            }
        }
    };
}

simple_tool!(Collapse, "collapse", "Remove Line Breaks", collapse::collapse);
simple_tool!(Uppercase, "uppercase", "Uppercase", upper::uppercase);
simple_tool!(Lowercase, "lowercase", "Lowercase", lower::lowercase);
simple_tool!(Title, "title", "Title Case", title::title);
simple_tool!(Reverse, "reverse", "Reverse", reverse::reverse);
simple_tool!(Trim, "trim", "Trim", trim::trim);
simple_tool!(UrlEncode, "url-encode", "URL Encode", url::url_encode);
simple_tool!(UrlDecode, "url-decode", "URL Decode", url::url_decode);
simple_tool!(HtmlEncode, "html-encode", "HTML Encode", html::html_encode);
simple_tool!(HtmlDecode, "html-decode", "HTML Decode", html::html_decode);
simple_tool!(HexEncode, "hex-encode", "Hex Encode", hex::hex_encode);
simple_tool!(HexDecode, "hex-decode", "Hex Decode", hex::hex_decode);
simple_tool!(Base64Encode, "base64-encode", "Base64 Encode", base64::base64_encode);
simple_tool!(Base64Decode, "base64-decode", "Base64 Decode", base64::base64_decode);
simple_tool!(UnicodeStrip, "unicode-strip", "Unicode -> ASCII", unicode_strip::unicode_strip);
simple_tool!(WordCount, "word-count", "Word/Char/Line/Byte Count", word_count::word_count);
simple_tool!(JsonPretty, "json-pretty", "JSON Pretty Print", json::json_pretty);
simple_tool!(JsonMinify, "json-minify", "JSON Minify", json::json_minify);
simple_tool!(Slug, "slug", "Slugify", slug::slug);

// ---------------------------------------------------------------------------
// Parameterised tools
// ---------------------------------------------------------------------------

/// `replace` / `replace-all`: uses `find`, `replace`, and `action` (count).
pub struct Replace;

impl Tool for Replace {
    fn slug(&self) -> &str {
        "replace"
    }
    fn label(&self) -> &str {
        "Find & Replace"
    }
    fn run(&self, input: &ToolInput) -> String {
        let find = input.find.as_deref().unwrap_or("");
        let repl = input.replace.as_deref().unwrap_or("");
        if find.is_empty() && repl.is_empty() {
            // No params: apply the documented demo substitution (first `X`).
            return replace::replace(input.input.as_str(), "X", "_", replace::count_from_action(input.action.as_deref()));
        }
        replace::replace(
            &input.input,
            find,
            repl,
            replace::count_from_action(input.action.as_deref()),
        )
    }
}

/// `replace-all`: replaces every occurrence of `find` with `replace`.
pub struct ReplaceAll;

impl Tool for ReplaceAll {
    fn slug(&self) -> &str {
        "replace-all"
    }
    fn label(&self) -> &str {
        "Replace All"
    }
    fn run(&self, input: &ToolInput) -> String {
        let find = input.find.as_deref().unwrap_or("");
        let repl = input.replace.as_deref().unwrap_or("");
        if find.is_empty() && repl.is_empty() {
            return replace::replace_all(&input.input, "X", "_");
        }
        replace::replace_all(&input.input, find, repl)
    }
}

/// `case`: uses `action` (camel/snake/kebab/pascal).
pub struct Case;

impl Tool for Case {
    fn slug(&self) -> &str {
        "case"
    }
    fn label(&self) -> &str {
        "Case Converter"
    }
    fn run(&self, input: &ToolInput) -> String {
        case::case(&input.input, input.action.as_deref())
    }
}

/// `lorem`: uses `action` or the input as the word count.
pub struct Lorem;

impl Tool for Lorem {
    fn slug(&self) -> &str {
        "lorem"
    }
    fn label(&self) -> &str {
        "Lorem Ipsum"
    }
    fn run(&self, input: &ToolInput) -> String {
        let words = input
            .action
            .as_deref()
            .and_then(|a| a.parse::<usize>().ok())
            .or_else(|| input.input.parse::<usize>().ok())
            .or_else(|| {
                input
                    .input
                    .strip_prefix("words=")
                    .and_then(|w| w.parse::<usize>().ok())
            })
            .unwrap_or(5);
        lorem::lorem(words)
    }
}

// ---------------------------------------------------------------------------
// Registry construction
// ---------------------------------------------------------------------------

/// Build a [`ToolRegistry`] with every tool registered.
pub fn build_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(Replace));
    reg.register(Box::new(ReplaceAll));
    reg.register(Box::new(Collapse));
    reg.register(Box::new(Uppercase));
    reg.register(Box::new(Lowercase));
    reg.register(Box::new(Title));
    reg.register(Box::new(Reverse));
    reg.register(Box::new(Trim));
    reg.register(Box::new(UrlEncode));
    reg.register(Box::new(UrlDecode));
    reg.register(Box::new(HtmlEncode));
    reg.register(Box::new(HtmlDecode));
    reg.register(Box::new(HexEncode));
    reg.register(Box::new(HexDecode));
    reg.register(Box::new(Base64Encode));
    reg.register(Box::new(Base64Decode));
    reg.register(Box::new(UnicodeStrip));
    reg.register(Box::new(WordCount));
    reg.register(Box::new(JsonPretty));
    reg.register(Box::new(JsonMinify));
    reg.register(Box::new(Slug));
    reg.register(Box::new(Case));
    reg.register(Box::new(Lorem));
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_tools() {
        let reg = build_registry();
        assert_eq!(reg.len(), 23);
        for slug in [
            "replace",
            "replace-all",
            "collapse",
            "uppercase",
            "lowercase",
            "title",
            "reverse",
            "trim",
            "url-encode",
            "url-decode",
            "html-encode",
            "html-decode",
            "hex-encode",
            "hex-decode",
            "base64-encode",
            "base64-decode",
            "unicode-strip",
            "word-count",
            "json-pretty",
            "json-minify",
            "slug",
            "case",
            "lorem",
        ] {
            assert!(reg.get(slug).is_some(), "missing tool: {slug}");
        }
    }

    #[test]
    fn registry_run_matches_legacy_dispatcher() {
        let reg = build_registry();
        let cases = [
            ("uppercase", ToolInput::new("hi"), "HI"),
            ("slug", ToolInput::new("Hello World"), "hello-world"),
            ("title", ToolInput::new("the quick brown"), "The Quick Brown"),
            ("replace", ToolInput::new("aXbXc").with_find_replace("X", "_"), "a_bXc"),
            (
                "replace",
                ToolInput::new("aXbXc").with_find_replace("X", "_").with_action("2"),
                "a_b_c",
            ),
            (
                "case",
                ToolInput::new("hello world foo").with_action("snake"),
                "hello_world_foo",
            ),
            ("lorem", ToolInput::new("words=3"), "lorem ipsum dolor"),
            ("lorem", ToolInput::new("3"), "lorem ipsum dolor"),
            ("lorem", ToolInput::new("").with_action("4"), "lorem ipsum dolor sit"),
        ];
        for (slug, input, expected) in cases {
            let out = reg.run(slug, &input).expect("tool should exist");
            assert_eq!(out.result, expected, "slug={slug}");
            assert!(out.is_ok());
        }
    }

    #[test]
    fn registry_run_unknown_returns_none() {
        let reg = build_registry();
        assert!(reg.run("nope", &ToolInput::new("x")).is_none());
    }

    #[test]
    fn registry_benchmark_every_tool() {
        let reg = build_registry();
        let samples = reg.benchmark_all(&ToolInput::new("hello world"));
        assert_eq!(samples.len(), 23);
        for s in &samples {
            assert!(s.total_time >= 0.0);
            assert!(s.avg_time >= 0.0);
        }
    }
}
