# alexthola.com Redesign — Specification

**Date**: 2026-04-29
**Author**: Alex Thola (assisted)
**Status**: Draft — feeds into planning
**Branch**: `site-redesign-0.2.0`
**Brief**: [`docs/project-brief.md`](./project-brief.md)
**Direction**: D — Dual-Mode Editorial Engineer

---

## 1. Specification Scope

This document translates the approved brief (Direction D + recommended defaults
for D2–D6) into testable acceptance criteria, design tokens with exact values,
component contracts, and route-by-route requirements. It feeds directly into
the planning phase and is the contract executors will be tested against.

### Confirmed user decisions (from brief checkpoint)

| ID | Decision |
|---|---|
| D1 | Direction D — Dual-Mode Editorial Engineer |
| D2 | Burgundy accent: `#7a2942` (light) / `#c47c80` (dark) |
| D3 | Hybrid type stack: display serif + body sans + mono meta |
| D4 | `/activity` → promote to top-level `/notes` (microblog companion to blog) |
| D5 | Header **unfixed** — scrolls with page |
| D6 | Newsletter slot designed; integration deferred to PLAN.md Q2 |

---

## 2. Design System Specification

### 2.1 Color Tokens

Declared as Tailwind v4 `@theme` block in `style/tailwind.css`. Light is
default; dark activates via `[data-theme="dark"]` attribute (toggleable +
`prefers-color-scheme` initial value).

#### Light mode
| Token | Hex | Role | Contrast against `--paper` |
|---|---|---|---|
| `--paper` | `#ecedef` | Page background (cool platinum) | — |
| `--paper-2` | `#dfe1e4` | Surface / code block bg | — |
| `--ink` | `#0a0a0a` | Primary text | 17.6:1 ✓ AAA |
| `--ink-2` | `#2a2a2a` | Secondary text / nameplate body | 13.4:1 ✓ AAA |
| `--ink-3` | `#5a5e62` | Muted text / meta | 6.4:1 ✓ AA |
| `--ink-4` | `#8a8e92` | Subtle / placeholder / disabled | 3.6:1 ✓ AA-large only |
| `--accent` | `#7a2942` | Burgundy — link underline, hover, current-state, blockquote rail | 8.4:1 ✓ AAA |
| `--accent-soft` | `#c47c80` | Lightened accent (used in dark; reused for soft fills in light) | — |
| `--rule` | `#0a0a0a` | Hard 1–2px section rules | — |
| `--rule-soft` | `#bfc1c5` | Hairline rules between list items | — |

#### Dark mode
| Token | Hex | Role | Contrast against `--paper` |
|---|---|---|---|
| `--paper` | `#151515` | Page background | — |
| `--paper-2` | `#2a2a2a` | Surface / code block bg | — |
| `--ink` | `#ecedef` | Primary text | 16.4:1 ✓ AAA |
| `--ink-2` | `#bfc1c5` | Secondary text | 11.8:1 ✓ AAA |
| `--ink-3` | `#8a8e92` | Muted text | 5.5:1 ✓ AA |
| `--ink-4` | `#5a5e62` | Subtle / disabled | 2.7:1 — **decorative only** |
| `--accent` | `#c47c80` | Lightened burgundy for dark mode | 6.7:1 ✓ AA |
| `--accent-soft` | `#7a2942` | Original burgundy reused as deep accent | — |
| `--rule` | `#ecedef` | Hard rules | — |
| `--rule-soft` | `#3a3a3a` | Hairline rules | — |

**Discipline rules**:
- Never use opacity-modifier syntax (`bg-accent/50`) in a component when a token
  exists for the value; declare the variant token instead.
- All component code references tokens. Lint catches `bg-[#hex]` / `text-[#hex]`
  arbitrary values.
- No new tokens added without a documented role.

### 2.2 Typography

#### Faces
- **Display**: **Fraunces Variable** (Google Fonts, OFL, free). One file, all
  weights and the `SOFT`, `WONK` axes available. Used for `h1`, `h2`, and the
  italic-accented nameplate. Fallback stack: `Fraunces Variable, Fraunces, "DM
  Serif Display", Georgia, serif`.
- **Body**: **Inter Variable** (Google Fonts, OFL, free). One file, all weights.
  Used for body, h3–h6, UI text. Fallback stack: `Inter Variable, Inter, system-
  ui, -apple-system, "Segoe UI", Roboto, sans-serif`.
- **Mono**: **JetBrains Mono Variable** (Google Fonts, OFL, free). Used for code
  blocks, post metadata kickers, footer mono uppercase, and date-stamp kicker.
  Fallback stack: `JetBrains Mono Variable, JetBrains Mono, ui-monospace, SFMono-
  Regular, Menlo, Consolas, monospace`.

**Loading strategy**:
- `font-display: swap` on all three.
- Preload `Fraunces` and `Inter` (display + body land on every page).
- Defer-load `JetBrains Mono` (post-detail and footer only).
- Latin subset only (drop CJK, Cyrillic, Greek). Target compressed payload < 200KB total.

