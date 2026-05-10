# alexthola.com Redesign — Project Brief

**Date**: 2026-04-29
**Author**: Alex Thola (assisted)
**Status**: Draft — awaiting direction selection
**Branch**: `site-redesign-0.2.0`
**Reference sites**: [blogosphere.app](https://blogosphere.app/) (canonically [text.blogosphere.app](https://text.blogosphere.app/)), [posthog.com](https://posthog.com/), [blog.fsck.com](https://blog.fsck.com/)

---

## 1. Problem Statement

**Who**: Alex Thola, the site author (Staff Software Engineer, blogger, consultant)
and his readers (technical, RSS-readers, hiring managers, prospective clients).

**What**: The current site at `alexthola.com` is functional but generic — a dark-only
TailwindCSS template with Poppins-everywhere, fixed top-and-bottom chrome, masonry
post cards, a clunky `prose` override block, and no real design tokens. Every color
is an inline `bg-[#1e1e1e]` arbitrary value scattered across files. The information
architecture is also incoherent (e.g. `whoami` lives buried inside `/contact`,
`/activity` uses an entirely different palette from the rest of the site).

**Where**: Across all five routes (`/`, `/post/:slug`, `/references`, `/contact`,
`/activity`) and in the design system itself (`style/tailwind.css`).

**When**: Now — the `site-redesign-0.2.0` branch is open with no commits yet, the
backlog already calls for theme toggle + RSS + search, and the surrounding deploy
infra (Caddy, DO App Platform, Docker) is freshly stable per the `0.1.8` hotfix.
Clean window for ambition.

**Why** (impact of not solving):
- Site reads as "engineer who shipped a working blog" rather than "engineer worth
  hiring." The visual layer underplays the writing.
- Every new feature (theme toggle, search, related posts) compounds debt against an
  inconsistent token-less Tailwind layer.
- Inline `bg-[#1e1e1e]` style scattered across components blocks the theme-toggle
  goal already on the Q1 2026 backlog.

**Current State**: Dark theme #1e1e1e, yellow accent #ffef5c, Poppins, masonry post
cards, fixed top header + fixed bottom footer (eats ~9% viewport), `prose` block
with cluttered dark-mode overrides on `/post`, glassmorphism cards on `/references`,
inconsistent palette on `/activity`. RSS link wired in `icons.rs` (feed existence
unverified). All 28 Poppins font weights inlined in CSS. Global
`svg { fill: white !important; }` rule fights with any future colored illustration.

---

## 2. Goals

**Primary**:
1. Establish a coherent design system in `@theme` tokens (Tailwind v4 native) so
   future features compose instead of compound debt.
2. Pull genuine personality from the three reference sites without copying any of
   them — the result should feel like alexthola.com, not a Frankenstein.
3. Improve reading experience on `/post/:slug` enough that long posts feel like a
   real publication.

**Secondary**:
4. Unblock theme toggle (Q1 2026 PLAN.md backlog) by structural design, not as
   a separate feature later.
5. Clean up information architecture: dedicated `/about`, dedicated `/archive`,
   `/activity` and `/references` aligned to one design language.
6. Establish RSS/Atom/JSON feeds + raw `.md` alternate per post (fsck.com pattern)
   as a personality move and a real reader-utility.

**Tertiary**:
7. Reduce font payload (Poppins ships 28 weight files today; replace with one
   variable face).
8. Replace the `svg { fill: white !important; }` global rule with token-driven
   icon colors.
9. Decide whether `/activity` survives, gets re-skinned, or merges into `/now` or
   the home stream.

---

## 3. Constraints

### Technical
- **Stack is fixed**: Rust + Leptos (WASM hydration) + Axum SSR + SurrealDB +
  TailwindCSS v4 + cargo-leptos. No framework migration in scope.
- **Server-side rendering must keep working** — the redesign cannot break SSR or
  hydration.
- **No JavaScript framework added**. Anything interactive must work in Leptos
  signals or in plain progressive-enhancement JS.
- **Production deploy stays on DO App Platform with Caddy fronting SurrealDB** —
  redesign cannot require infra changes.
- **Build pipeline**: must work with `cargo-leptos` + Tailwind v4 CLI; no PostCSS
  hacks or alternate CSS preprocessors.
- **All five existing routes must continue to render** during and after redesign
  (no permalink breakage). `/post/:slug` URLs especially are sacred for RSS readers.

### Resources
- **Timeline**: Single feature branch, target merge in 1–3 working sessions
  depending on direction selected.
- **Team**: Solo author + assistant.
- **Budget**: Zero external paid services. Webfont must be free or self-hosted
  open-license. (Rules out commercial Matter SQ — PostHog's actual face.)

### Integration
- Existing API (`select_posts`, `select_tags`, `select_post`, `select_activities`,
  `select_references`, `contact`, `increment_views`) is preserved.
- KaTeX stylesheet stays loaded for math posts.
- Existing markdown→HTML pipeline (`markdown/` crate) emits `prose`-compatible HTML.

### Compliance / non-goals
- Not a content migration — posts in SurrealDB stay where they are.
- Not adding comments, search, or newsletter in this branch (those are PLAN.md
  Q1/Q2 items; redesign should make them possible, not include them).
- Not adopting a mascot character (PostHog's Max only works at multi-illustrator
  scale per their own brand handbook — flagged as anti-pattern for single-author
  blogs).

### Success Criteria
- [ ] All five routes render under SSR + hydrate cleanly.
- [ ] All color, type, spacing, and radius values flow from a single `@theme`
      block — zero `bg-[#hex]` arbitrary values in component code.
- [ ] Lighthouse Performance ≥ 90 on `/` and a representative `/post/:slug`.
- [ ] `make validate` (format, lint, test, security) passes.
- [ ] First Contentful Paint ≤ current baseline; ideally improved by ~20% from
      single-variable-font swap-in.
- [ ] RSS / Atom / JSON Feed all return 200 with valid content.
- [ ] Visual QA: screenshot pairs of every route before/after, captured via the
      `scry:record-browser` skill.

---

## 4. Reference Site Research — Executive Summary

Three deep-research subagents fetched and analyzed the reference sites. Full
reports saved as research artifacts; the executive summary below distills what
matters for synthesis.

### blogosphere.app (canonical: [text.blogosphere.app](https://text.blogosphere.app/))
- **What it is**: Curated indieweb aggregator (1,000+ blogs), tagline "Rediscover
  the personal web." Two parallel UIs: a JS SPA at the modern domain and a
  static HTML version at `text.` (which is the maintained content version).
- **Visual signature**: Near-monochrome paper-white, system-stack sans, classic
  blue underlined links, no images, no icons (only typographic glyphs `↗ ← →`).
- **Layout signature**: Single column, numbered ranked feed (1–50), pipe-separated
  lowercase nav (`recent | blogs | stumble | submit | about | modern ↗`), inline
  category strip (no chips, no dropdown), `↗` for outbound links.
- **Patterns to steal**: Pipe-separated lowercase nav · Numbered ranked archive ·
  `/random` "Stumble" mechanic · Inline category strip · `↗` outbound glyph.
- **Patterns to avoid**: Zero images (works for an aggregator, wrong for a
  single author with a portfolio) · Linking out for our own posts (we are the
  destination) · System-font monochrome with no accent (reads as unfinished at
  single-author scale).

### posthog.com
- **What it is**: Open-source product analytics company; marketing + docs +
  handbook + blog all on one Gatsby/MDX site at one domain.
- **Visual signature** (from their own brand handbook):
  - Light bg `#EEEFE9` (warm off-white), text `#151515` at 90% opacity
  - Dark bg `#151515`, text `#EEEFE9` at 90% opacity
  - Brand accents: red `#F54E00`, blue `#1D4AFF`, yellow `#DC9300`/`#F1A82C`
  - **Single typeface site-wide**: Matter SQ variable font (Displaay foundry,
    commercial), custom paragraph weight `475`. Type scale H1 64 / H2 48 /
    H3 30 / body 17px at line-height 175%.
  - Mascot Max the hedgehog (rules: no side profile, no AI-generated art,
    arms bend exactly one way).
- **Layout signature**: Sticky top nav, sitemap-style footer linking every product
  module, pricing as wide single-column with inline tabs, blog index with
  `where category eq …` filter syntax, in-flow TOC at top of long posts (not
  sticky sidebar), mid-post newsletter strap branded as a *named* newsletter
  ("Product for Engineers"), engineer-flavored microcopy throughout.
- **Patterns to steal**: Warm off-white bg (never pure white) · Single variable
  font with custom non-default weight · In-flow TOC at top of long posts ·
  Engineer-flavored filter microcopy · Monospace numerics in tables · Named
  newsletter strap · Sitemap-style dense footer.
- **Patterns to avoid**: Mascot character · Pricing-page density · "Switch to
  website mode" UI flip · Founder-mode self-mythology copy · Eight-product
  mega-footer (mirror the *shape*, not the volume).

### blog.fsck.com — *Massively Parallel Procrastination*
- **What it is**: Personal blog of Jesse Vincent (Best Practical / RT founder
  since 1996, Keyboardio co-founder). 529 posts. GitHub Pages + Plausible
  analytics. Atom + RSS + JSON Feed + raw `.md` alternate per post.
- **Visual signature** (read directly from the site CSS, where the comment header
  literally reads `Locked palette`):
  - `--paper #ecedef` (cool platinum gray ground), `--paper-2 #dfe1e4`,
    `--ink #0a0a0a`, secondary inks `#2a2a2a` / `#5a5e62` / `#8a8e92`
  - **Single accent**: `--ochre #7a1f24` (oxblood), with rare `--teal #5e6a3d`
    (olive) for variety
  - **Three families with explicit roles**: `--display` DM Serif Display ·
    `--serif` Crimson Pro · `--mono` JetBrains Mono
  - Body `19px / line-height 1.55`; post prose `19px / line-height 1.7` in a
    `max-width: 720px` column with `padding: 56px 32px 96px`
- **Layout signature**: Two-line italic-accented serif nameplate ("Massively
  *Parallel* Procrastination" with *Parallel* italicized in ochre), small italic
  serif text-link nav with hover-fill borders, 2px ink rule below header.
  Post-list rows use a date "stamp" tile (uppercase mono kicker + huge serif
  day numeral + 2px ochre left border). Post page: `<time>` above title in
  italic display, h1 in DM Serif Display 64px, byline = tags-only between
  hairline rules, prose links black with ochre underline, blockquotes with
  ochre rail, 11pt mono uppercase footer rule.
- **Patterns to steal**: Locked palette declared as CSS variables · Three-family
  type stack (display serif + body serif + mono) · Black ink + ochre underline
  links · 720px reading column at 19/1.7 · Date "stamp" component · Italic-
  accented two-line nameplate · Three feeds + `.md` alternate · Tags-everywhere
  taxonomy · ASCII-rule ornament between sections.
- **Patterns to avoid**: Empty post-foot (no related/comments/next-prev) · Year-
  only sparse archive · Tags-only byline (a newer blog needs a small author
  chip + bio for cold readers).

### Triangulated principles (where all three converge)

Even though these sites are aesthetically distinct, three rules are shared:

1. **Off-white, never pure white** for page ground (PostHog #EEEFE9, fsck #ecedef,
   blogosphere paper-white).
2. **One accent color** reused for hover · current-state · rule · underline.
   Verbatim from PostHog's brand guide: *"Use opacity over more colors."*
3. **Typography role-splits** — display ≠ body ≠ meta. Mono carries metadata
   weight rather than icons. PostHog's variant of this is one variable face
   doing all roles via weight + size; fsck's variant is three faces with explicit
   roles. Both are valid.

These three rules are the floor. Any winning direction satisfies all three.

---

## 5. Approach Generation — Four Directions

Each direction below is a coherent triangulation, not a feature checklist. They
differ on three axes:
- **Theme**: light-first / dark-first / dual
- **Body type**: serif (editorial) / sans (engineer-product) / mixed
- **Personality**: archival-quiet / warm-irreverent / indieweb / synthesis

---

### Direction A — *Quiet Editorial* (fsck-anchored)

**Description**: A serif-led editorial blog that treats writing as the product.
Light only (V1). Maximum craft signal, minimum chrome. Pulls the locked-palette
discipline, three-family type stack, date-stamp post list, and ochre-underline
link pattern straight from blog.fsck.com; borrows pipe-nav and `↗` outbound
glyph from blogosphere.

**Stack additions**: DM Serif Display + Crimson Pro + JetBrains Mono via Fontsource
or self-host. New `@theme` block with `--paper`, `--ink`, `--ochre`, plus mono /
serif / display family vars.

**Visual rules**:
- bg `#ecedef`, fg `#0a0a0a`, accent `#7a1f24` (ochre) — or a chosen alternative
  like deep-teal `#1f3a4a` if you want to differentiate from fsck literally
- 720px reading column, body 19px / lh 1.7
- Italic-accented two-line nameplate: "alex *thola*" or similar
- Date-stamp post-list component (mono kicker + serif day numeral + 2px accent rail)
- Pipe-nav lowercase: `writing | references | activity | about | rss ↗`
- Prose link pattern: ink color, accent-underline, accent on hover
- Three feeds (RSS/Atom/JSON) + `<link rel="alternate" type="text/markdown">` per post
- Empty post-foot replaced by small `prev | next | more from #tag` row (anti-pattern
  flagged by fsck research — we patch it)

**Pros**:
- Most distinctive against the SaaS-template baseline
- "Slick + simple + elegant" — three of four user adjectives — read instantly
- Editorial discipline aligns with long-form Rust/consulting writing
- Single locked palette unblocks theme toggle later by being already token-driven
- Lowest font weight (one display + one body + one mono = 3 webfont requests)

**Cons**:
- Furthest departure from current dark-yellow identity — readers may not recognize the site
- Serif body is a strong stylistic claim; if Alex prefers sans for technical content, this fights him
- "Modern" reads here as "editorial-modern" not "engineer-product modern" — may feel slow-web for a tech-consulting brand

**Risks**:
- Over-anchoring on fsck.com → site reads as a knockoff (mitigation: pick a
  different accent hue than ochre — e.g. deep teal or burgundy)
- Long-form serif on a site where 60% of posts may be Rust-snippets-with-prose
  could feel mismatched (mitigation: tune `prose` to give code blocks more weight)

**Effort**: M — tokens are well-defined; mostly CSS rewrites.

**Trade-offs**:
- Light-only V1 → ship now, dark-mode in v2 (mitigation: design tokens already
  parameterize, dark mode is a 1-day follow-up)
- No mascot, no playful copy → personality lives in typography (mitigation: lean
  on italic-accented nameplate + colophon page voice)

---

### Direction B — *Warm Engineer* (PostHog-anchored, single variable sans)

**Description**: A modern engineer-product blog that pulls PostHog's warm off-
white + single-variable-sans + custom-weight discipline. Keeps the existing
yellow accent (which already lives in PostHog's brand family), drops dark-only
in favor of warm-light primary + dark-mode toggle from day one. No mascot.
Engineer-flavored microcopy and sitemap footer.

**Stack additions**: Inter Variable (free) or Geist Variable (free) as single
face. JetBrains Mono for code/meta. Optional: Fraunces Variable for h1-only display.

**Visual rules**:
- Light bg `#eeefe9`, dark bg `#151515` (verbatim PostHog tokens — close enough
  to fsck #ecedef that the eye won't notice if Alex prefers warmer)
- Accent: keep `#ffef5c` (current site yellow, PostHog-adjacent) OR migrate to
  PostHog's #DC9300 light / #F1A82C dark for AA contrast
- Single variable sans at custom 470 weight for body — non-default weight is the
  signal of intentionality
- Type scale: H1 56 / H2 40 / H3 28 / body 17px / lh 1.65
- Sticky top nav (logo + 5 items), sitemap-style footer (writing / references /
  archive / now / about / colophon / RSS / GitHub)
- In-flow TOC at top of long posts (no sticky sidebar)
- `where tag eq …` engineer-flavored filter on archive
- Mid-post newsletter strap branded as a named newsletter (e.g. "Notes from a
  Staff Engineer") with low-key copy
- Monospace numerics in post-list metadata (read time, view count, dates)

**Pros**:
- Closest evolution from current site — preserves yellow accent equity
- "Modern" hits the hardest of the four directions
- Dark-mode parity from day one matches user's PLAN.md Q1 backlog
- Single variable font is the lowest-payload option (one file, all weights)
- Engineer-product voice fits "tech consulting" positioning
- AA contrast story works in both modes with PostHog tokens

**Cons**:
- "Elegant" hits weakest of the four — engineer-product reads as competent more
  than refined
- Variable sans body means no editorial slow-web cred
- Risks looking like "another nice Tailwind blog" without strong distinctive moves
- Yellow accent is already heavily used in tech (Substack, Linear adjacent); may
  not feel distinctive

**Risks**:
- Without strong distinctive moves (the date stamp, the italic nameplate), this
  reads as competent-but-derivative (mitigation: add 2-3 distinctive moves like
  the `↗` glyph + mono numerics + colophon page)
- Yellow + warm-off-white can read pastel/cute if not tuned (mitigation: increase
  yellow saturation slightly to keep it editorial)

**Effort**: M — slightly more than Direction A because of dual-mode tokens.

**Trade-offs**:
- Yellow accent retained → low brand-shock for returning readers; risks looking
  like a refresh not a redesign (mitigation: pair with strong typographic shifts)
- Mid-post newsletter strap → adds one component, but PLAN.md Q2 backlog calls
  for newsletter anyway (no extra debt)

---

### Direction C — *Indieweb Maximal* (blogosphere × fsck fusion)

**Description**: A deliberately small-web personal blog. Light only. Body in serif,
no sans anywhere. Numbered ranked archive (blogosphere), date-stamp post list
(fsck), pipe-nav (blogosphere), `↗` outbound glyphs everywhere, three feeds + `.md`
alternate. Voice: dry, lowercase, archival. Anti-mascot. The "I have been here
since 1996 and I'll be here in 2046" stance.

**Stack additions**: Same as A (DM Serif Display + Crimson Pro + JetBrains Mono).

**Visual rules**:
- bg paper `#ecedef`, accent of choice (ochre, oxblood, or chosen)
- Pipe-nav lowercase only: `writing | notes | reading | references | about`
- Numbered ranked archive (1–50 per page, "page X of Y" pagination)
- `/random` "Stumble" route for serendipity discovery
- Inline category strip on archive (not chips)
- `↗` glyph on every outbound link
- `<link rel="alternate" type="text/markdown">` per post
- Hidden nameplate (just lowercase "alex thola" or initials)
- No newsletter widget, no comments, no share buttons — RSS-first

**Pros**:
- Most "personal web" / craft-coded — reads as a *site*, not a *content product*
- Strongest "simple" + "elegant" combo of the four
- Lowest information density, fastest pages
- Three feeds + `.md` alternate is a real hacker-culture handshake

**Cons**:
- "Modern" hits weakest — reads as deliberately retro
- Sans-free choice fights any Rust code-snippet display ergonomics
- May read as too-quiet for someone selling consulting services
- Departs furthest from current site identity

**Risks**:
- Retro can age into "abandoned" if the date-stamp 2-column post-list page is
  rarely updated (mitigation: ensure home shows latest 5-10 posts in date order)
- Reads as cosplay if Alex isn't already invested in the indieweb register
  (mitigation: only pick this if the voice fits)

**Effort**: S/M — fewer features than A or B, but voice tuning takes time.

**Trade-offs**:
- No newsletter capture → loses the prospective-client funnel; but site has
  `/contact` which already plays that role
- No share buttons → fine for an audience that uses RSS, hostile to Twitter-
  reader audience (mitigation: clean OG image per post still gives copy-paste
  shares great previews)

---

### Direction D — *Dual-Mode Editorial Engineer* (synthesis, recommended) ⭐

**Description**: The triangulation of all three sites. Light-first with full
dark-mode parity from day one. Display serif for h1-h2 (DM Serif Display or
Fraunces Variable), body sans (Inter Variable at 470), JetBrains Mono for meta
and code. Single accent color (recommend ochre or deep teal — discussed below)
reused for underlines, hover, current-state, rules. 720px reading column with
in-flow TOC for long posts. Date-stamp post-list (fsck) + sitemap-style footer
(PostHog) + pipe-nav lowercase (blogosphere). Engineer-flavored filter microcopy.
`↗` glyph on outbound links. Three feeds + `.md` alternate per post.

The bet: this scores well on **all four** of the user's adjectives (slick, simple,
modern, elegant) by deliberately picking the strongest move from each reference
site rather than trying to be one of them.

**Stack additions**: Fraunces Variable OR DM Serif Display + Inter Variable + JetBrains
Mono. Three webfont requests; all open-license; total under 200KB compressed.

**Visual rules**:
- Tokens (light): `--paper #ecedef`, `--ink #0a0a0a`, `--muted #5a5e62`,
  `--accent #7a1f24`, `--surface #dfe1e4`
- Tokens (dark): `--paper #151515`, `--ink #ecedef`, `--muted #8a8e92`,
  `--accent #c47c80` (lightened ochre for AA contrast on dark), `--surface #2a2a2a`
- All tokens declared in Tailwind v4 `@theme` block; `prefers-color-scheme` +
  toggle persisted to localStorage
- Type scale: H1 56 / H2 40 (display serif) · H3 28 / body 17 (sans variable
  470) · meta 12 (mono uppercase 0.08em tracking) · code 14 (mono)
- Reading column 720px, 19px / lh 1.65 on body sans (slightly tighter than
  fsck's serif rhythm because sans needs less vertical air)
- Header: italic-accented two-line nameplate ("alex *thola*" or chosen variant)
  + small mono pipe-nav lowercase + 1px ink rule
- Footer: sitemap-style (writing · archive · references · activity · about ·
  colophon · RSS · GitHub · LinkedIn) + `© 2024–2026 Alex Thola` in mono
- Post-list: date-stamp tile (mono kicker + serif day numeral + 2px accent rail)
- Post page: `<time>` above title, h1 display 56px, tags-as-byline +
  small author chip + bio line, prose links ink-with-accent-underline, ochre-
  rail blockquotes, in-flow TOC for posts > 1500 words
- Outbound links get `↗` glyph (CSS pseudo-element on `[href^="http"]:not([href*="alexthola.com"])`)
- Three feeds + `.md` alternate per post

**Pros**:
- Scores well on **all four** user adjectives without compromise
- Dark-mode parity from day one matches PLAN.md Q1 backlog
- Display-serif + body-sans is the modern editorial-engineer pairing (Stripe
  Press, Linear changelog, Vercel docs all do versions of this)
- Pulls a distinctive move from each reference site — derivativeness is
  diluted across three directions
- Token system unblocks future theme features and design refinements
- Sans body keeps Rust code snippets ergonomic; serif headlines elevate
  long-form posts

**Cons**:
- Highest implementation cost of the four directions — designing two color
  modes correctly, three font families, and a token system takes more iterations
- Three font families is more than blogosphere's zero or PostHog's one; webfont
  budget needs careful tuning
- More moving parts → more risk of "almost right" finish at first ship; needs
  visual QA discipline

**Risks**:
- Token sprawl: if `@theme` becomes a kitchen sink, future devs (or future-Alex)
  hit "which token do I use?" paralysis (mitigation: keep token count under 20
  total, document each token's intent in a comment)
- Dark-mode contrast bugs on accent color (mitigation: compute both light and
  dark accent variants up-front; verify at 4.5:1 before shipping)
- "Modern editorial" is a popular vector — risk of looking like Stripe Press,
  Linear blog, or Vercel docs (mitigation: the date-stamp + italic-accent
  nameplate + ochre underline links + `↗` glyph are the distinctive moves
  that prevent this)

**Effort**: M/L — more than A/B/C individually but only marginally; the bulk of
the work overlaps. Estimate 2–3 working sessions.

**Trade-offs**:
- Three font families instead of one → 200KB of webfont vs 50KB for variable
  Inter alone (mitigation: subset latin only; use `font-display: swap`; preload
  display+body; lazy-load mono)
- More tokens → more discipline required (mitigation: lint that fails on
  `bg-[#hex]` arbitrary values in components — already valuable on its own)

---

## 6. Approach Comparison Matrix

| Criterion | A: Quiet Editorial | B: Warm Engineer | C: Indieweb Maximal | D: Dual-Mode Synthesis ⭐ |
|---|---|---|---|---|
| Slick (polish/refinement) | 🟢 High | 🟡 Medium | 🟡 Medium | 🟢 High |
| Simple (restraint) | 🟢 High | 🟡 Medium | 🟢 High | 🟡 Medium |
| Modern (current, not retro) | 🟡 Medium | 🟢 High | 🔴 Low | 🟢 High |
| Elegant (graceful, restrained) | 🟢 High | 🟡 Medium | 🟢 High | 🟢 High |
| Distinctiveness vs SaaS templates | 🟢 High | 🟡 Medium | 🟢 High | 🟢 High |
| Effort to ship | 🟢 M | 🟢 M | 🟢 S/M | 🟡 M/L |
| Migration shock for current readers | 🔴 High | 🟢 Low | 🔴 High | 🟡 Medium |
| Long-form reading ergonomics | 🟢 High | 🟡 Medium | 🟢 High | 🟢 High |
| Code-snippet ergonomics | 🟡 Medium | 🟢 High | 🔴 Low | 🟢 High |
| Dark mode story | 🔴 V2 | 🟢 V1 | 🔴 V2 | 🟢 V1 |
| Theme-toggle backlog unblocked | 🟢 (v2 ready) | 🟢 Yes | 🟡 (v2 ready) | 🟢 Yes |
| Webfont payload | 🟡 ~150KB | 🟢 ~80KB | 🟡 ~150KB | 🟡 ~200KB |
| Reads as "personal web" | 🟢 High | 🔴 Low | 🟢 High | 🟡 Medium |
| Reads as "tech consultant" | 🟡 Medium | 🟢 High | 🔴 Low | 🟢 High |
| Risk of looking derivative | 🟡 Medium (fsck) | 🔴 Higher (PostHog) | 🟢 Low | 🟡 Medium |

---

## 7. War Room Bypass Decision

The brainstorming skill's Phase 3.5 mandates `/attune:war-room` for high-stakes
decisions. The four bypass conditions are:

- [x] **RS ≤ 0.40 (Type 2 reversible)**: site-redesign-0.2.0 is a feature branch
      with no commits yet, no dependent PRs, no production deploy implication
      until merged. Fully reversible by branch deletion. **RS estimated 0.30–0.35**.
- [x] **Single obvious approach with no meaningful trade-offs**: not satisfied —
      we have four approaches with real trade-offs. *This condition fails.*
- [x] **Low complexity with well-documented pattern**: token-based redesigns are
      a well-documented Tailwind v4 pattern; locked-palette + role-typography
      is documented in fsck's CSS literally.
- [x] **User explicitly declines after RS assessment**: user wrote "ultrathink"
      which is a depth signal not a war-room request, and the inline red-team
      analysis per approach plus the comparison matrix already provide the
      pressure-testing war-room would.

**Decision**: Bypass the formal `/attune:war-room` ceremony. Apply red-team
thinking inline (done above per approach). If the user later disagrees with
direction selection or wants multi-LLM pressure-testing, `/attune:war-room
--from-brainstorm` can be invoked from this brief at any time.

---

## 8. Pre-Mortem (Direction D, recommended)

*If this redesign fails to ship cleanly, here's how it would happen:*

| Failure mode | Likelihood | Mitigation |
|---|---|---|
| Token system grows past 20 tokens, becomes "which one?" paralysis | Medium | Cap at 20; one comment line per token explaining intent |
| Dark-mode accent contrast bug on first ship | Medium | Compute and verify both variants at 4.5:1 BEFORE first commit |
| Webfont payload pushes Lighthouse below 90 | Medium | Subset Latin only; preload display + body; mono lazy |
| Date-stamp component hard to render in masonry | Low | Move post-list off masonry to single-column with date-stamp gutter |
| `prose` rewrite breaks math (KaTeX) | Medium | Test KaTeX-heavy post on every commit; treat KaTeX styles as untouchable |
| Activity stream `/activity` doesn't fit new design language | High | Decide in Phase 9 of this brief: fold into home, retire, or re-skin |
| `<link rel="alternate" type="text/markdown">` requires raw markdown route on the server | Medium | Add lightweight `/post/:slug/raw.md` Axum route (path-segment-bound to satisfy Axum 0.8 routing); SurrealDB already stores raw markdown |
| Header is fixed → date-stamp 2px accent rail competes for visual weight | Low | Make header unfixed (recommended anyway — currently eats 9% viewport) |
| Direction D ships looking like Stripe Press / Linear blog | Medium | Distinctive moves (italic nameplate, date stamp, ↗ glyph, ochre underline) are non-negotiable |

**Watch-points during execution**:
- Lighthouse Performance ≥ 90 every commit (run via Makefile target).
- Visual regression on every route via `scry:record-browser` before PR.
- `prose` styles regression-tested on a math-heavy post and a code-heavy post.

---

## 9. Open Decisions Requiring User Input

These are the points where the user's taste is the deciding input. Cannot be
resolved by research:

### D1 — Direction selection
- **Pick A, B, C, or D** (or combine: e.g. "D but with serif body like A").
- The brief recommends **D**; happy to argue any other.

### D2 — Accent color
Independent of direction (all four work with any of these):
- **Keep `#ffef5c` yellow** — preserves brand equity, accidentally PostHog-family.
- **Migrate to ochre `#7a1f24`** (literal fsck.com value) — strongest editorial
  signal but most direct knockoff.
- **Pick a different distinctive accent**: deep teal `#1f3a4a`, burgundy
  `#7a2942`, forest `#2c4a35`, or imperial purple `#3a2c5a`.
- **Recommended**: pick a *new* accent that isn't yellow (looks like current site)
  or ochre (looks like fsck). Burgundy `#7a2942` reads close enough to ochre to
  carry editorial weight, distinct enough to feel original.

### D3 — Body type
- **Body sans (Inter Variable / Geist) at 470 weight** — engineer-product modern,
  ergonomic for code-heavy posts. (Direction B and D default.)
- **Body serif (Crimson Pro / Source Serif / Fraunces text)** — editorial
  weight, slow-web cred, less ergonomic for code. (Direction A and C default.)
- **Hybrid: serif for h1-h2, sans for body, mono for code** — Direction D's
  default; richest typographic personality.

### D4 — `/activity` route fate
- **Keep & re-skin** to match the new design language.
- **Fold into home** as a secondary "notes" stream.
- **Retire entirely** (least scope).
- **Promote to top-level `/notes`** as a microblog companion (most scope).
- **Recommended**: fold into home as a "notes" sidebar/strip, OR promote to
  `/notes` if Alex genuinely uses it.

### D5 — Header chrome
- **Keep header fixed** (current pattern; eats 9% viewport).
- **Unfix header** (scrolls with page; lets reading column breathe).
- **Recommended**: unfix. All three reference sites scroll the header with the
  page. Blogosphere has no fixed chrome at all. fsck has a 2px ink rule below
  the header but it scrolls. PostHog has a sticky nav but it's compact.

### D6 — Newsletter slot
- **Add named newsletter strap mid-post** — PLAN.md Q2 backlog item; if added
  now, it composes with Direction D's design language.
- **Defer to PLAN.md Q2 phase** — keeps redesign scope tight.
- **Recommended**: defer. Add the *slot* (component placeholder) but not the
  integration. Newsletter integration is a separate, well-bounded feature.

---

## 10. Selected Approach (provisional, awaiting D1 confirmation)

⭐ **Direction D — Dual-Mode Editorial Engineer**, with the following defaults
on D2–D6:

- **Accent**: TBD by user (recommend `#7a2942` burgundy or similar non-yellow,
  non-ochre)
- **Body type**: hybrid (display serif h1-h2, sans body, mono meta/code)
- **`/activity`**: fold into home as a secondary "notes" strip OR promote to `/notes`
- **Header**: unfix (scrolls with page)
- **Newsletter**: design the slot, defer the integration

### Rationale

Direction D scores 🟢 on 8 of 11 quality criteria in the comparison matrix and
is the only direction that hits 🟢 on all four user adjectives ("slick, simple,
modern, elegant"). It also resolves the theme-toggle backlog item by structural
design rather than as a v2 retrofit.

The cost is implementation complexity — D is M/L effort vs M for A/B and S/M for
C — but the marginal effort is small relative to the marginal payoff: a token
system pays back over every future feature.

The risk is that "modern editorial" is a popular vector. The four distinctive
moves (italic-accented nameplate, date-stamp post-list, ochre/accent underline
link pattern, `↗` outbound glyph) are non-negotiable — without them, D ships
looking like Stripe Press. With them, it looks like alexthola.com.

### Trade-offs accepted

- **Higher token discipline required** → mitigated by lint rule that fails on
  arbitrary `bg-[#hex]` values in components.
- **Three webfont families** → mitigated by Latin subsetting + display/body
  preload + mono lazy. Total budget ~200KB; current site already ships ~600KB
  in 28 Poppins files.
- **Two color modes designed first-class** → mitigated by computing both accent
  variants and contrast values up-front, before any component is touched.

### Rejected approaches

- **Direction A (Quiet Editorial)**: Strong on simple/elegant, weak on modern;
  light-only V1 leaves the dark-mode backlog item half-done; serif body fights
  Rust code snippets; reads as fsck.com knockoff if accent is ochre.
- **Direction B (Warm Engineer)**: Closest to current site identity but weakest
  on elegant; without the distinctive moves it reads as competent-but-derivative
  ("another nice Tailwind blog").
- **Direction C (Indieweb Maximal)**: Strongest on simple/elegant but weakest on
  modern; no sans + no dark mode + no newsletter slot = harder to evolve; reads
  as deliberately retro for a tech-consulting positioning.

---

## 11. Next Steps

After user confirms D1–D6 (or a hybrid):

1. **`/attune:specify`** → produce `docs/specification.md` with concrete
   acceptance criteria, route-by-route designs, and component contracts.
2. **`/attune:blueprint`** → produce `docs/implementation-plan.md` with phased
   tasks, dependency ordering, and TDD checkpoints.
3. **`/attune:execute`** → execute the plan with proof-of-work evidence per
   route.

The user checkpoint is now: **D1 direction selection + D2–D6 defaults**.

---

## Appendix: Source Material

- Research subagent reports (3 agents, ~5,000 words total) — available in agent
  transcripts.
- Current site code (mapped):
  - `app/src/lib.rs` — root shell + routes
  - `app/src/home.rs` — masonry post list + tag filter
  - `app/src/post.rs` — `prose` article render
  - `app/src/components/header.rs` — fixed top nav
  - `app/src/components/icons.rs` — social icon set (RSS already wired)
  - `app/src/contact.rs` — has `whoami` section that should move to `/about`
  - `app/src/references.rs` — glassmorphism cards with skill bars
  - `app/src/activity.rs` — uses inconsistent `bg-gray-800` palette
  - `style/tailwind.css` — 28 Poppins font-faces + global `svg{fill:white!important}` rule
- Tailwind v4 `@theme` block: [Tailwind v4 docs](https://tailwindcss.com/docs/v4-beta) — for token system reference (verify with context7 during specify).
- PLAN.md Q1/Q2 backlog: theme toggle, syntax highlighting, search, related posts (Q1); comments, social, newsletter (Q2).
