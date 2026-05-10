# alexthola.com Redesign — Implementation Plan

**Date**: 2026-04-29
**Author**: Alex Thola (assisted)
**Status**: Draft — feeds into execution
**Branch**: `site-redesign-0.2.0`
**Brief**: [`docs/project-brief.md`](./project-brief.md)
**Specification**: [`docs/specification.md`](./specification.md)

---

## 1. Plan Overview

This plan converts the specification into 32 ordered tasks across 5 sprints.
Each task is sized to ≤ 1 working session and has a concrete acceptance test
suitable for proof-of-work evidence at commit time.

### Scope discipline (additive-bias scan)

Before sequencing, I challenged every NEW item in the spec against
`Skill(imbue:scope-guard)` "do we need this now?" criteria:

| Item | Decision | Rationale |
|---|---|---|
| `/archive` route | KEEP | Replaces broken IA; spec §4.3 is well-defined |
| `/notes` route | KEEP | Renames `/activity`; necessary for IA cleanup |
| `/about` route | KEEP | Lifts existing `whoami` content; small surface |
| `/colophon` route | KEEP, simplified | Render via existing `/post/:slug` route as a special markdown post — no new component |
| `/random` route | KEEP | One server function; high personality return per LOC |
| `/post/:slug/raw.md` route | KEEP | Server route only; no component (Axum 0.8 forces the trailing-segment shape) |
| Three feed formats | DEFER **JSON Feed**, ship Atom + RSS | JSON Feed is nice but Atom + RSS cover 99% of readers |
| ThemeToggle UI | DEFER | Spec §5.9 explicitly defers; tokens land here, UI ships next branch |
| Newsletter slot component | KEEP | Static placeholder; no integration; unblocks PLAN.md Q2 |
| `Toc` component | KEEP | Improves long-post UX; ships only on > 1500-word posts |
| Visual regression screenshots | KEEP | Required by spec §7 quality gate |

The `/colophon` simplification and JSON Feed deferral reduce scope by ~3 task-
hours without losing user-facing value.

---

## 2. Architecture

```
                        site-redesign-0.2.0
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
   STYLE LAYER            COMPONENT LAYER           ROUTE LAYER
   (style/tailwind.css)   (app/src/components/)     (app/src/*.rs)
   - @theme tokens         - Nameplate              - home.rs
   - 3 webfonts            - PipeNav                - post.rs
   - prose rules           - DateStamp              - archive.rs (NEW)
   - icon styles           - PostListRow            - notes.rs (renamed)
                           - TagStrip               - references.rs
                           - Toc                    - about.rs (NEW)
                           - Footer                 - contact.rs
                                                    - colophon-post.md (data, NEW)
        │                       │                       │
        └───────────┬───────────┘───────────┬───────────┘
                    │                       │
              FOUNDATION              SERVER LAYER
              (app/src/lib.rs)        (server/src/*.rs)
              - shell unfix           - feeds.rs (Atom + RSS)
              - sitemap footer        - markdown_alt.rs
              - theme pre-paint       - random.rs
              - head additions        - activity_redirect.rs
                                      - colophon route
```

**Layer dependency direction**: Style → Components → Routes → Server.
- Server work (feeds, redirects) is independent of style and can run in parallel.
- Route work depends on Components.
- Components depend on Style.

---

## 3. File Structure