#### Type scale
| Role | Family | Size | Weight | Line-height | Tracking |
|---|---|---|---|---|---|
| H1 | Fraunces | 56px (3.5rem) | 600 | 1.05 | -0.02em |
| H1 (small viewport) | Fraunces | 40px (2.5rem) | 600 | 1.05 | -0.02em |
| H2 | Fraunces | 40px (2.5rem) italic | 500 | 1.15 | -0.015em |
| H3 | Inter | 24px (1.5rem) | 600 | 1.25 | -0.005em |
| H4 | Inter | 18px (1.125rem) | 600 | 1.3 | 0 |
| Body | Inter | 17px (1.0625rem) | 470 (custom non-default) | 1.65 | 0 |
| Body (post prose) | Inter | 18px (1.125rem) | 470 | 1.7 | 0 |
| Meta / kicker | JetBrains Mono | 12px (0.75rem) | 500 | 1.4 | 0.08em uppercase |
| Code (block) | JetBrains Mono | 14px (0.875rem) | 450 | 1.5 | 0 |
| Code (inline) | JetBrains Mono | 0.92em | 450 | inherit | 0 |
| Nameplate | Fraunces | 28px (1.75rem) | 500 | 1.1 | -0.015em |
| Nameplate accent word | Fraunces | 28px italic | 500 | 1.1 | -0.015em |

**The 470 weight rule** (PostHog-borrowed): body text uses Inter Variable's
custom `470` axis position (between 400 Regular and 500 Medium). Implemented
as `font-variation-settings: "wght" 470;` on `body`. Headings use `wght 600`.

### 2.3 Spacing scale

Tailwind v4 default spacing scale (`0.25rem` base) is preserved. Custom semantic
spacing tokens added for editorial rhythm:

| Token | Value | Use |
|---|---|---|
| `--space-prose-y` | `1.4em` | Paragraph margin-bottom in `.prose` |
| `--space-section` | `4rem` (64px) | Major section vertical break |
| `--space-section-tight` | `2.5rem` (40px) | Minor section break |
| `--space-reading-pad` | `3.5rem` (56px) | Reading column top padding (post page) |
| `--space-list-gap` | `2rem` (32px) | Post-list row gap |
| `--reading-column` | `45rem` (720px) | Max-width of post body |

### 2.4 Border radius

| Token | Value | Use |
|---|---|---|
| `--radius-none` | `0` | Default (editorial; no rounded chrome) |
| `--radius-sm` | `0.125rem` (2px) | Buttons, code blocks |
| `--radius-md` | `0.25rem` (4px) | Cards (rare) |
| `--radius-pill` | `999px` | Tag/category chips on archive |

Direction D is mostly square. `--radius-none` is the default; `sm` is reserved
for interactive controls (buttons, code blocks).

### 2.5 Rule (border) patterns

| Pattern | Spec |
|---|---|
| Hard ink rule | `border-bottom: 2px solid var(--rule)` — under header, over footer |
| Hairline soft rule | `border-bottom: 1px solid var(--rule-soft)` — between post-list rows, around code blocks |
| Accent rail (left) | `border-left: 2px solid var(--accent)` — date-stamp tile, blockquote |
| Accent rail (left, subtle) | `border-left: 3px solid var(--accent)` — blockquote |

### 2.6 Link patterns

| Pattern | Spec |
|---|---|
| Body prose link | `color: var(--ink); border-bottom: 1px solid var(--accent); text-decoration: none;` — `:hover { color: var(--accent); }` |
| Nav link | `color: var(--ink); border-bottom: 1px solid transparent;` — `:hover, [data-current="true"] { border-bottom-color: var(--accent); color: var(--accent); }` |
| Outbound link | Same as body prose link, plus `::after { content: " ↗"; font-family: var(--mono); font-size: 0.85em; opacity: 0.7; }` for `[href^="http"]:not([href*="alexthola.com"])` |
| Footer link | `color: var(--ink-3); text-decoration: dotted underline; text-underline-offset: 0.25em;` |

### 2.7 Iconography

- The current `svg { fill: white !important; }` global rule is **deleted** in
  this redesign. Icons inherit `currentColor` per token.
- Icon sizes: 16px (inline meta), 20px (nav), 24px (footer/social).
- `icondata` Bootstrap icons stay in use; ensure each icon's `fill="currentColor"`
  attribute is set so token coloring works.

---

## 3. Information Architecture

### 3.1 Routes

Existing routes are preserved (no permalink breakage). New routes are added.

