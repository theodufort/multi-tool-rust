//! Machine + UI routes for the text tools.
//!
//! - `GET /api/<slug>?<input>`            -> raw transform output (plain text, contract)
//! - `GET /api/<slug>?<input>&<action>`   (case) -> output with an action param
//! - `GET /tool/<slug>...`                -> rendered page with a `<pre id="result">` body
//!
//! Rocket 0.5.1 has no `QueryData` extractor and no `rocket::request()`. Query
//! params are therefore read as handler arguments (`?<ident>` / `?<ident>&<ident>`).
//! Routes are registered most-specific first so a URL carrying an extra param
//! matches the richer route. Each handler has a unique name (Rocket 0.5.1 needs
//! distinct function names when mounted together).

use rocket_dyn_templates::{Template, context};

use crate::tools;

/// Full tool catalog (slug, label). Sidebar lists every tool.
pub const CATALOG: &[(&str, &str)] = &[
    ("replace", "Find & Replace"),
    ("replace-all", "Replace All"),
    ("collapse", "Remove Line Breaks"),
    ("uppercase", "Uppercase"),
    ("lowercase", "Lowercase"),
    ("title", "Title Case"),
    ("reverse", "Reverse"),
    ("trim", "Trim"),
    ("url-encode", "URL Encode"),
    ("url-decode", "URL Decode"),
    ("html-encode", "HTML Encode"),
    ("html-decode", "HTML Decode"),
    ("hex-encode", "Hex Encode"),
    ("hex-decode", "Hex Decode"),
    ("base64-encode", "Base64 Encode"),
    ("base64-decode", "Base64 Decode"),
    ("unicode-strip", "Unicode -> ASCII"),
    ("word-count", "Word/Char/Line/Byte Count"),
    ("json-pretty", "JSON Pretty Print"),
    ("json-minify", "JSON Minify"),
    ("slug", "Slugify"),
    ("case", "Case Converter"),
    ("lorem", "Lorem Ipsum"),
];

/// HTML `<a>` tiles for the home page grid (one per tool).
pub fn home_tiles() -> String {
    CATALOG
        .iter()
        .map(|(slug, label)| format!(
            "          <a class=\"tool-tile\" href=\"/tool/{slug}\" data-slug=\"{slug}\" data-name=\"{label}\">{label}<small>/api/{slug}</small></a>\n"
        ))
        .collect()
}

/// Human-readable label for `slug`, falling back to the slug itself.
pub fn tool_label(slug: &str) -> String {
    CATALOG
        .iter()
        .find(|(s, _)| *s == slug)
        .map(|(_, label)| label.to_string())
        .unwrap_or_else(|| slug.to_string())
}

/// HTML `<li>` items for the sidebar (the `<ul>` lives in the templates).
pub fn sidebar_links() -> String {
    let mut s = String::new();
    for (slug, label) in CATALOG {
        s.push_str(&format!(
            "          <li class=\"nav-item\">\n            <a class=\"nav-link\" href=\"/tool/{slug}\">{label}</a>\n          </li>\n"
        ));
    }
    s
}

// ---------------------------------------------------------------------------
// Machine endpoints: `/api/<slug>`
// ---------------------------------------------------------------------------

// All optional query params are declared on a single route so any subset of
// them matches. Rocket ignores query params that a client sends but the route
// does not declare, and binds a declared-but-absent `?{}` param to `None`, so
// one route covers `?input=...`, `?input=...&action=...`,
// `?input=...&find=...&replace=...`, etc.
// `rank` keeps this ahead of the mounted FileServer route (default rank 10,
// `/<path..>`), which otherwise collides with every path.
#[get("/api/<slug>?<input>&<find>&<replace>&<action>", rank = 1)]
pub fn api(
    slug: &str,
    input: Option<String>,
    find: Option<String>,
    replace: Option<String>,
    action: Option<String>,
) -> String {
    tools::run(
        slug,
        input.as_deref().unwrap_or(""),
        action.as_deref(),
        find.as_deref(),
        replace.as_deref(),
    )
}

