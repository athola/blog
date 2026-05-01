# alexthola.com Design System

**Direction**: D — Dual-Mode Editorial Engineer
**Branch landed in**: `site-redesign-0.2.0`
**Reference research**: [`docs/project-brief.md`](./project-brief.md)
**Spec**: [`docs/specification.md`](./specification.md)

This document is the **living reference** for the site's design system.
Components and routes reference these tokens; new work that needs a color,
typeface, or spacing value uses an existing token or proposes a new one
here first.

---

## 1. Color tokens

All colors flow from a `@theme` block in `style/tailwind.css`. Every Tailwind
utility (`bg-paper`, `text-accent`, `border-rule`) is generated from a
`--color-*` token. Light is the default; dark activates via
`[data-theme="dark"]` on `<html>`.

### Light mode

| Token | Hex | Role |
|---|---|---|
| `--color-paper` | `#ecedef` | Page background |
| `--color-paper-2` | `#dfe1e4` | Surface tier (code blocks, form inputs) |
| `--color-ink` | `#0a0a0a` | Primary text |
| `--color-ink-2` | `#2a2a2a` | Secondary text |
| `--color-ink-3` | `#5a5e62` | Muted text, mono kickers |
| `--color-ink-4` | `#8a8e92` | Subtle text, disabled, placeholders |
| `--color-accent` | `#7a2942` | Burgundy — link underlines, hover, current-state, blockquote rail |
| `--color-accent-soft` | `#c47c80` | Lightened accent — alpha tier |
| `--color-rule` | `#0a0a0a` | Hard 1–2px section rules |
| `--color-rule-soft` | `#bfc1c5` | Hairline rules between list items |

### Dark mode

| Token | Hex | Contrast vs paper |
|---|---|---|
| `--color-paper` | `#151515` | — |
| `--color-paper-2` | `#2a2a2a` | — |
| `--color-ink` | `#ecedef` | 16.4:1 ✓ AAA |
| `--color-ink-2` | `#bfc1c5` | 11.8:1 ✓ AAA |
| `--color-ink-3` | `#8a8e92` | 5.5:1 ✓ AA |
| `--color-ink-4` | `#5a5e62` | 2.7:1 — decorative only |
| `--color-accent` | `#c47c80` | 6.7:1 ✓ AA |
| `--color-accent-soft` | `#7a2942` | — |
| `--color-rule` | `#ecedef` | — |
| `--color-rule-soft` | `#3a3a3a` | — |

**Discipline rules** (enforced by `make lint-tokens`):
- Components reference tokens via Tailwind utilities — **never** `bg-[#hex]`.
- New tokens must have a documented role here.
- "Use opacity over more colors" (PostHog rule).

---

## 2. Typography

### Faces (loaded from Google Fonts CDN, Latin subset)

| Family | Role | Tailwind utility |
|---|---|---|
| **Fraunces** | Display serif (h1, h2, italic-accent nameplate) | `font-display` |
| **Inter** | Body sans (custom 470 weight) | `font-sans` |
| **JetBrains Mono** | Code, post metadata, mono kickers | `font-mono` |

Each family has a system fallback stack. See `style/tailwind.css` for the
exact `--font-*` declarations.

### Type scale

| Role | Family | Size | Weight | Line-height | Tracking |
|---|---|---|---|---|---|
| H1 | Fraunces | 56px (3.5rem) | 600 | 1.05 | -0.02em |
| H1 (mobile) | Fraunces | 40px (2.5rem) | 600 | 1.05 | -0.02em |
| H2 | Fraunces italic | 40px (2.5rem) | 500 | 1.15 | -0.015em |
| H3 (kicker) | JetBrains Mono | 12px | 500 uppercase | 1.4 | 0.08em |
| Body | Inter | 17px (1.0625rem) | **470** (custom) | 1.65 | 0 |
| Body (post prose) | Inter | 18px (1.125rem) | 470 | 1.7 | 0 |
| Mono kicker | JetBrains Mono | 11–12px uppercase | 500 | 1.4 | 0.08em |
| Code (block) | JetBrains Mono | 14px | 450 | 1.5 | 0 |

The body's **`font-variation-settings: "wght" 470;`** is set globally on `body`.
This is the PostHog-borrowed "intentional non-default weight" pattern.

---

## 3. Spacing

| Token | Value | Use |
|---|---|---|
| `--spacing-prose-y` | 1.4em | Paragraph margin in `.prose` |
| `--spacing-section` | 4rem | Major section break |
| `--spacing-section-tight` | 2.5rem | Minor section break |
| `--spacing-reading-pad` | 3.5rem | Post page top padding |
| `--container-reading` | 45rem (720px) | Post body max-width |