| File | Status | Sprint | Owner task |
|---|---|---|---|
| `style/tailwind.css` | Rewritten | 0 | T01, T02, T03, T04 |
| `app/src/lib.rs` | Modified | 0 | T06 |
| `app/src/components/header.rs` | Replaced by Nameplate + PipeNav | 1 | T07, T08 |
| `app/src/components/nameplate.rs` | NEW | 1 | T07 |
| `app/src/components/pipe_nav.rs` | NEW | 1 | T08 |
| `app/src/components/date_stamp.rs` | NEW | 1 | T09 |
| `app/src/components/post_list_row.rs` | NEW | 1 | T10 |
| `app/src/components/tag_strip.rs` | NEW | 1 | T11 |
| `app/src/components/footer.rs` | NEW | 1 | T12 |
| `app/src/components/toc.rs` | NEW | 1 | T14 |
| `app/src/components/icons.rs` | Modified (currentColor) | 0 | T04 |
| `app/src/home.rs` | Refactored | 2 | T15 |
| `app/src/post.rs` | Refactored | 2 | T16 |
| `app/src/references.rs` | Refactored | 2 | T17 |
| `app/src/contact.rs` | Refactored | 2 | T18 |
| `app/src/notes.rs` (renamed from `activity.rs`) | Refactored | 2 | T19 |
| `app/src/archive.rs` | NEW | 3 | T20 |
| `app/src/about.rs` | NEW | 3 | T21 |
| `markdown/colophon.md` | NEW data | 3 | T22 |
| `server/src/feeds.rs` (or update existing) | NEW or modified | 3 | T26 |
| `server/src/markdown_alt.rs` | NEW | 3 | T24 |
| `server/src/random.rs` | NEW | 3 | T23 |
| `server/src/redirects.rs` (or extend) | Modified | 3 | T25 |
| Server router config | Modified | 3 | T23–T26 |
| `docs/design-system.md` | NEW | 4 | T34 |
| `README.md` | Updated | 4 | T34 |
| `PLAN.md` | Updated | 4 | T34 |

**Files DELETED in execute phase**:
- All 28 `@font-face` Poppins blocks in `style/tailwind.css` (T03)
- The global `svg{fill:white!important}` rule and ~80 lines of `.icon-white` overrides in `style/tailwind.css` (T04)
- `app/src/components/header.rs` after replacement (T07/T08)

---

## 4. Sprint Breakdown

### Sprint 0 — Foundation (must complete first)

**Goal**: Tokens, fonts, shell, and lint are in place before any component work.
**Capacity**: ~1 working session (M effort total).

| ID | Task | Effort | FR | Test |
|---|---|---|---|---|
| **T01** | Declare Tailwind v4 `@theme` block with **light** color tokens | S | §2.1 | Visual inspection: existing `bg-[#1e1e1e]` still renders (coexist) |
| **T02** | Add **dark** color tokens to `@theme`, gated by `[data-theme="dark"]` selector | S | §2.1 | Manual: toggle `data-theme` on `<html>` in DevTools, verify dark palette applies |
| **T03** | Replace Poppins block with Fraunces + Inter + JetBrains Mono `@font-face` rules; add type-scale `@theme` entries | M | §2.2 | Build succeeds; webfont network requests show Latin subset only; total payload < 200KB |
| **T04** | Delete global `svg{fill:white!important}` + `.icon-white` blocks; migrate `icons.rs` SVGs to `fill="currentColor"`; update other `<svg>` callsites | M | §2.7 | Run `make build`; manually verify all icons (header, footer, post meta, references, contact form) render in correct token color in both light and dark |
| **T05** | Add `make lint-tokens` target: greps `app/src/**/*.rs` for `bg-\[#`, `text-\[#`, `border-\[#`, fails on hits | XS | §2 discipline | `make lint-tokens` passes after T01–T04; **temporarily allow-listed** until Sprint 2 finishes route refactor |
| **T06** | Refactor `app/src/lib.rs` shell: unfix header (`fixed top-0` → `relative`), unfix footer, set body `data-theme` from inline `<script>` reading `localStorage` then `prefers-color-scheme` (pre-paint, no FOUC), add `<link rel="alternate">` placeholders for feeds + markdown alt | M | §6.5, §3.2 | Manual: visit every route, scroll — header and footer both scroll with page; no FOUC on light/dark; `<head>` contains all 4 `rel="alternate"` links (Atom, RSS, JSON-Feed-or-skipped, markdown-on-post-pages-only) |

**Sprint 0 acceptance**: tokens compile, fonts load, no FOUC, no fixed chrome, icons render in correct color, lint check exists. Other than `lint-tokens` allow-list, no functional regression.

---

### Sprint 1 — Components

**Goal**: All new components scaffolded and unit-tested in isolation. No route changes yet.
**Capacity**: ~1.5 working sessions.