/// Benchmark tools and print per-tool timing.
///
/// `?input=...` sets the sample text (default `"hello world"`); `?iterations=N`
/// sets how many times each tool runs (default 1000). Pass `?slug=<slug>` to
/// benchmark a single tool; omit it to benchmark every registered tool.
///
/// Output is plain text, one line per tool:
/// `slug  avg_ms  total_ms  iterations`.
#[get("/api/benchmark?<slug>&<input>&<iterations>", rank = 0)]
pub fn benchmark(slug: Option<String>, input: Option<String>, iterations: Option<u32>) -> String {
    let input = input.unwrap_or_else(|| "hello world".to_string());
    let iterations = iterations.unwrap_or(1000);
    let tool_input = crate::types::tools::ToolInput::new(input);
    let reg = tools::registry();

    // Benchmark a single tool when `slug` is given.
    if let Some(slug) = slug {
        return match reg.benchmark(&slug, &tool_input, iterations) {
            Some(b) => format!(
                "benchmarking {} ({} iterations)\n{:<16} {:>10.4} ms/op  {:>10.4} ms total  ({} iters)",
                b.tool,
                b.iterations,
                b.tool,
                b.avg_ms(),
                b.total_time * 1000.0,
                b.iterations
            ),
            None => format!("unknown tool: {}", slug),
        };
    }

    // Otherwise benchmark every registered tool.
    let mut lines = vec![format!(
        "benchmarking {} tools, {} iterations each",
        reg.len(),
        iterations
    )];
    for slug in reg.slugs() {
        if let Some(b) = reg.benchmark(slug, &tool_input, iterations) {
            lines.push(format!(
                "{:<16} {:>10.4} ms/op  {:>10.4} ms total  ({} iters)",
                b.tool,
                b.avg_ms(),
                b.total_time * 1000.0,
                b.iterations
            ));
        }
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// UI endpoints: `/tool/<slug>`
// ---------------------------------------------------------------------------

/// Template context shared by all `/tool/<slug>` route variants.
fn tool_context(
    slug: &str,
    input: String,
    output: String,
    action: Option<String>,
    find: Option<String>,
    repl: Option<String>,
) -> rocket_dyn_templates::Template {
    let needs_find_replace = slug == "replace" || slug == "replace-all";
    let needs_action = slug == "case";
    let action = action.unwrap_or_default();
    Template::render(
        "tool",
        context! {
            title: slug,
            label: tool_label(slug),
            input,
            output,
            action: action.clone(),
            find: find.unwrap_or_default(),
            replace: repl.unwrap_or_default(),
            needs_find_replace,
            needs_action,
            sel_camel: action == "camel" || action.is_empty(),
            sel_snake: action == "snake",
            sel_kebab: action == "kebab",
            sel_pascal: action == "pascal",
            sidebar: sidebar_links(),
        },
    )
}

/// Most-specific tool route: carries the full URL state
/// (`?input=...&find=...&replace=...&action=...`) so the server-rendered
/// output matches what the page's client-side re-run would produce.
#[get("/tool/<slug>?<input>&<find>&<replace>&<action>", rank = 1)]
pub fn tool_find_replace(
    slug: &str,
    input: Option<String>,
    find: Option<String>,
    replace: Option<String>,
    action: Option<String>,
) -> Template {
    let input = input.unwrap_or_default();
    let output = tools::run(
        slug,
        &input,
        action.as_deref(),
        find.as_deref(),
        replace.as_deref(),
    );
    tool_context(slug, input, output, action, find, replace)
}

#[get("/tool/<slug>?<input>&<output>&<action>", rank = 2)]
pub fn tool_full(
    slug: &str,
    input: Option<String>,
    output: Option<String>,
    action: Option<String>,
) -> Template {
    let input = input.unwrap_or_default();
    let (output, action) = match output {
        Some(o) => (o, action),
        None => (
            tools::run(slug, &input, action.as_deref(), None, None),
            action,
        ),
    };
    tool_context(slug, input, output, action, None, None)
}

#[get("/tool/<slug>?<input>&<output>", rank = 3)]
pub fn tool_output(slug: &str, input: Option<String>, output: Option<String>) -> Template {
    let input = input.unwrap_or_default();
    let output = output.unwrap_or_else(|| tools::run(slug, &input, None, None, None));
    tool_context(slug, input, output, None, None, None)
}

#[get("/tool/<slug>?<input>", rank = 4)]
pub fn tool(slug: &str, input: Option<String>) -> Template {
    let input = input.unwrap_or_default();
    let output = tools::run(slug, &input, None, None, None);
    tool_context(slug, input, output, None, None, None)
}
