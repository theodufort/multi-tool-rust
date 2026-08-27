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

/// HTML list of `<li>` items for the sidebar.
pub fn sidebar_links() -> String {
    let mut s = String::from("        <ul class=\"nav flex-column\">");
    for (slug, label) in CATALOG {
        s.push_str(&format!(
            "          <li class=\"nav-item\">\n            <a class=\"nav-link\" href=\"/tool/{slug}\">{label}</a>\n          </li>\n"
        ));
    }
    s.push_str("        </ul>");
    s
}

// ---------------------------------------------------------------------------
// Machine endpoints: `/api/<slug>`
// ---------------------------------------------------------------------------

// Rank breaks the query-string collision: both routes can match a URL that
// carries an extra param, but Rocket only treats routes with an *equal* rank
// as colliding. The more specific route wins by having the lower rank.
// `rank` must differ from the mounted FileServer route (default rank 10,
// `/<path..>`), which otherwise collides with every path. Lower ranks are
// matched first, so the more specific routes win over their base routes.
#[get("/api/<slug>?<input>&<action>", rank = 0)]
pub fn api_with_action(slug: &str, input: Option<String>, action: Option<String>) -> String {
    tools::run(slug, input.as_deref().unwrap_or(""), action.as_deref())
}

#[get("/api/<slug>?<input>", rank = 1)]
pub fn api(slug: &str, input: Option<String>) -> String {
    tools::run(slug, input.as_deref().unwrap_or(""), None)
}

// ---------------------------------------------------------------------------
// UI endpoints: `/tool/<slug>`
// ---------------------------------------------------------------------------

#[get("/tool/<slug>?<input>&<output>&<action>", rank = 0)]
pub fn tool_full(slug: &str, input: Option<String>, output: Option<String>, action: Option<String>) -> Template {
    Template::render(
        "tool",
        context! {
            title: slug,
            input: input.unwrap_or_default(),
            output: output.unwrap_or_default(),
            action: action,
        },
    )
}

#[get("/tool/<slug>?<input>&<output>", rank = 1)]
pub fn tool_output(slug: &str, input: Option<String>, output: Option<String>) -> Template {
    Template::render(
        "tool",
        context! {
            title: slug,
            input: input.unwrap_or_default(),
            output: output.unwrap_or_default(),
            action: None::<String>,
        },
    )
}

#[get("/tool/<slug>?<input>", rank = 2)]
pub fn tool(slug: &str, input: Option<String>) -> Template {
    Template::render(
        "tool",
        context! {
            title: slug,
            input: input.unwrap_or_default(),
            output: String::new(),
            action: None::<String>,
        },
    )
}