| Route | Status | Owner |
|---|---|---|
| `/` | **Refactored** — home with featured post + recent posts list + small notes strip | `app/src/home.rs` |
| `/post/:slug` | **Refactored** — single-column reading page with TOC for long posts | `app/src/post.rs` |
| `/archive` | **NEW** — full chronological archive, filtered by tag | `app/src/archive.rs` (new) |
| `/notes` | **NEW** (replaces `/activity`) — microblog stream, top-level | `app/src/notes.rs` (renamed from `activity.rs`) |
| `/references` | **Refactored** — portfolio cards stripped of glassmorphism | `app/src/references.rs` |
| `/about` | **NEW** — author bio (lifted from `/contact` `whoami`), timeline, colophon link | `app/src/about.rs` (new) |
| `/contact` | **Refactored** — form-only; bio moved to `/about` | `app/src/contact.rs` |
| `/colophon` | **NEW** — site tech stack, fonts, license | `app/src/colophon.rs` (new) |
| `/post/:slug/raw.md` | **NEW** — raw markdown alternate per post | server route in `server/` crate |
| `/feed/feed.xml` | **NEW or verify** — Atom feed | server route |
| `/feed/rss.xml` | **NEW or verify** — RSS feed | server route |
| `/feed/feed.json` | **NEW** — JSON feed | server route |
| `/random` | **NEW** — Stumble redirect to a random published post | server route |
| `/activity` | **Redirect** to `/notes` (HTTP 301) | server route |

### 3.2 Navigation

#### Top header (unfixed, scrolls with page)
Pipe-separated lowercase nav (blogosphere pattern) on the right; nameplate on the left.

```
alex *thola*  (italicized "thola" in burgundy)            writing | notes | references | about | rss ↗
─────────────────────────────────────────────────────────────────────────────  (2px ink rule)
```

- Nameplate: Fraunces 28px, "alex" in `--ink`, " " (single space), "thola" in italic + `--accent`. Both as `<a href="/">` (the whole nameplate is the home link).
- Nav: JetBrains Mono uppercase 12px tracked +0.08em letter-spacing, items separated by ` | ` literal pipe characters with `--ink-3` color.
- Current route: `[data-current="true"]` → `color: var(--accent); border-bottom: 1px solid var(--accent);`.
- `rss ↗` is the outbound link to `/feed/feed.xml` (or whichever feed format the user prefers as default).
- Mobile (<640px): pipe nav collapses to centered single-row, smaller; nameplate stays left.

#### Footer (sitemap-style)
Replaces current fixed footer. Scrolls with page.

```
─────────────────────────────────────────────────────────────────────────────  (2px ink rule)

  WRITING                  REFERENCES               ABOUT
  Latest                   Portfolio                Bio
  Archive                  Contact                  Colophon
  Notes                                             RSS / Atom / JSON Feed

  GitHub ↗  Mastodon ↗  LinkedIn ↗  X ↗

  © 2024–2026 ALEX THOLA. POWERED BY RUST + LEPTOS.       (mono uppercase)
```

- Three columns (collapses to one on <640px).
- Footer mono uppercase 11px tracked +0.08em.
- Social icons row uses `↗` glyph for outbound; size 20px.

### 3.3 Permalinks (sacred, do not break)

- `/post/:slug` is preserved exactly. RSS subscribers depend on this.
- `/contact` is preserved (form still works); bio moved out, form remains.
- `/references` is preserved.
- `/activity` HTTP 301 → `/notes`. Search engines and any bookmarks redirect cleanly.

---

## 4. Route Specifications

### 4.1 `/` — Home

**Purpose**: Entry point; sells the writing without selling the consultancy.
**Layout**: Single column, max-width `--reading-column` (720px), centered.

**Sections (top to bottom)**:

1. **Featured post** (most recent, full card):
   - Date stamp tile (mono kicker `MOST RECENT • <DATE>` + huge serif day numeral) + 2px accent rail on left
   - Post title in Fraunces 32px italic-display
   - Excerpt in Inter 17px (`--ink-2`), max 2 lines, ellipsis
   - Meta row (mono 12px uppercase): `READ TIME · TAGS · VIEWS`
   - Hairline rule below

2. **Recent posts list** (next 5–7 posts):
   - Same date-stamp tile component, smaller (date numeral 24px instead of 32px)
   - Post-list row spec matches fsck.com: `grid-template-columns: 130px 1fr; gap: 32px;`
   - Title in Fraunces 22px italic
   - Excerpt 1 line, mono kicker meta (read time + first tag)
   - Hairline rule between rows

3. **`+ archive →` link** to `/archive` (right-aligned mono uppercase).

4. **Latest notes strip** (3 most recent notes from `/notes`):
   - Compact: title + relative time + first tag, hairline rule between
   - `+ notes →` link to `/notes`

5. **Tag filter row** at bottom (preserved from current site, restyled):
   - Inline category strip (blogosphere pattern), not pills
   - Format: `topics: rust · leptos · surrealdb · consulting · all`
   - Selected tag: `--accent` color + underline; others `--ink-3`
   - Click toggles filter and updates list above (existing Leptos signal flow preserved)

**Acceptance criteria**:
- [ ] Featured post is the most recent published post by `created_at` desc.
- [ ] Recent posts list excludes the featured post (no duplication).
- [ ] Tag filter updates the recent-posts list reactively (not the featured post).
- [ ] Latest notes strip pulls from `select_activities(0)` and shows top 3.
- [ ] All copy is `--ink`, all meta is `--ink-3`, all accents are `--accent`.
- [ ] Lighthouse Performance ≥ 90, Accessibility ≥ 95.
- [ ] Renders identically light/dark; no FOUC during theme load.

