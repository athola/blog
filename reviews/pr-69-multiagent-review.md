# PR #69 — Multi-agent review (consolidated)

**Reviewer**: `/pr-review-toolkit:review-pr all` (5 specialist agents in parallel)
**Agents**: code-reviewer, comment-analyzer, pr-test-analyzer, silent-failure-hunter, type-design-analyzer
**Companion**: this extends the prior `/sanctum:pr-review` comment above. The two CI blockers (B1 clippy, B2 wasm-bindgen) and four other findings (N1 `.attune`, N2 sitemap, N3 writeln consistency, N4 `Redirect` ergonomics, N5 `Content-Disposition`) from that review are NOT duplicated here — see [the consolidated review](https://github.com/athola/blog/pull/69#issuecomment-4414372246) for those.

The findings below are **net-new** from the specialist agents, ordered by severity.

---

## 🚨 Critical — adds blockers beyond the prior review

### MA-C1 — Atom `<updated>` uses `created_at` instead of `updated_at`
**File**: `server/src/utils.rs:719-728` (and feed-level `:679-682`)
**Surfaced by**: code-reviewer (confidence 88)

Per Atom 1.0 §4.2.15, `<updated>` is "the last time the entry was meaningfully modified." Both `<updated>` and `<published>` currently bind to `escape_xml(&post.created_at)`. The `Post` struct has `updated_at` (used correctly in `app/src/post.rs:360` for JSON-LD `dateModified`), so the data is available — just not threaded through. **Edited posts never appear "new" to feed readers checking `<updated>`**, defeating the contract feed clients use to decide what to re-fetch.

```diff
-    let _ = writeln!(feed, "    <updated>{}</updated>", escape_xml(&post.created_at));
+    let _ = writeln!(feed, "    <updated>{}</updated>", escape_xml(&post.updated_at));
     let _ = writeln!(feed, "    <published>{}</published>", escape_xml(&post.created_at));
```

Also propagate to feed-level `<updated>` at `:681`: use `posts.iter().map(|p| &p.updated_at).max()` rather than `posts.first().map(|p| p.created_at.clone())`.

### MA-C2 — `generate_atom` silently corrupts feed body on markdown failure
**File**: `server/src/utils.rs:710`
**Surfaced by**: silent-failure-hunter (severity Critical)

```rust
let processed_body = process_markdown(&post.body).unwrap_or_else(|_| post.body.clone());
```

If `process_markdown` fails for one post, raw markdown gets shoved into `<content type="html">`. Feed readers render `**bold**` and `# heading` as literal text; any `<` becomes entity-escaped; links break. **There is zero log of the failure** — no `error!`, no tracing. Six months from now, no one will know why one entry renders garbage in the user's RSS reader.

Minimum fix: `error!(?err, slug = %slug, "atom: process_markdown failed; skipping post"); continue;`. Skipping is preferable to publishing corrupted HTML; if you must include it, label `type="text"` not `type="html"`.

### MA-C3 — Spec / code drift on `/post/:slug.md` route shape
**Surfaced by**: comment-analyzer (severity Critical)

Implementation ships `/post/{slug}/raw.md` (`server/src/main.rs:316`) due to an Axum 0.8 limitation explained at `:312-315`. But **the spec, brief, and design-system all assert the simpler `/post/:slug.md`** at 9 locations. `app/src/post.rs:202` even contradicts itself — the `href` two lines earlier is `/post/{slug}/raw.md`, but the comment says "Pairs with the server route `/post/{slug}.md`".

| File:line | Asserts | Actual |
|---|---|---|
| `server/src/utils.rs:637` (banner) | `/post/:slug.md` | `/post/{slug}/raw.md` |
| `app/src/post.rs:202` (comment) | `/post/{slug}.md` | `/post/{slug}/raw.md` (visible at `:195`) |
| `docs/specification.md:186, 331, 341, 342, 521, 526, 664` | `/post/:slug.md` | wrong |
| `docs/design-system.md:162` | `/post/:slug.md` | wrong |
| `docs/project-brief.md:535` | `/post/:slug.md` | wrong |

Future readers wiring tooling against the spec (curl scripts, `<link rel=alternate>` validators, third-party client code) will get 404s. Either:
- Update the spec to match the implementation (recommended; what shipped is canonical), or
- Refactor the route — but `main.rs:312-315` documents the Axum 0.8 limitation as load-bearing, so option (a) is the realistic path.

`MISSION-REPORT.md:30` reference is OK as historical record.

---

## 🟡 Important — should fix in this PR or file follow-ups

### MA-I1 — `/random` integration test passes when handler is reverted to a stub
**File**: `tests/server_integration_tests.rs:1280-1300`
**Surfaced by**: pr-test-analyzer (revert-test failure)

The test accepts `302 | 404 | 503`. If `random_handler` were reverted to `async {StatusCode::NOT_FOUND}`, the test passes. The "no posts seeded in CI" escape hatch destroys the regression-protection value of the assertion. Either seed one post and require `302` + `Location: /post/...`, or split into a no-post case asserting the *exact* 404 body string `"No published posts to stumble through"` (which a stub would not produce).

### MA-I2 — `escape_xml` security claim has no end-to-end test in the Atom feed
**File**: `tests/server_integration_tests.rs:1255-1278`
**Surfaced by**: pr-test-analyzer (coverage gap)

The `escape_xml` invariant unit tests in `server/src/utils.rs` are excellent — but the integration test for `/feed/feed.xml` only asserts the namespace declaration. If `escape_xml(&processed_body)` at `generate_atom` were silently replaced with raw `&post.body` (an XSS regression), nothing catches it. Seed a post containing `<script>alert(1)</script>` in title or body, fetch the feed, and assert the body contains `&lt;script&gt;` and not `<script>`. This is the load-bearing security claim of the entire `escape_xml` test cluster.

### MA-I3 — `random_handler` collapses 3 failure modes into 2 misleading messages
**File**: `server/src/utils.rs:770-790`
**Surfaced by**: silent-failure-hunter

| Real cause | User sees |
|---|---|
| DB transport failure | `503 "No posts available"` (lies — DB is down, not "no posts") |
| Deserialize failure | `503 "No posts available"` (lies — schema/driver mismatch masquerading as empty data) |
| True empty result | `404 "No published posts to stumble through"` (truth) |

Operators using synthetic monitoring or status pages can't distinguish a real outage from intended state. The `error!` logs do exist — the issue is the user-facing body. Fix: generic `"Stumble unavailable, try again shortly"` for failure-modes #1 and #2; reserve the truthful "no posts" message for the actual empty case.

### MA-I4 — `random_handler` redirect can ship without a `Location` header
**File**: `server/src/utils.rs:806-808`
**Surfaced by**: silent-failure-hunter

```rust
if let Ok(location) = target.parse() { response.headers_mut().insert("Location", location); }
```

For today's input (`/post/{slug}` with DB-sourced slug), `HeaderValue::from_str` is *practically* infallible — but it is not infallible *by stdlib contract*. It rejects bytes outside `0x20..=0x7E` plus tab. If a future migration permits unicode slugs, or a slug ever contains a control char from a misbehaving import, this silently emits a `302` with **no `Location`**. Browsers display blank pages or hang.

The literal `"public, max-age=60"` and `"text/plain; charset=utf-8"` at `:809` and `:812` *are* known-good ASCII at compile time — switch to `HeaderValue::from_static(...)` (const-checked, infallible). The `if let Ok(...)` pattern at those sites incorrectly implies fallibility and trains future maintainers to copy the pattern for non-static values.

### MA-I5 — `build_article_jsonld` doesn't json-escape `post_url`
**File**: `app/src/post.rs:354-363`
**Surfaced by**: code-reviewer (confidence 82)

Every interpolated string passes through `json_escape()` except `post_url` (`:362`), interpolated raw. Slugs are URL-safe so practical risk is low — but a slug containing `"` or `\` (data-import anomaly, future migration) breaks the JSON-LD and Google Search Console rich-results stop working for that post. Defense-in-depth: `json_escape(&post_url)`.

### MA-I6 — `parse_date_pieces` silently degrades to `·` for unrecognized dates
**File**: `app/src/components/post_list_row.rs:126`
**Surfaced by**: silent-failure-hunter

For any date that doesn't match `YYYY-MM-DD`, the row renders day = `·`, meta = the raw string. **No `tracing::warn`, no `console_error`, no telemetry.** If a DB migration changes the date format, every post's date stamp silently breaks and no alarm fires. Fix: `tracing::warn!(raw = %created_at, "post_list_row: unrecognized date format")` (or `web_sys::console::warn` on the WASM side) before returning the fallback.

Same shape applies to `app/src/post.rs:307` `format_post_date`, which lacks the `parts[2].len() <= 2 && !is_empty()` guards that `parse_date_pieces` has — `"2026-04-345"` would render as `"April 345,"`. Two near-identical date parsers in the same crate; consolidate into a shared helper.

### MA-I7 — `atom_handler` `<updated>` = "now" when feed is empty
**File**: `server/src/utils.rs:679-682`
**Surfaced by**: silent-failure-hunter

```rust
.unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
```

RFC 4287 §4.2.15 requires `<updated>` reflect last *meaningful modification*. An empty feed updating every second on every poll thrashes downstream conditional-GET caches and signals "fresh content!" to aggregators that find nothing. Use a stable epoch (build time, deploy time, or `1970-01-01T00:00:00Z`) so `If-Modified-Since` works.

### MA-I8 — Notes page "load more" button clickable before initial load resolves
**File**: `app/src/notes.rs:104-119`
**Surfaced by**: code-reviewer (confidence 80)

On first paint, `all_notes` is empty, `is_loading` is false, `has_more` is true (defaults). The pagination footer renders the `load more →` button before page-0 data has arrived. A click during the in-flight initial fetch advances `current_page` to 1 — depending on resolve order, the user sees page-1 results appended before page-0, or page-0 missing entirely. Gate the button on `!all_notes.is_empty()`, or set `is_loading=true` initially until the first resolve fires.

### MA-I9 — `DateStampSize::Compact` is a dead variant with `#[allow(dead_code)]`
**File**: `app/src/components/date_stamp.rs:18-28`
**Surfaced by**: type-design-analyzer

`Compact` exists "for the explicit spec contract and future use" but `kicker_class()` at `:48` collapses `Default`/`Compact` via `_ =>`, so they render identically. `#[allow(dead_code)]` is the canonical anti-pattern that keeps a variant lying around for years. Either wire it (the doc says it's "spec'd for the home notes strip" — but the notes strip currently uses inline rows), or delete it.

### MA-I10 — `TocHeading::level: u8` should be `enum HeadingLevel { H2, H3 }`
**File**: `app/src/components/toc.rs:18-25`
**Surfaced by**: type-design-analyzer

`u8` for "h2 or h3" is wider than the domain. The runtime filter at `toc.rs:47` (`.filter(|h| h.level == 2)`) becomes a `match` against the enum, and `level: 47` becomes unrepresentable. Encapsulation 2/5, invariant expression 2/5 — invariants live in docs and runtime guards, not in the type.

### MA-I11 — `Post` shadowing in `sitemap_handler` is a maintainability landmine
**File**: `server/src/utils.rs:509`
**Surfaced by**: type-design-analyzer

A function-local `struct Post { slug, created_at }` shadows `app::types::Post` (12 fields). Two `Post` types with different fields in the same crate hierarchy is a paper cut waiting to confuse a `grep -rn "struct Post"` reader. Compare to `SlugOnly`/`BodyOnly` at `:754, :833` which already follow the projection-naming convention. Rename to `SitemapRow`.

### MA-I12 — Component tests are signature checks, not behavior tests
**Files**: `app/src/components/{nameplate,footer,pipe_nav,date_stamp,tag_strip,toc,post_list_row}.rs` (7 files)
**Surfaced by**: pr-test-analyzer (test-quality)

```rust
#[test]
fn test_nameplate_component_structure() {
    let _: fn() -> _ = component;  // <-- compile-time signature guard
}
```

These pass as long as the function exists with declared arity. They would NOT fail if the function body were `unimplemented!()` or returned wrong markup. The names (`test_*_component_structure`) imply more than they deliver. Two paths forward:

- **Rename for honesty**: `test_*_signature_compiles` (cheap, but underwhelming).
- **Add real markup assertions**: `leptos::ssr::render_to_string(component)` and assert on the resulting HTML string. More work, real coverage.

The genuinely behavioral component tests (`post_list_row.rs:174-198`, `pipe_nav.rs:90-99`, `date_stamp.rs:84-95`, `toc.rs:79-88`) are the model — they would fail on independent reverts of the implementation.

---

## 🟢 Suggestions

### MA-S1 — `/activity → /notes` is `Redirect::permanent` (308) but test accepts 301-or-308
**File**: `tests/server_integration_tests.rs:1224-1252`
**Surfaced by**: pr-test-analyzer

Comment justifies acceptance. Fine — but pin one. Today's code uses 308 (`Redirect::permanent`). A regression to 301 (which breaks some old crawlers) would go undetected. Assert `308` exactly; doc-comment can note "if migrating to 301, update here."

### MA-S2 — `escape_xml` invariant set could add CDATA / control-char cases
**Surfaced by**: pr-test-analyzer

The 5 chosen invariants (5 metachars, XSS, non-idempotence, ampersand independence, multibyte) are well-chosen. Genuine gaps:
- **Control characters** (`\x00`, `\x08`, `\x1f`) — XML 1.0 forbids most C0 controls; current impl passes them through, producing technically invalid feeds.
- **CDATA terminator `]]>`** — current escape_xml leaves it as-is. Not exploitable in attribute-context output, but worth a passing test that locks in the "we don't use CDATA anywhere" assumption.

### MA-S3 — `raw_markdown_handler` echoes slug in 404 body
**File**: `server/src/utils.rs:872-881`
**Surfaced by**: silent-failure-hunter

Reflected-content surface (no HTML context, so XSS is mild; log-injection / cache-key pollution possible). `"Post not found"` is enough.

### MA-S4 — Section-banner WHAT-comments in `home.rs` and `post.rs`
**Files**: `app/src/home.rs:128-159`, `app/src/post.rs` various
**Surfaced by**: comment-analyzer

`// Section kicker`, `// Note rows`, `// Notes link`, `// Featured post — full bleed within reading column` restate visible structure. Per CLAUDE.md: "Don't explain WHAT the code does, since well-named identifiers already do that." The next-line `A(...)` to `/archive` with text `"+ archive →"` needs no `// Archive link` preamble.

### MA-S5 — `let _ = writeln!()` reads identically to ignoring a real `Result`
**File**: `server/src/utils.rs:687-747`
**Surfaced by**: silent-failure-hunter

Acceptable per stdlib (`String::write_str` is infallible) — but teaches the wrong reflex. Either `.expect("String::write_str is infallible")` once at the top with a comment, or use `feed.push_str(&format!(...))` throughout. The current pattern reads identically to "we're ignoring this error because we got tired of handling it."

### MA-S6 — Brand-consistency question: nameplate `coxeterelement` vs everywhere-else `Alex Thola`
**File**: `app/src/components/nameplate.rs:18, 27-28`

This appears to be **intentional** (commit `fe6fd96 fix(nameplate): coxeterelement wordmark`; doc-comment at `:18` explicitly names the wordmark). Flagging as a question, not a defect: every other surface — `Title()` everywhere, JSON-LD `name:"Alex Thola"`, RSS title `"alexthola"`, footer copyright `"ALEX THOLA"`, domain `alexthola.com`, social handles — uses the real brand. Is the divergence intentional (stage-name in the masthead, real name elsewhere)? The `nameplate.rs:3` doc-comment "Renders 'alex *thola*'" contradicts the `:18` and `:27-28` actual implementation, suggesting either the doc-comment is stale or the wordmark drift was unintentional.

---

## Out of scope (file separately)

- Pre-existing `with_env_vars!` macro is a no-op (`server/src/utils.rs:963-989`) — already noted in prior review as out-of-scope.
- `app/src/components/{nameplate,footer,pipe_nav}.rs` `IA inconsistency`: PipeNav doesn't include `/contact` (which is in the footer), and home + `/archive` both render an h1 of "writing" — surfaced by code-reviewer but classed as a deliberate IA decision, not a code defect.

---

## What's working well (cross-agent observations)

- **`bounded_terminate`** (`tests/server_integration_tests.rs:778`) — exemplary. Short, well-commented about *why* (D-state lsof on WSL2, surrealkv directory vs. file). Directly addresses the failure mode rather than masking it.
- **`escape_xml` invariant tests** — the GIVEN/WHEN/THEN docstrings (especially `escape_xml_escapes_ampersand_independently_of_other_metacharacters`) are model TDD-style tests that prevent specific future "optimizations" from regressing the contract.
- **`parse_date_pieces` test set** (`post_list_row.rs:178-198`) — happy path, edge case, variant, unknown-format fallback; all four would fail on independent reverts.
- **Component module headers** (`nameplate.rs`, `pipe_nav.rs`, `date_stamp.rs`, `tag_strip.rs`, `footer.rs`, `toc.rs`) — uniformly explain *why* (pattern source, spec section, design rationale) rather than restating code.
- **`PostListSize` enum design** — correct choice over `bool`; the `stamp_size()` method couples `PostListSize` to `DateStampSize` in a way `bool` cannot. 5/5 enforcement.
- **Sprint-tagged commits + atomicity** — already noted in prior review; cross-agent reinforcement.

---

## Action plan

Fixing all of the above is far beyond a single review-cycle response. **Suggested triage** (combining with the prior review):

**Must land in this PR (blocking merge):**
1. B1 (clippy) — already in prior review (~2 lines)
2. B2 (Dockerfile pin) — already in prior review (~1 line)
3. **MA-C1 (`<updated>` → `updated_at`)** — 2-line fix, real RSS-reader regression risk
4. **MA-C2 (silent feed corruption on markdown failure)** — add `error!` + `continue`; real silent-failure with downstream user impact
5. **MA-C3 (route-shape spec drift)** — ~10 sed-able replacements across `docs/`, plus 2 comment fixes in `server/` and `app/`
6. N2 (sitemap missing 4 routes) — already in prior review

**Should land before merge (in-scope but classifiable as follow-up):**
7. MA-I1, MA-I2 — strengthen `/random` and Atom-XSS tests
8. MA-I3, MA-I7 — `random_handler` error message cleanup + Atom empty-feed `<updated>`
9. MA-I8 — notes page button gating
10. MA-I12 — at minimum, rename signature-only tests for honesty

**File as follow-up issues:**
11. MA-I4 (Header parse fallibility), MA-I9 (dead `Compact`), MA-I10 (TocHeading enum), MA-I11 (Post shadowing rename), MA-I5 (json-escape post_url), MA-I6 (date parser silent degradation), MA-S1–S6.
12. MA-S6 — ask the author about the brand divergence; if intentional, document the choice in `nameplate.rs:3` (currently contradicts `:18`).
