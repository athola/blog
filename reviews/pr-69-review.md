# PR Review — #69 Site redesign 0.2.0 — Dual-Mode Editorial Engineer

**Reviewer**: `/sanctum:pr-review` (scope-mode: standard)
**Branch**: `site-redesign-0.2.0` → `master`
**Diff**: +6093 / −1506 across 40 files, 27 commits, 14 days
**CI state**: `mergeStateStatus: UNSTABLE` — 3 failing required checks on HEAD `94b4a28`
**Verdict**: **Request changes** — 2 blocking CI failures with mechanical fixes; otherwise the work is well-scoped, well-documented, and substantively sound.

---

## Scope baseline

The mission artifacts (`docs/project-brief.md`, `docs/specification.md`, `docs/implementation-plan.md`, `docs/MISSION-REPORT.md`) define a clear contract: **Direction D — Dual-Mode Editorial Engineer**, light-default token system, 3 new client routes (`/archive`, `/about`, `/colophon`), 3 new server handlers (`atom_handler`, `random_handler`, `raw_markdown_handler`), `/activity → /notes` rename with 301, and 8 new components. The diff matches the contract — no scope creep into theme-toggle UI, newsletter integration, JSON Feed, comments, or syntax highlighting (all explicitly deferred per spec §8 and plan §1).

The author's "Footnote on size" pre-empts the scope-guard RED warning (7,599 lines / 14 days / 18 new files): splitting after the fact would land partial-state intermediate commits where the design system is half-applied. **I accept that argument.** The intra-branch sprint-tagged commit walk (`T01…T34`) is a more reviewable unit than the +6093/−1506 mega-diff, and the mission-state JSON makes the sprint boundaries machine-readable.

## Quality gates summary

| Gate | Author's claim | Actual CI result | Notes |
|---|---|---|---|
| `cargo check --workspace` | ✓ 40.6s | ✓ (passes in `Analyze (rust)`) | clean |
| `cargo test -p server --bin server utils::tests` | ✓ 17/17 | ⏳ in progress | waiting |
| `cargo test --test server_integration_tests test_redesign_routes_sprint3` | ✓ 12.5s | ⏳ in progress | waiting |
| `make lint-tokens` | ✓ no `bg-[#hex]` | not in CI matrix | local-only gate |
| **`cargo clippy --all-targets -- -D warnings`** | (not listed) | **❌ FAILURE** | **two errors in the new test** |
| **Build Docker Image** | (not listed) | **❌ FAILURE** | **wasm-bindgen schema mismatch** |
| Stale `rustblog_test_*.db` (before/after) | 0/0 (was 219+) | not measurable in CI | great fix, can't verify in this review |

The PR description's quality-gates table omits the two gates that are actually failing in CI. That's the central process gap — see **Blocking #3** below.

---

## Blocking issues

### B1 — Clippy errors in `test_redesign_routes_sprint3` (PR-introduced)

**Files**: `tests/server_integration_tests.rs:1268`, `tests/server_integration_tests.rs:1296`
**Lint**: `clippy::useless_borrows_in_formatting`
**Commit that introduced it**: `c33caed test(server): escape_xml invariants, bounded test cleanup, redesign routes`

```
error: redundant reference in `assert!` argument
    --> tests/server_integration_tests.rs:1268:13
1268 |             &body.chars().take(120).collect::<String>()
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     | help: remove the redundant `&`
```

Same shape at `:1296` with `take(200)`. Workspace builds with `-D warnings`, so this fails the `Rust Compilation Test` and `clippy` jobs and blocks merge.

**Fix** (mechanical, ~2 lines):

```diff
-            "/feed/rss.xml body should look like XML/RSS, got: {}…",
-            &body.chars().take(120).collect::<String>()
+            "/feed/rss.xml body should look like XML/RSS, got: {}…",
+            body.chars().take(120).collect::<String>()
```

```diff
-            "Atom feed must declare the Atom namespace, body starts: {}…",
-            &body.chars().take(200).collect::<String>()
+            "Atom feed must declare the Atom namespace, body starts: {}…",
+            body.chars().take(200).collect::<String>()
```

### B2 — Docker build broken: wasm-bindgen pin drift (PR-introduced)

**Symptom** (from the `Build Docker Image` log):

```
wasm-bindgen failed with:
  rust Wasm file schema version: 0.2.120
     this binary schema version: 0.2.114
```

**Root cause**: this branch's `Cargo.lock` resolved `wasm-bindgen` to `0.2.120` (master pins `0.2.114`), but `Dockerfile:19` still installs the matching CLI at the old version:

```
master:    wasm-bindgen 0.2.114  ⇄  Dockerfile pins wasm-bindgen-cli 0.2.114  ✓
this PR:   wasm-bindgen 0.2.120  ⇄  Dockerfile pins wasm-bindgen-cli 0.2.114  ✗
```

The lockfile bump appears to have come in via `44a4ebd chore(cargo): scope surrealdb features per-crate` when cargo regenerated transitive dependencies. The Dockerfile pin was not updated to follow.