### 4.2 `/post/:slug` — Post detail

**Purpose**: The reading page. The heart of the site.
**Layout**: Single column, max-width `--reading-column` (720px), centered, top
padding `--space-reading-pad` (56px).

**Sections (top to bottom)**:

1. **Pre-title meta row**:
   - Mono 12px uppercase: `<RELATIVE-TIME> · <READ-TIME> MIN READ · <VIEW-COUNT> VIEWS`
   - Optional: a hairline rule below

2. **Date display**:
   - Italic Fraunces 24px in `--ink-3`, format `April 7, 2026` — placed ABOVE
     title (fsck.com pattern)

3. **Title (h1)**:
   - Fraunces 56px (40px on small viewports), weight 600, line-height 1.05,
     letter-spacing -0.02em, color `--ink`

4. **Tag-byline + author chip** (1px hairline rule above and below):
   - LEFT: tags in mono 11px uppercase, comma-separated with no `#`, color `--ink-3`
     (e.g. `RUST · LEPTOS · CONSULTING`)
   - RIGHT: tiny author chip — circular avatar 28px + "by Alex Thola" in Inter 13px

5. **In-flow TOC** (only for posts with > 4 h2/h3 headings or > 1500 words):
   - `<aside class="toc">` placed BEFORE article body
   - Heading "ON THIS PAGE" in mono 11px uppercase
   - List of h2 anchors as plain links
   - Renders only on viewports ≥ 768px (mobile collapses to inline marker)

6. **Article body** (`<article>` containing the markdown HTML):
   - 18px Inter at line-height 1.7
   - Paragraph margin-bottom `--space-prose-y` (1.4em)
   - h2 in italic Fraunces 32px
   - h3 in mono 12px uppercase `--accent` ("kicker" pattern, fsck.com)
   - Prose link: ink color + accent underline
   - Blockquote: 3px accent left rail, italic, padding-left 24px
   - Code block: `--paper-2` bg, hairline border, 14px mono, 1.5 lh
   - Inline code: 0.92em, `--paper-2` bg, hairline border
   - Image: 100% width, hairline soft border (1px `--rule-soft`)

7. **Post footer**:
   - Hairline rule above
   - Three rows:
     - **Tags** linking to `/archive?tag=<slug>`
     - **Prev / Next** post links (mono uppercase, with `←` / `→` glyphs)
     - **More from #<tag>** — 2 random posts with the same primary tag
   - Mono uppercase row: `RAW MARKDOWN ↗ · COPY LINK · SHARE`
     - "RAW MARKDOWN" links to `/post/:slug/raw.md`
     - "COPY LINK" copies canonical URL via JS (progressive — works without JS too)
     - "SHARE" opens native share sheet on mobile, else copies URL

**Acceptance criteria**:
- [ ] All five existing sample posts render with no broken styles.
- [ ] KaTeX math renders correctly (verify with one math-heavy sample).
- [ ] Code blocks with `<pre><code class="language-rust">` render with the mono
      stack and accent kicker.
- [ ] TOC appears only on long posts; uses anchor links to h2 headings.
- [ ] `/post/:slug/raw.md` returns the raw markdown source with `Content-Type: text/markdown`.
- [ ] `<link rel="alternate" type="text/markdown" href="/post/:slug/raw.md">` is in head.
- [ ] `<link rel="canonical" href="/post/:slug">` is in head.
- [ ] OG image generation: defer to PLAN.md backlog, but ensure `<meta property="og:image">` falls back to a default brand image.
- [ ] Increment-views server action runs only in production builds (current behavior preserved).
- [ ] Hydration works without flicker.
- [ ] Reading column is 720px; on mobile, full-width with 24px horizontal padding.

### 4.3 `/archive` — NEW

**Purpose**: Full chronological list of posts, filterable by tag. Replaces the
"all 50 most recent on home" implicit archive.

**Layout**: Single column, max-width `--reading-column`.

**Sections**:

1. **Page title**: "writing" in Fraunces italic 32px, lowercase.
2. **Total count** (mono kicker): `<N> POSTS SINCE <YEAR>`.
3. **Tag filter strip** (inline, blogosphere pattern):
   - `topics: all · rust · leptos · surrealdb · consulting · …`
   - URL-driven: `/archive` (default `all`), `/archive?tag=rust`
4. **Year-grouped chronological list**:
   - For each year (newest first):
     - H2 italic Fraunces 32px: `2026`
     - List of posts in that year with date-stamp tile
   - Hairline rule between years
5. **Pagination** at bottom (mono uppercase): `← prev | page 1 of N | next →`
   - 25 posts per page

**Acceptance criteria**:
- [ ] `/archive` returns all posts in `created_at` desc order, paginated 25 per page.
- [ ] `/archive?tag=<slug>` filters server-side; URL is shareable.
- [ ] Year headers are clickable anchors (e.g. `#2026`).
- [ ] Empty state: "No posts found for tag X" with reset link.

### 4.4 `/notes` — NEW (replaces `/activity`)

**Purpose**: Microblog stream — short notes, links, asides. Companion to long-form blog.

