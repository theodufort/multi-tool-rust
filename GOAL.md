# GOAL: Multi-Tool Text Utility Web App (Rust + Rocket)

This spec guides an unattended build of a single Rust web app that serves many
small developer text utilities from one sidebar. Continuous improvement runs
until the operator stops the loop. Re-run `check.sh` after every change; the
loop ends when `SCORE` reaches the target and all criteria pass.

## Refined objective
Build a self-contained Rocket web server (`multi-tool-rust`) with a dark sidebar
of tool links. Each link opens a small, single-purpose text tool. Tools perform
client-side-safe, stateless text transforms (no auth, no persistence). The app
must be one Cargo package, build cleanly, pass tests, and expose deterministic
results.

## Scope
### In scope
- One Cargo binary. Rocket `=0.5.1`, `rocket_dyn_templates` with Handlebars.
- Left sidebar listing every tool; clicking a tool loads its page.
- Every tool has (a) a pure-Rust transform function, (b) a machine endpoint,
  (c) a rendered UI page, and (d) unit tests.
- Deterministic, dependency-light transforms (no network, no DB).

### Tool catalog (all required for full completion)
| Slug | Name | Action params | Example: input -> output |
|------|------|---------------|--------------------------|
| replace | Find & Replace | `find`, `replace`, `count`(default 1) | `aXbXc` -> `a_bXc` |
| replace-all | Replace All | `find`, `replace` | `aXbXc` -> `a_b_b` |
| collapse | Remove Line Breaks | none | `a\n\n  b\tc` -> `a b c` |
| uppercase | Uppercase | none | `hello` -> `HELLO` |
| lowercase | Lowercase | none | `HELLO` -> `hello` |
| title | Title Case | none | `the quick brown` -> `The Quick Brown` |
| reverse | Reverse | none | `abc 123` -> `321 cba` |
| trim | Trim | none | `  hi  ` -> `hi` |
| url-encode | URL Encode | none | `a b&c` -> `a+b%26c` |
| url-decode | URL Decode | none | `a+b%26c` -> `a b&c` |
| html-encode | HTML Encode | none | `a<b>"` -> `a&lt;b&gt;"` |
| html-decode | HTML Decode | none | `a&lt;b&gt;"` -> `a<b>"` |
| hex-encode | Hex Encode | none | `AB` -> `4142` |
| hex-decode | Hex Decode | none | `4142` -> `AB` |
| base64-encode | Base64 Encode | none | `Man` -> `TWFu` |
| base64-decode | Base64 Decode | none | `TWFu` -> `Man` |
| unicode-strip | Unicode -> ASCII | none | `Héllo Wörld` -> `Hllo Wrld` |
| word-count | Word/Char/Line/Byte Count | none | `one two three` -> words:3 chars:11 lines:1 bytes:11 |
| json-pretty | JSON Pretty Print | none | `{"a":1,"b":2}` -> `{  "a": 1,\n  "b": 2 }` |
| json-minify | JSON Minify | none | `{ "a": 1 }` -> `{"a":1}` |
| slug | Slugify | none | `Hello World & Friends!` -> `hello-world-friends` |
| case | Case Converter | `action=camel\|snake\|kebab\|pascal` | `hello world foo` -> `helloWorldFoo` |
| lorem | Lorem Ipsum | `words`(default 5) | `words=3` -> `lorem ipsum dolor` |

## Non-goals
- No authentication, rate limiting, or persistent storage.
- No server-side rendering of tool *logic* (logic is pure Rust functions).
- No heavy third-party crates beyond the allowed template engine.
- No external network calls from tools.
- No mobile-specific JS frameworks; plain HTML/CSS/JS is fine.

## Architecture constraints
- Pure transforms live in `src/tools/mod.rs` (one function per tool, `pub fn`).
- Machine endpoint `GET /api/<slug>?input=<enc>&[action=...]=<enc>` returns the
  raw transform output as plain text (no HTML). This is the testable contract.
- UI endpoint `GET /tool/<slug>?input=...&output=...` renders a page whose
  `<pre id="result">` contains the same output.
- No trailing-newline differences: `/api/<slug>` output is trimmed of a single
  trailing `\n` before comparison.
- Default port `8099` (override with `--port`).

## Measurable completion criteria (objective checks)
1. `cargo build` succeeds with no errors.
2. `cargo test` passes and reports **>= 30** test cases.
3. `README.md` exists and is > 30 lines, documenting build/run + the catalog.
4. Every catalog slug above is implemented and returns the documented output.
5. Sidebar links resolve to real `/tool/<slug>` pages (no `href="#"` left).
6. `check.sh` prints `SCORE: <n>` and exits 0 with `n >= 90`.

## Milestone roadmap (continuous improvement)
- **M1 (core text):** replace, replace-all, collapse, uppercase, lowercase,
  title, reverse, trim. Score target: implement all 8.
- **M2 (encoding):** url-encode/decode, html-encode/decode, hex-encode/decode,
  base64-encode/decode, unicode-strip. Adds 5 slugs.
- **M3 (transforms & counts):** word-count, json-pretty, json-minify, slug,
  case, lorem. Adds 6 slugs.
- After each milestone: add tests, docs, commit, re-run `check.sh`. Higher
  SCORE = better. Do not stop until SCORE >= 90 and criteria 1-5 all pass.

## Quality standards
- **Tests:** every tool function has unit tests with the table examples. `cargo
  test` must pass. Aim for >= 30 cases total.
- **Docs:** `README.md` with build/run commands, architecture note, and the full
  catalog table. Each tool page has a one-line description.
- **Git:** one focused commit per milestone/tool group. Message format:
  `feat: add <slug> tool` or `test: cover <slug>`. Keep working tree clean after
  each milestone.
- **Style:** consistent naming (`snake_case` fns, `kebab-case` slugs). Run
  `cargo fmt` before committing.

## Explicit assumptions
- Rust toolchain >= 1.85 (edition 2024) and Rocket 0.5.1 are available offline
  or via the existing `Cargo.lock`.
- `curl` and `bash` are present for `check.sh`.
- Transforms operate on UTF-8 bytes; ASCII examples are a subset.
- `title` capitalizes the first char of each whitespace-delimited segment and
  lowercases the rest of that segment.

## Verification
Run the completion check:

    ./check.sh

It builds, tests, starts the server, hits every `/api/<slug>` endpoint, checks
the sidebar, computes `SCORE`, and exits 0 when `SCORE >= 90`.