---

## 4. Component contracts

| Component | Source | Notes |
|---|---|---|
| `Nameplate` | `app/src/components/nameplate.rs` | Italic-accent two-piece title; whole element is the home link. |
| `PipeNav` | `app/src/components/pipe_nav.rs` | Pipe-separated lowercase nav; current-route highlight. |
| `DateStamp` | `app/src/components/date_stamp.rs` | Newspaper stamp tile (mono kicker + serif day numeral + 2px accent rail). 3 sizes. |
| `PostListRow` | `app/src/components/post_list_row.rs` | DateStamp + title + excerpt + meta. 2 sizes. |
| `TagStrip` | `app/src/components/tag_strip.rs` | Inline middot category strip with reactive selection. |
| `Footer` | `app/src/components/footer.rs` | Sitemap-style 3-column footer + social row + mono uppercase copyright. |
| `Toc` | `app/src/components/toc.rs` | In-flow TOC, conditional render at `TOC_MIN_HEADINGS = 4`. |

Outbound link `↗` glyph is a CSS pseudo-element rule on
`a[href^="http"]:not([href*="alexthola.com"])::after` — no Rust component.

---

## 5. Patterns

### Editorial link (body prose)
```css
color: var(--color-ink);
border-bottom: 1px solid var(--color-accent);
text-decoration: none;
/* :hover { color: var(--color-accent); } */
```

### Date-stamp tile (post list row)
```
┌──────────────┬─────────────────────────────────┐
│ POSTED       │ Italic display title            │
│   29         │ One-line excerpt …              │
│  APR 2026    │ READ TIME · TAG (mono kicker)   │
└──────────────┴─────────────────────────────────┘
  130px gutter   1fr
  ↑ 2px accent rail on the left of the stamp
```

### Sitemap footer
```
─────────────────────────  2px ink rule

  WRITING        REFERENCES    ABOUT
  Latest         Portfolio     Bio
  Archive        Contact       Colophon
  Notes                        RSS / Atom

  GitHub  X  LinkedIn  RSS

  © 2024–2026 ALEX THOLA. POWERED BY RUST + LEPTOS.
```

---

## 6. Routes

| Path | Owner | Notes |
|---|---|---|
| `/` | `app/src/home.rs` | Featured + recent + notes-strip + tag filter |
| `/post/:slug` | `app/src/post.rs` | Pre-meta + italic date + h1 + tag-byline + TOC + ochre prose + post-foot |
| `/post/:slug.md` | `server/src/utils.rs::raw_markdown_handler` | Raw markdown alternate per post |
| `/archive` | `app/src/archive.rs` | Year-grouped chronological archive with `?tag=foo` filter |
| `/notes` | `app/src/notes.rs` | Microblog stream (renamed from `/activity`) |
| `/references` | `app/src/references.rs` | Portfolio rows with mono ▰▱ tech-stack bars |
| `/about` | `app/src/about.rs` | Bio, links, colophon link, JSON-LD Person |
| `/colophon` | `app/src/colophon.rs` | Stack, fonts, source, license |
| `/contact` | `app/src/contact.rs` | Form-only |
| `/random` | `server/src/utils.rs::random_handler` | 302 to a random published post (the "stumble" mechanic) |
| `/feed/feed.xml` | `server/src/utils.rs::atom_handler` | Atom 1.0 |
| `/feed/rss.xml` | `server/src/utils.rs::rss_handler` (alias) | RSS 2.0 |
| `/rss.xml` | `server/src/utils.rs::rss_handler` (legacy) | Backward-compat alias |
| `/activity` | server `Redirect::permanent("/notes")` | 301 redirect |

---

## 7. Quality gates

The redesign holds these invariants:

- `make fmt -- --check` ✓
- `make lint` ✓ (clippy `-D warnings`)
- `make lint-tokens` ✓ no arbitrary color values
- `cargo test -p app --lib` ✓
- `cargo test -p server` ✓
- `cargo build --workspace` ✓

Full `make validate` requires a running SurrealDB for integration tests.

### Visual regression (manual)

T32 in the implementation plan calls for before/after screenshots
captured via `Skill(scry:record-browser)`. Run with `make watch` to bring
up the dev server, then capture each route in light + dark mode.

### Lighthouse audit (manual)

T33 calls for Lighthouse audits on `/` and `/post/:slug` targeting:
- Performance ≥ 90
- Accessibility ≥ 95
- Best Practices ≥ 90
- SEO ≥ 95

Run after `make build-release` against the built site.