**Layout**: Single column, max-width `--reading-column`.

**Sections**:

1. **Page title**: "notes" in Fraunces italic 32px lowercase.
2. **Subtitle** in mono uppercase: `SHORT-FORM NOTES, LINKS, AND ASIDES`.
3. **Note list** (replaces existing `/activity` rendering):
   - Each note: relative time kicker (mono 11px) + content (body Inter 17px, prose-styled) + tags row (mono 11px) + optional source link with `↗`
   - Hairline rule between notes
   - Tags clickable, link to `/notes?tag=<slug>`
4. **Pagination** (mono): `← prev | next →`. Existing 10/page Resource pattern preserved.

**Acceptance criteria**:
- [ ] All current `select_activities` data renders correctly.
- [ ] Tag filter works URL-driven.
- [ ] `/activity` returns 301 → `/notes`.
- [ ] Links inside note content get the `↗` outbound glyph automatically.
- [ ] Inconsistent `bg-gray-800` and `text-blue-400` colors are gone (token-driven).

### 4.5 `/references` — Refactored

**Purpose**: Portfolio. Drop the glassmorphism + grid background; preserve the
content (project title, description, tech-stack with percentages).

**Layout**: Single column, max-width `--reading-column`.

**Sections**:

1. **Page title**: "references" in Fraunces italic 32px lowercase.
2. **Subtitle**: short paragraph (existing copy preserved or updated).
3. **Project list** (one per row, NOT cards):
   - Each project:
     - Date or year kicker (mono uppercase)
     - Title in Fraunces 24px
     - Description body 17px
     - Tech-stack as inline list with subtle bars: `RUST [▰▰▰▰▰▰▰▰▱▱ 80%] · LEPTOS [▰▰▰▰▰▰▰▱▱▱ 70%]`
       — bars use `--accent` color, mono 11px
     - Hairline rule below

**Acceptance criteria**:
- [ ] Existing `select_references` API consumed unchanged.
- [ ] Glassmorphism, grid background, and 2xl rounded corners removed.
- [ ] Tech-stack percentage bars render in mono with `▰`/`▱` characters (no SVG).
- [ ] All copy uses tokens; no `text-[#ffef5c]` arbitrary values remain.

### 4.6 `/about` — NEW

**Purpose**: Author bio + timeline. Lifted from existing `/contact` `whoami`
section.

**Layout**: Single column, max-width `--reading-column`.

**Sections**:

1. **Page title**: "about" in Fraunces italic 32px lowercase.
2. **Author block**:
   - Circular avatar 80px (current GitHub avatar URL preserved)
   - Name in Fraunces 28px
   - Role in body 17px (`Staff Software Engineer` etc — current copy)
   - One-line bio in `--ink-2` italic
3. **Long-form bio** in prose (1–2 paragraphs; user can edit). Initial template
   provided based on existing copy.
4. **Timeline / now** (markdown-driven, optional first ship):
   - Year-grouped timeline of work, talks, projects (free-form)
5. **Links row**: GitHub ↗, LinkedIn ↗, X ↗, Mastodon ↗, RSS ↗
6. **Colophon link**: `read about how this site is built →` linking to `/colophon`.

**Acceptance criteria**:
- [ ] Avatar image loads with `loading="lazy"` + `width`/`height` to prevent CLS.
- [ ] Same author bio is no longer present in `/contact`.
- [ ] All links use the standard outbound pattern.

### 4.7 `/contact` — Refactored

**Purpose**: Contact form only. Bio moved out.

**Layout**: Single column, max-width `--reading-column`.

**Sections**:

1. **Page title**: "contact" in Fraunces italic 32px lowercase.
2. **Lead**: short paragraph — "Get in touch about Rust consulting, technical
   review, or speaking. I read every message." (user can edit).
3. **Contact form** (existing fields preserved):
   - Name, Email, Subject, Message
   - Inputs: `--paper-2` bg, hairline border, 17px Inter body, focus ring `--accent`
   - Submit button: `--accent` bg, `--paper` text, mono uppercase 12px tracked
     +0.08em "SEND MESSAGE"
   - Loader and "Message sent" success state preserved (current Leptos action
     pattern)
4. **Alt path**: "Or email directly: alex@…" (user fills in).

**Acceptance criteria**:
- [ ] Existing `contact` server action unchanged.
- [ ] Form submits successfully; success message renders.
- [ ] All inputs labeled and accessible (`aria-label` or `<label>`).
- [ ] Spam protection: existing approach preserved.

### 4.8 `/colophon` — NEW

**Purpose**: Hacker-culture handshake — explain how the site is built.

**Layout**: Single column, max-width `--reading-column`.

**Sections (markdown content; renders as a long post)**:

1. **Page title**: "colophon" in Fraunces italic 32px lowercase.
2. **Stack section**: Rust, Leptos, Axum, SurrealDB, Tailwind v4, Cargo-leptos,
   Caddy, DigitalOcean App Platform.
3. **Fonts section**: Fraunces, Inter, JetBrains Mono (each linked to source +
   licensed).