| ID | Task | Effort | FR | Test |
|---|---|---|---|---|
| **T07** | `app/src/components/nameplate.rs` — italic-accented two-piece nameplate | S | §5.1, §3.2 | Renders "alex" + italic accent "thola"; full nameplate is `<a href="/">`; mounted in lib.rs replacing existing header |
| **T08** | `app/src/components/pipe_nav.rs` — lowercase pipe-separated nav with `current_route` prop and outbound `↗ rss` | S | §5.2, §3.2 | Renders `writing | notes | references | about | rss ↗`; `current_route="/"` shows current state on "writing"; collapses correctly on mobile |
| **T09** | `app/src/components/date_stamp.rs` — date-stamp tile with `Featured`/`Default`/`Compact` size variants | M | §5.3, §4.1 | Three sizes render with correct font sizes (kicker 11/11/10, day 48/32/24); 2px accent left rail visible; mono uppercase tracking works |
| **T10** | `app/src/components/post_list_row.rs` — combines `DateStamp` + title + excerpt + meta + hairline rule | M | §5.4, §4.1, §4.3 | Test renders existing `Post` data without crash; grid is `130px 1fr`; mobile collapses; click navigates to `/post/:slug` |
| **T11** | `app/src/components/tag_strip.rs` — inline category strip with URL-driven selection | M | §5.5, §4.3, §4.4 | Renders as `topics: all · rust · leptos · …`; click updates URL; current tag has accent color + underline |
| **T12** | `app/src/components/footer.rs` — sitemap-style 3-column footer + social row + mono copyright | M | §5.8, §3.2 | Renders three columns, collapses to one on <640px; social icons use `↗` glyph; copyright reads `© 2024–2026 ALEX THOLA. POWERED BY RUST + LEPTOS.` |
| **T13** | Add CSS rule for outbound `↗` glyph: `[href^="http"]:not([href*="alexthola.com"])::after { content: " ↗"; ... }` | XS | §5.7 | Visual: any external link in nav, footer, or post body shows ↗; internal link does not |
| **T14** | `app/src/components/toc.rs` — in-flow TOC, conditional render | M | §5.6, §4.2 | Component takes `Vec<TocHeading>`; renders only if `len() >= 4`; produces accessible nested `<nav><ol>` with anchor links; "ON THIS PAGE" mono kicker visible |