**Two mechanical fixes** — pick one:

1. **Bump the Dockerfile pin** (forward-going): in `Dockerfile:19`, change `wasm-bindgen-cli --version 0.2.114` → `--version 0.2.120`.
2. **Pin cargo to the older version** (revert-style): `cargo update -p wasm-bindgen --precise 0.2.114 && cargo update -p wasm-bindgen-shared --precise 0.2.114`, then commit the lockfile.

I recommend (1) — moving forward avoids re-introducing the same drift the next time a transitive dep nudges wasm-bindgen.

### B3 — PR-process gap: `cargo clippy --all-targets` not in the verified gate set

The mission-state JSON's `verification_evidence` claims `cargo_clippy_make_lint: "6.45s clean (no warnings, -D warnings)"` — but that was captured **before** commit `c33caed` (post-mission hardening) added the failing test. The hardening pass added invariant tests (a good thing) but did not re-run the full verification matrix that the mission ran.

This is the recurring failure mode of declaring a project "done" and then iterating: the hardening commits don't re-trigger the gates the mission ran. To prevent recurrence:

- Add `cargo clippy --all-targets --workspace -- -D warnings` to `make validate` if it isn't already (the README says `make validate` runs "formatting, linting, tests, and security scans" — verify clippy is in there with `--all-targets`, not just the default targets).
- Treat any post-mission commit on a mission branch as a re-validation trigger.

Not a code defect, but a process finding worth a one-paragraph follow-up issue. **Marking as blocking only because B1 demonstrates the gap concretely; if B1 is fixed and clippy is added to `make validate`, B3 self-resolves.**

---

## Non-blocking improvements (in scope)

### N1 — `.attune/mission-state.json` committed to repo

`docs/MISSION-REPORT.md` lists this file under "Artifacts" with the rationale "Mission state for resume/audit". That's a defensible reason, but `.attune/` is also the runtime state directory for the attune plugin and not in `.gitignore`. Two cleaner options:

- Add `.attune/` to `.gitignore` and move the snapshot to `docs/mission-state.json` (where the artifact-intent is unambiguous), or
- Keep at the current path but explicitly add an exception in `.gitignore` (e.g., `!.attune/mission-state.json`) so future `.attune/` runtime files don't accidentally land in commits.

### N2 — Sitemap missing the new routes

`server/src/utils.rs::sitemap_handler` static URL list (around line 561):

```rust
let static_urls = vec![
    ("https://alexthola.com/", "daily", "0.9"),
    ("https://alexthola.com/contact", "weekly", "1.0"),
    ("https://alexthola.com/references", "weekly", "0.6"),
    ("https://alexthola.com/rss.xml", "daily", "0.5"),
    ("https://alexthola.com/sitemap.xml", "monthly", "0.5"),
];
```

The four new public routes — `/archive`, `/notes`, `/about`, `/colophon` — are missing. The PR adds them to the IA but Google won't discover them through the sitemap. Author flagged this in their own MISSION-REPORT follow-up #5. Worth landing in this PR rather than as a follow-up because the sitemap is the SEO contract for the new IA — shipping the routes without indexing them is a 2-week regression window for organic discovery.

### N3 — Atom feed: silent error swallowing on `writeln!` to `String`

`server/src/utils.rs` lines 687–747 use `let _ = writeln!(feed, ...)` throughout `generate_atom`. Writing to `String` via the `core::fmt::Write` impl is infallible by stdlib contract (`String::write_str` always returns `Ok`), so this is technically safe. But the same file's `sitemap_handler` (lines 572–630) uses `if let Err(err) = writeln!(...)` with full error propagation for the same operation. Either pattern is defensible; the inconsistency between the two handlers is what's worth picking. I'd vote to drop the `Err` branches in `sitemap_handler` (since they're unreachable for `String`) rather than add them to `generate_atom`. Suggestion only.

### N4 — `random_handler` builds `Response` manually instead of using `axum::response::Redirect`

`server/src/utils.rs` lines 803–815 hand-build a 302 with `*response.status_mut() = StatusCode::FOUND` and a manual `Location` header. The same file uses `Redirect::temporary` and `Redirect::permanent` elsewhere (cleanly, in `main.rs`). The hand-rolled version does add a `Cache-Control: public, max-age=60` header that `Redirect::temporary` wouldn't, so there's a real reason to keep manual control — but a comment naming that reason would prevent the next reader from "simplifying" it. Suggestion only.

### N5 — `raw_markdown_handler` — no Content-Disposition

`server/src/utils.rs` line 873 returns `body` with `Content-Type: text/markdown; charset=utf-8`. That'll render in the browser. Most "show me the raw source" handlers also set `Content-Disposition: inline; filename="<slug>.md"` so a `Save As` from the browser produces a sensibly-named file. Tiny UX nicety; not load-bearing.

### N6 — Documentation slop scan: light em-dash density

5 new docs (`project-brief`, `specification`, `implementation-plan`, `MISSION-REPORT`, `design-system`) — total ~17K words. **Slop score: ~2/10 (Light)**.