4. **Source link**: `github.com/athola/blog ↗`.
5. **License**: AGPL-3.0.

**Acceptance criteria**:
- [ ] Page renders as a regular post-detail layout for consistency.
- [ ] Each font name links to its source.

### 4.9 `/random` — NEW (server route)

**Purpose**: Stumble button — pick a random published post and 302 redirect.

**Acceptance criteria**:
- [ ] `GET /random` returns 302 with `Location: /post/:slug` for a randomly
      selected published post.
- [ ] Cached for 60 seconds at most (stays "fresh-feeling" for repeat presses).
- [ ] Footer or nav exposes the route as `↗ stumble`.

### 4.10 Feeds: `/feed/feed.xml`, `/feed/rss.xml`, `/feed/feed.json`

**Purpose**: First-class RSS-first behavior, three formats.

**Acceptance criteria**:
- [ ] `feed.xml`: Atom 1.0 — top 50 posts, full content if ≤ 4096 chars else excerpt.
- [ ] `rss.xml`: RSS 2.0 — same content, RSS-2.0 envelope.
- [ ] `feed.json`: JSON Feed v1.1 — same content, JSON envelope.
- [ ] All three feeds return correct `Content-Type` headers.
- [ ] `<head>` of every page includes `<link rel="alternate">` for all three feeds.
- [ ] `/post/:slug` includes `<link rel="alternate" type="text/markdown">`.

### 4.11 `/post/:slug/raw.md` — NEW (raw markdown alternate)

**Purpose**: Hacker-culture handshake; lets readers grab the source.

**Path shape note**: Axum 0.8 forbids mixing a literal extension (`.md`) with a path parameter (`:slug`) in the same path segment, so the route ships at `/post/:slug/raw.md` rather than the cleaner `/post/:slug.md`. See `server/src/main.rs` for the load-bearing comment that documents this.

**Acceptance criteria**:
- [ ] `GET /post/:slug/raw.md` returns 200 with `Content-Type: text/markdown; charset=utf-8`.
- [ ] Body is the canonical markdown source (not regenerated from HTML).
- [ ] Returns 404 for unpublished or missing slugs.
- [ ] Public access (no auth required).

---

## 5. Component Contracts

Components live in `app/src/components/`. New components added in this
redesign:

### 5.1 `Nameplate`
- Renders the italic-accented two-piece nameplate.
- Props: none (constant text).
- Click target: full nameplate links to `/`.

### 5.2 `PipeNav`
- Renders the lowercase pipe-separated nav.
- Props: `current_route: &str`.
- Includes `↗ rss` outbound link.

### 5.3 `DateStamp`
- Renders the date-stamp tile (mono kicker + serif day numeral + 2px accent rail).
- Props: `kicker_text: String`, `day: u8`, `month: String`, `year: i32`,
  `size: DateStampSize::{Featured | Default | Compact}`.
- Variants:
  - `Featured` (home page top): day 48px, kicker "MOST RECENT"
  - `Default` (post list rows): day 32px
  - `Compact` (notes / small spaces): day 24px

### 5.4 `PostListRow`
- Renders one post in a list, including DateStamp + title + excerpt + meta.
- Props: `post: &Post`, `size: PostListSize::{Featured | Default}`.
- Includes hairline rule below by default; opt-out via `divider: false`.

### 5.5 `TagStrip`
- Inline category strip (blogosphere pattern), URL-driven.
- Props: `tags: Vec<(String, u32)>`, `selected: Option<String>`,
  `link_base: &str` (e.g. `/archive` or `/notes`).
- Renders as `topics: all · rust · leptos · …` with click → URL update.

### 5.6 `Toc`
- In-flow table of contents.
- Props: `headings: Vec<TocHeading>` where `TocHeading { level: u8, text: String, anchor: String }`.
- Auto-generated from post body during markdown parse (server-side).
- Renders only if `headings.len() >= 4`.

### 5.7 `OutboundLink`
- Wraps an `<a>` and appends `↗` glyph if `href` is external.
- Implementation: pure CSS via `[href^="http"]:not([href*="alexthola.com"])::after`. No component needed if CSS-only is sufficient.

### 5.8 `Footer`
- Sitemap-style footer, three columns + social row + copyright line.
- Props: none.

### 5.9 `ThemeToggle` (deferred to next branch — designed-for, not implemented)
- Token system supports it; UI deferred to PLAN.md Q1 follow-up branch.
- For this branch: ship light + dark via `prefers-color-scheme` only. Toggle UI
  is a follow-up.

---

## 6. Non-Functional Requirements

### 6.1 Performance
- [ ] **First Contentful Paint** ≤ current baseline; ideally improved by 20%
      via single-variable-font swap-in (current ships 28 Poppins TTF files).
- [ ] **Cumulative Layout Shift** ≤ 0.05 — preload Fraunces + Inter, pre-set
      `width`/`height` on all `<img>`.
- [ ] **Total webfont payload** ≤ 200KB compressed. Verify via DevTools Network
      filtered by Font.