**Sprint 1 acceptance**: All 8 components render in isolation when manually mounted. Each component has at least one structural unit test (matches existing `app/src/activity.rs` test pattern). No styling regressions in existing routes (since components aren't wired in yet).

---

### Sprint 2 — Route refactors

**Goal**: Existing five routes refactored to use new components and tokens. Old chrome removed. Lint allow-list cleared.
**Capacity**: ~1.5 working sessions.

| ID | Task | Effort | FR | Test |
|---|---|---|---|---|
| **T15** | Refactor `app/src/home.rs` per §4.1: featured post (DateStamp Featured), recent posts list (PostListRow), latest-notes strip (3 most recent from `select_activities(0)`), TagStrip filter at bottom | L | §4.1 | All ACs in §4.1; manual test: visit `/`, see featured + 5 recent posts + 3 notes; tag filter works reactively |
| **T16** | Refactor `app/src/post.rs` per §4.2: pre-title meta, italic date, h1, tag-byline + author chip, conditional Toc, prose styles, post-foot (prev/next + more-from-tag + raw-md/copy/share) | L | §4.2 | All ACs in §4.2; verify with: 1 short post (no TOC), 1 long post (TOC visible), 1 math post (KaTeX intact), 1 code-heavy post |
| **T17** | Refactor `app/src/references.rs` per §4.5: drop glassmorphism + grid bg + 2xl rounded; replace with single-column rows; mono `▰`/`▱` percentage bars | M | §4.5 | All ACs in §4.5; lint check finds zero `bg-[#ffef5c]` arbitrary values in this file |
| **T18** | Refactor `app/src/contact.rs` per §4.7: extract `whoami` block (move to `/about` in T21), keep form-only with new token-driven inputs, accent submit button | M | §4.7 | All ACs in §4.7; form still submits; success state renders |
| **T19** | Rename `app/src/activity.rs` → `app/src/notes.rs`; refactor per §4.4: replace inconsistent `bg-gray-800` / `text-blue-400` with tokens; outbound `↗` on note source links | M | §4.4 | All ACs in §4.4; URL `/notes` resolves; existing pagination + Resource pattern preserved |
| **T20** | Drop the lint allow-list from T05 — `make lint-tokens` must now pass cleanly | XS | §2 discipline | `make lint-tokens` exits 0 on the entire `app/src/**/*.rs` tree |

**Sprint 2 acceptance**: All five existing routes render with new components, no token-violation lint hits, KaTeX still works, `make build` succeeds, `make test` passes (existing test signatures preserved per §3 spec — APIs unchanged).

---

### Sprint 3 — New routes & server work

**Goal**: All NEW routes added; server routes (feeds, redirects, raw markdown, random) wired.
**Capacity**: ~1 working session.

| ID | Task | Effort | FR | Test |
|---|---|---|---|---|
| **T21** | `app/src/archive.rs` per §4.3: full chronological archive with year-grouped sections, `/archive?tag=<slug>` filter, 25-per-page pagination | L | §4.3 | All ACs in §4.3; `/archive` and `/archive?tag=rust` both render; pagination links work |
| **T22** | `app/src/about.rs` per §4.6: lifts `whoami` content from old `contact.rs`, adds links row + colophon link | M | §4.6 | All ACs in §4.6; avatar has dimensions to prevent CLS; links use outbound pattern |
| **T23** | `markdown/colophon.md` (data file) — colophon content per §4.8; route `/colophon` renders via post-detail layout (no new component file) | S | §4.8 | `/colophon` resolves and renders the markdown file at `/post/:slug` style; spec-deviating simplification approved by additive-bias scan above |
| **T24** | `server/src/random.rs` (or extend existing): `/random` returns 302 to a random published post slug, cached 60s | S | §4.9 | `curl -I /random` returns 302 with `Location: /post/:slug`; repeat call within 60s returns same slug |
| **T25** | `server/src/markdown_alt.rs` (or extend): `/post/:slug/raw.md` returns 200 with `Content-Type: text/markdown; charset=utf-8`, body is canonical markdown source | S | §4.11 | `curl /post/<existing-slug>/raw.md` returns 200 with text/markdown content type and raw markdown body; non-existent slug returns 404 |
| **T26** | `/activity` HTTP 301 → `/notes` redirect | XS | §4.4 | `curl -I /activity` returns 301 with `Location: /notes` |
| **T27** | Atom feed at `/feed/feed.xml` and RSS at `/feed/rss.xml` (top 50 posts, full content if ≤ 4096 chars else excerpt) — JSON Feed deferred per scope decision | M | §4.10 | Both feeds return 200 with correct Content-Type; valid against [W3C Feed Validator](https://validator.w3.org/feed/); top entry matches latest blog post |
| **T28** | `<head>` `<link rel="alternate">` for Atom + RSS on every page; `<link rel="alternate" type="text/markdown">` on post pages only | XS | §6.3 | View source on `/`, `/notes`, `/archive`: 2 alternates (Atom, RSS). View source on `/post/:slug`: 3 alternates (Atom, RSS, markdown) |

**Sprint 3 acceptance**: All NEW routes return 200; redirects return 301/302; feeds validate; `<head>` is correct per route type.

---

### Sprint 4 — Quality gates & documentation

**Goal**: Performance, accessibility, SEO, visual QA, docs. The "stop ship" sprint.
**Capacity**: ~0.5 working session.

| ID | Task | Effort | FR | Test |
|---|---|---|---|---|
| **T29** | Add `<a href="#main-content" class="skip-link">Skip to content</a>` and `id="main-content"` on `<main>`; ensure `prefers-reduced-motion` disables `duration-500` transitions | S | §6.2 | Tab from address bar shows skip link in upper-left; reduced-motion device disables hover transitions |
| **T30** | Add Schema.org JSON-LD: `Article` block on `/post/:slug` (with author, date, headline, image), `Person` block on `/about` | S | §6.3 | View source: valid JSON-LD passes [Schema.org Validator](https://validator.schema.org/) |
| **T31** | Run `make validate` (format + lint + test + build + security); fix all issues | M | §7 quality | All gates green; commit |
| **T32** | Run `Skill(scry:record-browser)` to capture before/after screenshots of every route on `master` vs `site-redesign-0.2.0`; save to `docs/redesign-screenshots/` | M | §7 quality | Screenshot pairs exist for `/`, `/post/:slug`, `/archive`, `/notes`, `/references`, `/about`, `/contact`, `/colophon` in both light and dark modes |
| **T33** | Lighthouse audit on `/` and `/post/:slug` (representative): Performance ≥ 90, Accessibility ≥ 95, Best Practices ≥ 90, SEO ≥ 95 | S | §6.1, §6.2 | Lighthouse reports archived as proof-of-work in PR description |
| **T34** | Update `README.md` to reflect new routes + design system; create `docs/design-system.md` (extract spec §2 as living doc); update `PLAN.md` to reflect what shipped | S | §7 docs | All three files committed; `docs/design-system.md` is self-contained reference |

**Sprint 4 acceptance**: All quality gates pass; visual evidence captured; docs updated. Branch ready for PR.

---

## 5. Critical Path

```
T01 → T02 → T03 → T04 → T06        (Sprint 0: foundation)
                  ↓
T07, T08, T09, T10, T11, T12, T14   (Sprint 1: components, parallelizable)
                  ↓
T15, T16                            (Sprint 2: home + post — biggest tasks)
                  ↓
T17, T18, T19, T20                  (Sprint 2: remaining routes + lint clear)
                  ↓
T21, T22, T23, T24, T25, T26, T27   (Sprint 3: new + server — parallelizable)
                  ↓
T29, T30, T31, T32, T33, T34        (Sprint 4: quality + docs)
```

**Critical-path tasks** (longest serial chain): T01 → T03 → T06 → T16 → T31 → T33.

If any task on the critical path slips, the branch slips. Other tasks have
slack and can be reordered.

---

## 6. FR-to-Task Coverage

Every functional requirement in the spec maps to ≥ 1 task:

| Spec FR | Tasks |
|---|---|
| §2.1 Color tokens | T01, T02 |
| §2.2 Typography | T03 |
| §2.3 Spacing | T01 (alongside color block) |
| §2.4 Border radius | T01 |
| §2.5 Rule patterns | T01 + per-component (T09, T10, T12, T14) |
| §2.6 Link patterns | T01 (CSS rules) + T13 (outbound glyph) |
| §2.7 Iconography | T04 |
| §3.1 Routes | T15–T28 |
| §3.2 Navigation | T07, T08, T12, T06 (header/footer wiring) |
| §3.3 Permalinks (preserved) | T26 (redirect) |
| §4.1 Home | T15 |
| §4.2 Post detail | T16 |
| §4.3 Archive | T21 |
| §4.4 Notes | T19, T26 |
| §4.5 References | T17 |
| §4.6 About | T22 |
| §4.7 Contact | T18 |
| §4.8 Colophon | T23 |
| §4.9 Random | T24 |
| §4.10 Feeds | T27, T28 |
| §4.11 Markdown alternate | T25, T28 |
| §5.x Components | T07–T14 |
| §6.1 Performance | T03 (font subset), T33 (Lighthouse) |
| §6.2 Accessibility | T29, T33 |
| §6.3 SEO | T28, T30 |
| §6.4 Browser support | implicit (Tailwind v4 + modern Leptos) |
| §6.5 Theme handling | T01, T02, T06 |
| §7 Master ACs | T31, T32, T33, T34 |

No orphan FRs.

---

## 7. Dependencies (DAG)

```
T01 ─┬─ T02 ─┬─ T05 ─ T20
     │       │
     ├─ T03 ─┤
     │       │
     ├─ T04 ─┤
     │       │
     └─ T06 ─┴─ T07 ─┬─ all components ── T15, T16, T17, T18, T19
                     │
                     └─ T13 (outbound glyph CSS, independent)

T15 ─┬─ T20 (lint clear) ─ T21, T22 (new routes)
T16 ─┘                     │
                           ├─ T23 (colophon, depends on T16's post-detail layout)
                           │
                           ├─ T24, T25, T26, T27 (server routes, parallel)
                           │
                           └─ T28 (head links, depends on T27 existing)

T29, T30 ─ depend on T16 (post route) and T22 (about route)
T31 ── all preceding tasks
T32 ── all preceding tasks  
T33 ── T31 (must build clean before Lighthouse)
T34 ── all preceding tasks
```

Acyclic. ✓

---

## 8. Effort & Capacity

| Sprint | Total effort | Sessions |
|---|---|---|
| Sprint 0 (Foundation) | ~5 task-units (S+S+M+M+XS+M) | ~1 |
| Sprint 1 (Components) | ~7 task-units (S+S+M+M+M+M+XS+M) | ~1.5 |
| Sprint 2 (Routes) | ~7 task-units (L+L+M+M+M+XS) | ~1.5 |
| Sprint 3 (New routes) | ~6 task-units (L+M+S+S+S+XS+M+XS) | ~1 |
| Sprint 4 (Quality) | ~3 task-units (S+S+M+M+S+S) | ~0.5 |
| **Total** | ~28 task-units | **~5.5 sessions** |

(Effort sizing: XS=15min, S=30min, M=60min, L=90–120min.)

A "working session" assumed to be ~3–4 task-units before context fatigue.

---

## 9. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Webfont payload exceeds 200KB even with subset | Medium | Medium | T03 tests payload size; if over, drop Fraunces variable axes (use 2–3 static weights instead) |
| Dark-mode contrast regression on accent | Medium | High | T02 verifies all token pairs at ≥ 4.5:1 BEFORE any component task starts |
| `prose` rewrite breaks KaTeX styling | Medium | Medium | T16 tests on math-heavy post explicitly; isolate `.katex` from new prose rules |
| Removing `svg{fill:white!important}` breaks an icon usage we miss | High | Low | T04 audits ALL `<svg>` callsites; visual smoke test of every page in both modes; revert is one CSS line |
| `T16` (post page) is a single L task — could exceed session | Medium | Medium | T16 split into sub-checkpoints internally: meta+title → tag-byline → toc → prose → post-foot; commit at each |
| Server route additions (feeds, markdown alt) require touching server crate we haven't read | Medium | Medium | Pre-task: read `server/src/lib.rs` and existing route registration BEFORE T24/T25/T27 |
| `T15` notes-strip pulls from `select_activities(0)` but home was previously activity-free → SSR latency increase | Low | Low | T15 includes Suspense fallback so notes strip loads non-blocking |
| Lighthouse Performance dropping below 90 due to dark-mode pre-paint script | Low | Medium | T06 inline pre-paint script kept under 1KB; alternative: `color-scheme: light dark` CSS-only fallback if JS budget breaks |
| Redirect from `/activity` → `/notes` invalidates external bookmarks | Low | Low | 301 (permanent) signals to search engines; preserves SEO equity |
| Atom + RSS feed validation fails in production but not dev | Medium | Medium | T27 includes W3C Feed Validator step; pre-PR check |
| Visual regression screenshots exhaust execution context | Low | Low | T32 batched per route; each save commit checkpoints |
| `make validate` fails on existing security scan due to webfont URL changes | Low | Medium | Run `make validate` early in Sprint 0 to catch noise before adding new code |

---

## 10. Working Agreement

Per CLAUDE.md and project conventions:

1. **Branch**: stay on `site-redesign-0.2.0`. No rebases against master without
   user confirmation.
2. **Commits**: one commit per task ID, conventional-commit format (`feat(home):
   refactor to featured + recent + notes layout (T15)` style). Plain message,
   no AI attribution.
3. **No `--no-verify`**: pre-commit hooks must pass. Fix issues, don't bypass.
4. **TDD where it makes sense**: add structural tests for new components (matching
   existing `app/src/activity.rs` test pattern). Visual/route-level testing is
   manual verification + screenshots.
5. **Proof-of-work per task**: after each task, capture evidence (command output,
   screenshot, test pass, validator URL hit) and reference in commit body.
6. **Iron Law application**: For state-bearing logic (post selection, tag filter,
   archive pagination), write a failing test first. For pure-style components
   (Nameplate, PipeNav, Footer), structural test only.
7. **Two-challenge rule**: if any task fails twice in the same shape, stop and
   report — don't try a third variation.
8. **Scope creep guardrail**: out-of-scope list (spec §8) is non-negotiable.
   Any temptation to "while we're here" goes to PLAN.md as a follow-up issue.

---

## 11. Pre-Execute Checklist

Before invoking execute phase:

- [ ] Plan reviewed by user (this is the natural pause-point per orchestrator's
      interactive plan review protocol)
- [ ] Spec §8 (out-of-scope) and plan §10 (working agreement) acknowledged
- [ ] `site-redesign-0.2.0` is up to date with master (no rebase needed currently)
- [ ] `make validate` passes on master baseline (so we can detect regressions
      caused by us, not pre-existing)
- [ ] Memory palace check: existing knowledge about Leptos SSR + Tailwind v4 ready
- [ ] Webfont source URLs decided (Google Fonts CDN vs self-host vs Fontsource —
      defaulting to **self-host via cargo-leptos public/fonts** for predictability)

---

## 12. Next Steps

After plan acknowledged:

1. **`/attune:execute`** auto-invoked per orchestrator protocol
2. Execute tasks T01 → T34 in sprint order
3. Each task: implement → test → commit → update task status
4. After Sprint 4: open PR with screenshot evidence + Lighthouse reports

The execute phase will:
- Run quality gates per the post-implementation protocol
- Apply proof-of-work + iron-law TodoWrite items per task
- Update docs (sanctum:update-docs, abstract:make-dogfood, sanctum:update-readme,
  sanctum:update-tests) before declaring complete

---

## Appendix A: Out-of-Scope Reminders

(Repeated from spec §8 for plan-phase task discipline. If you find yourself
writing code for any of these, STOP and create a follow-up issue.)

- ❌ Theme **toggle UI** (tokens land here; UI ships in follow-up)
- ❌ Newsletter integration (slot designed; integration is PLAN.md Q2)
- ❌ Comments system
- ❌ Site search
- ❌ Server-side syntax highlighting
- ❌ Mascot / illustrations
- ❌ Sticky / fixed chrome
- ❌ Cookie banner / analytics changes
- ❌ JS framework changes
- ❌ Static site generator migration
- ❌ Database schema migrations
- ❌ Tag detail pages at `/tag/:slug` (use `/archive?tag=<slug>`)
- ❌ JSON Feed (deferred per §1 additive-bias scan; Atom + RSS only)

## Appendix B: Sprint Commit Cadence Template

```
feat(style): add @theme block with light tokens (T01)
feat(style): add dark-mode token variants (T02)
feat(style): replace Poppins with Fraunces+Inter+JetBrains Mono (T03)
fix(style): remove svg{fill:white!important} global; migrate icons to currentColor (T04)
chore(make): add lint-tokens target with allow-list for in-flight refactor (T05)
refactor(shell): unfix header+footer; add theme pre-paint; add head feed links (T06)

feat(components): add Nameplate (T07)
feat(components): add PipeNav (T08)
feat(components): add DateStamp with three sizes (T09)
feat(components): add PostListRow (T10)
feat(components): add TagStrip (T11)
feat(components): add Footer (T12)
feat(style): add outbound link ↗ glyph rule (T13)
feat(components): add Toc with conditional render (T14)

refactor(home): featured + recent + notes-strip layout (T15)
refactor(post): meta+title+toc+prose+post-foot (T16)
refactor(references): drop glassmorphism for editorial rows (T17)
refactor(contact): form-only; bio extracted to /about (T18)
refactor(notes): rename activity, align palette (T19)
chore(make): drop lint-tokens allow-list (T20)

feat(archive): add /archive route with year-grouped pagination (T21)
feat(about): add /about route with bio + links (T22)
feat(colophon): add /colophon as markdown-driven post (T23)
feat(server): add /random stumble redirect (T24)
feat(server): add /post/:slug/raw.md raw markdown alternate (T25)
feat(server): add /activity → /notes 301 redirect (T26)
feat(server): add Atom + RSS feeds (T27)
feat(seo): head rel=alternate for feeds + markdown (T28)

feat(a11y): skip-to-content + reduced-motion (T29)
feat(seo): JSON-LD Article + Person (T30)
chore: make validate green (T31)
docs: visual regression screenshots (T32)
docs: Lighthouse audit reports (T33)
docs: update README + design-system.md + PLAN.md (T34)
```