| File | Em-dash density | Top markers |
|---|---|---|
| `docs/specification.md` | 1.50 / 100 words | em-dash (load-bearing in tables) |
| `docs/MISSION-REPORT.md` | 1.48 / 100 words | em-dash, ✓ checkmarks (semantic — WCAG/gate markers) |
| `docs/project-brief.md` | 1.33 / 100 words | em-dash |
| `docs/design-system.md` | 0.94 / 100 words | borderline, mostly clean |
| `docs/implementation-plan.md` | 0.61 / 100 words | clean |

**Zero hits** for the canonical AI-slop vocabulary (`leverage`, `comprehensive`, `robust`, `delve`, `seamless`) across all 5 docs. The em-dash density is elevated vs human-typical (~0.3/100w) but not pathological, and the content is concrete-technical (token tables, AC checklists, route specs) rather than hype-padded. Non-blocking. If you want to tighten, run `/scribe:slop-detector` later, but I wouldn't gate the merge on it.

---

## Out-of-scope (route to backlog)

- **Theme-toggle UI** — tokens land here, UI is the `next branch` (per spec §8). Already correctly deferred.
- **Self-host webfonts** (Google Fonts CDN → `/public/fonts/`) — MISSION-REPORT follow-up #2; estimated 1 hour. Defer.
- **W3C Atom feed validator** — only meaningful post-deploy; defer to deploy verification.
- **JSON Feed** — explicit spec deferral.
- **Newsletter integration** — explicit spec deferral, slot reserved.
- **Pre-existing `with_env_vars!` macro is a no-op** in `server/src/utils.rs:963–989` — the macro's "result" block is empty, so the test code that should run between the env-set and env-restore phases never executes. The single caller (`test_connect_env_var_defaults`) re-reads env vars *outside* the macro, so the test asserts against the original env, not the simulated env. The test currently passes by accident. **This is pre-existing — not in this PR's diff** — so it's correctly out-of-scope, but worth filing as a separate issue. (Recommended title: "test: `with_env_vars!` macro is a no-op; `test_connect_env_var_defaults` doesn't actually test what it claims".)

---

## Code-quality observations

### Things this PR does well

- **Sprint-tagged commits (`T01–T34`)** make the +6093 diff actually reviewable. Every commit has a single concern; the message references the spec/plan task it's executing. This is the closest thing to "good big-PR hygiene" I've seen on this repo.
- **`make lint-tokens`** as an enforcement gate against arbitrary `bg-[#hex]` values is a great institutional defense against the design system rotting over time. The allow-list pattern (with `chore(make): clear lint-tokens allow-list across app crate (T20)` clearing it once the migration was done) is exactly right.
- **`escape_xml` invariant tests** (XSS payload, multi-byte Unicode, intentional non-idempotence, ampersand independence) cover the right surfaces. The "intentional non-idempotence" test is the kind of thing that prevents a future "let's make it idempotent" refactor from silently breaking the contract.
- **Bounded test cleanup** fixing the leaking `rustblog_test_*.db` directories (219+ stale dirs → 0) is a real platform fix, not just a code-cleanup. That's a long-tail bug.
- **Per-crate `surrealdb` feature scoping** in `44a4ebd` (workspace `default-features = false`, opt-in per crate) is a load-bearing improvement for build times and downstream consumer surface area.

### Things to consider

- `app/src/components/post_list_row.rs::parse_date_pieces` is solid (the `month_abbr` table, the `is_ascii_digit` guards, the `("·", raw)` fallback for malformed dates). The fallback gracefully degrades for invalid timestamps — exactly the right shape for view-layer code that shouldn't panic on bad data.
- The `raw_markdown_handler` URL had to land at `/post/{slug}/raw.md` instead of `/post/{slug}.md` because Axum 0.8 disallows mixing literal extensions with path params in the same segment. Worth noting because the PR description and spec both say `/post/:slug.md` — the actual URL is different. The server includes the comment explaining the constraint, but consider updating the spec/brief to match the implementation, or vice versa, so the artifact-of-record matches what's deployed.

---

## Test plan (separate comment)

Posted as a follow-up comment with the verification checklist.

---

## PR description

The existing PR description is comprehensive, accurate (modulo the missing clippy gate), and cites the right artifacts. No update required beyond fixing the quality-gates table to include the failing gate. I will not re-write the description; I'll add a note in a comment.

---

## Verdict

**Request changes** — but the path to "approve" is short:

1. Fix B1 (2-line clippy fix in the integration test).
2. Fix B2 (1-line Dockerfile pin bump OR a `cargo update -p wasm-bindgen --precise`).
3. Land N2 (sitemap entries for the 4 new routes) inside this PR rather than as a follow-up.
4. Decide on N1 (gitignore policy for `.attune/`).

Total estimated effort: **~30 minutes**. Once B1 and B2 are resolved and CI is green, this is a clean merge — the work is well-scoped, well-tested where it counts, well-documented, and the design-system migration is the kind of disciplined surface change that makes future iteration easier.