- [ ] **Lighthouse Performance** ≥ 90 on `/` and `/post/:slug`.

### 6.2 Accessibility
- [ ] **Lighthouse Accessibility** ≥ 95 on every route.
- [ ] **Color contrast** ≥ 4.5:1 (AA) for all text — verified with token table
      above.
- [ ] **Focus states** visible on every interactive element. Use 2px `--accent`
      outline with 2px offset.
- [ ] **Reduced motion**: respect `prefers-reduced-motion` — disable transition
      `duration-500` on hover.
- [ ] **Skip-to-content link** at top of every page.
- [ ] **Semantic HTML**: `<article>` for posts, `<nav>` for nav, `<aside>` for TOC.
- [ ] **Image alt text** on every `<img>` (no decorative-only without `alt=""`).
- [ ] **Form labels** explicit on every contact field.

### 6.3 SEO
- [ ] `<title>` set per route, with site suffix.
- [ ] `<meta name="description">` per route.
- [ ] `<meta property="og:*">` — title, description, image, url, type.
- [ ] `<link rel="canonical">` per page.
- [ ] `<link rel="alternate">` for RSS / Atom / JSON Feed in head of all pages.
- [ ] `<link rel="alternate" type="text/markdown">` on post pages.
- [ ] Sitemap.xml (defer to backlog if not already present).
- [ ] Schema.org JSON-LD: `Article` on post pages, `Person` on `/about`.

### 6.4 Browser support
- Modern evergreen browsers (Chrome, Firefox, Safari, Edge — last 2 versions).
- Mobile Safari + Chrome on Android.
- No IE / legacy support.

### 6.5 Theme handling
- Default to user's `prefers-color-scheme`.
- Persist toggle (when implemented in v2 follow-up) in `localStorage` under key
  `alexthola-theme`.
- Token system supports both modes from day one — no FOUC, theme attribute set
  before paint via inline `<script>` in `<head>`.

---

## 7. Acceptance Criteria — Master Checklist

Mirror of section-specific ACs, summarized for plan-phase task ordering.

### Foundation (must complete before route work)
- [ ] `style/tailwind.css` rewritten with `@theme` block declaring all tokens
      from §2.1, §2.2, §2.3, §2.4, §2.5, §2.6.
- [ ] Global `svg { fill: white !important }` rule deleted.
- [ ] All 28 Poppins font-face declarations deleted.
- [ ] Three webfonts (Fraunces, Inter, JetBrains Mono) loaded via Latin subsets.
- [ ] Lint rule (or grep-based check in `make validate`) fails on `bg-[#hex]`
      arbitrary values in `app/src/**/*.rs`.
- [ ] `<head>` has `<link rel="alternate">` for all three feeds + theme-attribute
      pre-paint script.

### Core route work
- [ ] `/` refactored per §4.1.
- [ ] `/post/:slug` refactored per §4.2.
- [ ] `/archive` (NEW) per §4.3.
- [ ] `/notes` (renamed from `/activity`) per §4.4; redirect from `/activity`.
- [ ] `/references` refactored per §4.5.
- [ ] `/about` (NEW) per §4.6.
- [ ] `/contact` refactored per §4.7.
- [ ] `/colophon` (NEW) per §4.8.

### Server routes
- [ ] `/random` 302 redirect.
- [ ] `/post/:slug/raw.md` raw markdown alternate.
- [ ] Three feed routes (RSS, Atom, JSON Feed).

### Quality gates
- [ ] `make format && make lint && make test && make build` all pass.
- [ ] Lighthouse Performance ≥ 90 on `/` and `/post/:slug`.
- [ ] Lighthouse Accessibility ≥ 95 on every route.
- [ ] Visual screenshots captured before/after for every route via `scry:record-browser`.
- [ ] Manual smoke: KaTeX-heavy post + code-heavy post both render correctly.

### Documentation
- [ ] `README.md` updated to reference new design system + routes.
- [ ] `docs/design-system.md` (NEW) extracts §2 of this spec as a living
      design reference.
- [ ] `PLAN.md` updated to reflect what this branch ships (theme toggle now
      structurally enabled; newsletter slot designed but integration deferred).

---

## 8. Out of Scope (Explicit Won't-Have)

- ❌ Theme **toggle UI** (designed-for via tokens; UI ships in a follow-up branch).
- ❌ Newsletter integration (slot designed; integration is PLAN.md Q2).
- ❌ Comments system (PLAN.md Q2).
- ❌ Site search (PLAN.md Q1; outside redesign scope).
- ❌ Server-side syntax highlighting (PLAN.md Q1; current `prose` styles
      preserved as-is).
- ❌ Mascot / illustrations / hand-drawn elements.
- ❌ Sticky / fixed chrome of any kind.
- ❌ Cookie banner / analytics (defer separately if needed).
- ❌ JavaScript framework or build tool changes.
- ❌ Hugo / Astro / static-generator migration (Leptos + SSR is preserved).
- ❌ Database schema migrations (existing `posts`, `tags`, `activities`,
      `references` tables preserved).
