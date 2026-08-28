# Multi-Tool Rust

A self-contained, single-Cargo-package web application built with
[Rocket 0.5.1](https://rocket.rs) and
[rocket_dyn_templates](https://crates.io/crates/rocket_dyn_templates)
(Handlebars). It ships a dark left-hand sidebar of tool links. Clicking a
link opens a small, **stateless** developer text utility. Every tool is also
exposed as a machine-readable HTTP endpoint.

## Layout

- `src/main.rs` — Rocket configuration, mount, sidebar wiring.
- `src/routes/index.rs` — landing page with the sidebar.
- `src/routes/tools.rs` — route definitions and the sidebar link list.
- `src/tools/mod.rs` — the **pure-Rust transforms** (one `pub fn` per tool)
  plus their unit tests. No I/O, no Rocket dependency.
- `templates/` — Handlebars page templates.
- `check.sh` — objective completion check (build, unit tests, README, and a
  live server smoke test).

## Running

```bash
cargo build
cargo test
./check.sh
```

The server listens on port **8091** by default (set `port` in `Rocket.toml`).

## Tools

| Slug | Endpoint | Description |
|------|----------|-------------|
| replace | `/api/replace` | `?input=...&find=X&replace=_&action=N` — replace up to N (`all`) occurrences; no params = demo `X -> _` |
| replace-all | `/api/replace-all` | `?input=...&find=X&replace=_` — replace every occurrence; no params = demo `X -> _` |
| collapse | `/api/collapse` | collapse consecutive newlines to a single space |
| uppercase | `/api/uppercase` | uppercase the input |
| lowercase | `/api/lowercase` | lowercase the input |
| title | `/api/title` | title-case the input |
| reverse | `/api/reverse` | reverse the character order |
| trim | `/api/trim` | trim leading/trailing whitespace |
| url-encode | `/api/url-encode` | form-urlencoded (`+` for space) |
| url-decode | `/api/url-decode` | reverse of url-encode |
| html-encode | `/api/html-encode` | encode `<`, `>`, `&`, `"`, `'` |
| html-decode | `/api/html-decode` | reverse of html-encode |
| hex-encode | `/api/hex-encode` | ASCII bytes to hex string |
| hex-decode | `/api/hex-decode` | hex string to text |
| base64-encode | `/api/base64-encode` | bytes to base64 |
| base64-decode | `/api/base64-decode` | base64 to bytes |
| unicode-strip | `/api/unicode-strip` | drop non-ASCII characters |
| word-count | `/api/word-count` | count words/chars/lines/bytes |
| json-pretty | `/api/json-pretty` | pretty-print JSON |
| json-minify | `/api/json-minify` | compact JSON |
| slug | `/api/slug` | URL-friendly slug |
| case | `/api/case` | `?action=camel\|snake\|kebab\|pascal` (default `camel`) |
| lorem | `/api/lorem` | generate 5 lorem-ipsum words; input `words=3` or `3` sets the count |

## Endpoint contract

- `GET /api/<slug>?input=<urlencoded>` returns the raw transformed text.
- Optional extra params are chained with `&` (Rocket 0.5.1 route grammar:
  `query := segment ('&' segment)*`), e.g.
  `?input=aXbXc&find=X&replace=_&action=all`.
- `GET /tool/<slug>?input=...` renders the page with the transform output
  already computed server-side; the Run button re-runs client-side via the
  same `/api/<slug>` endpoint. The result box updates live while you
  type (debounced) and Copy puts it on the clipboard.
- `GET /tool/<slug>` renders the UI page with the result in
  `<pre id="result">`.
- `GET /tool/<slug>?input=...&find=X&replace=_&action=N` carries the full
  UI state in the URL — the Find/Replace fields and case action are
  pre-filled server-side and the output matches a client-side re-run.

## Design notes

- Transforms are pure functions in `src/tools/mod.rs`, fully unit-tested.
- Routes are registered most-specific-first (the `&<action>` arm before the
  bare `input` arm) so URLs carrying an extra parameter match the richer
  route.
- Each `#[get(...)]` handler has a unique function name to avoid Rocket 0.5.1
  macro collisions.