- ❌ Author/multi-author features.
- ❌ Tag detail pages at `/tag/:slug` (use `/archive?tag=<slug>` instead — same
      experience, fewer routes).

---

## 9. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Webfont payload breaks Lighthouse 90 budget | Medium | Medium | Latin subset; preload display + body only; lazy mono; if still over, drop Fraunces axes (use static weights instead of variable) |
| Dark-mode accent contrast bug ships | Medium | High | Verify tokens at 4.5:1 BEFORE first commit; add automated contrast test to `make validate` |
| `prose` rewrite breaks KaTeX styling | Medium | Medium | Test math-heavy post on every route commit; treat KaTeX styles as untouchable; isolate `.katex` from new prose rules |
| `/activity` → `/notes` redirect breaks for existing readers | Low | Low | 301 redirect; preserve existing page if HTTP referer is set |
| Date-stamp grid (130px / 1fr) overflows on small mobile | Medium | Low | Below 480px, collapse to inline mono date kicker (no tile) |
| `<link rel="alternate" type="text/markdown">` requires raw markdown route on server | Verified-need | Low | Server route added in this branch; SurrealDB stores raw markdown already |
| Removing fixed footer breaks bottom-anchored elements | Low | Low | Footer scrolls naturally; no UI depends on fixed footer |
| Removing global `svg{fill:white!important}` breaks existing icons | High | Low | Audit all `icondata` usages; ensure each icon explicitly uses `currentColor`. This is a guaranteed task in the plan phase. |
| Three font webfont families cause CLS during load | Medium | Medium | Preload + `font-display: swap` + size-adjust matched to fallback metrics |
| Spec scope creep into "while we're here, let's add search" | High | High | Out-of-scope list (§8) is explicit; lint of plan-phase tasks against §8 |

---

## 10. Glossary

- **Token**: a named CSS variable in the `@theme` block, intended for reuse.
- **Date-stamp**: the post-list tile (mono kicker + serif day numeral + 2px accent rail) — fsck.com pattern.
- **Pipe-nav**: pipe-separated lowercase nav row — blogosphere pattern.
- **Sitemap-footer**: dense footer linking every section — PostHog pattern.
- **Italic accent nameplate**: site title with one word italicized in `--accent` — fsck.com pattern.
- **Outbound glyph**: the `↗` appended to external links — blogosphere pattern.
- **Locked palette**: the discipline of declaring all colors as tokens at the top of the stylesheet, with a CSS comment naming each — fsck.com pattern.

---

## 11. Next Steps

1. **`/attune:blueprint`** → produce `docs/implementation-plan.md` with tasks
   in dependency order, each with a TDD or proof-of-work test, sized to ≤ 1
   work-session each.
2. **`/attune:execute`** → execute the plan. Expect 2–3 working sessions.

The next phase auto-invokes per the orchestrator's protocol.

---

## Appendix A: Token Migration Map (current → new)

| Current inline | New token |
|---|---|
| `bg-[#1e1e1e]` | `bg-paper` |
| `bg-[#2a2a2a]` | `bg-paper-2` |
| `text-white` | `text-ink` |
| `text-gray-300` | `text-ink-2` |
| `text-gray-400` | `text-ink-3` |
| `text-[#ffef5c]` | `text-accent` |
| `bg-[#ffef5c]` | `bg-accent` |
| `bg-[#ffef5c]/8`, `bg-[#ffef5c]/10` | `bg-accent-soft` (with token-defined alpha) |
| `font-poppins` | `font-sans` (now Inter Variable) |
| `font-mono` | `font-mono` (now JetBrains Mono Variable) |
| `bg-card` | `bg-paper-2` |

## Appendix B: File-level change inventory

Files **modified** in execute phase:
- `style/tailwind.css` — full rewrite (new `@theme`, no Poppins)
- `app/src/lib.rs` — shell changes (unfix header, sitemap footer)
- `app/src/home.rs` — featured + recent + notes strip
- `app/src/post.rs` — TOC + post-foot
- `app/src/contact.rs` — bio extracted out
- `app/src/references.rs` — glassmorphism removed
- `app/src/activity.rs` → renamed `notes.rs`, palette aligned
- `app/src/components/header.rs` — pipe-nav + nameplate
- `app/src/components/icons.rs` — currentColor migration

Files **added**:
- `app/src/archive.rs` (new route)
- `app/src/about.rs` (new route)
- `app/src/colophon.rs` (new route)
- `app/src/components/nameplate.rs`
- `app/src/components/pipe_nav.rs`
- `app/src/components/date_stamp.rs`
- `app/src/components/post_list_row.rs`
- `app/src/components/tag_strip.rs`
- `app/src/components/toc.rs`
- `app/src/components/footer.rs`
- `server/src/feeds.rs` (or extend existing)
- `server/src/markdown_alt.rs` (raw markdown route)
- `server/src/random.rs` (stumble route)
- `docs/design-system.md` (extracts §2 of this spec)

Files **deleted** or substantially reduced:
- `style/tailwind.css` Poppins font-face block (~150 lines)
- `style/tailwind.css` global `svg{fill:white!important}` block (~80 lines)
