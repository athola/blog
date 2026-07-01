# Design Decisions: 0.2.0 Redesign

A record of the design direction chosen for the alexthola.com redesign, the
alternatives weighed, and why. The living token/component reference is
[`design-system.md`](./design-system.md). This document explains the *why*
behind it.

## Context

The redesign targeted four qualities: slick, simple, modern, elegant. Three
reference sites were studied in depth:

- **blog.fsck.com**: locked-palette discipline (every color declared as a token
  at the top of the stylesheet), three-family type stack, date-stamp post list,
  ochre underline links.
- **posthog.com**: warm off-white ground, a single accent reused everywhere via
  opacity ("use opacity over more colors"), one variable typeface across roles.
- **blogosphere.app**: lowercase pipe-nav, `↗` outbound-link glyph, near-zero
  chrome.

Three principles held across all three and became the floor for any direction:

1. **Off-white ground, never pure white** (`#ecedef` here).
2. **One accent color** reused for hover, current-state, rules, and underlines.
3. **Typographic role-splits**: display ≠ body ≠ meta, with mono carrying
   metadata weight instead of icons.

## Decision

**Direction D: Dual-Mode Editorial Engineer.** A light-first design with
full dark-mode parity from day one, synthesizing the strongest move from each
reference site rather than imitating any one of them:

- **Type**: display serif for H1–H2, sans body (variable, ~470 weight), mono for
  meta and code: the editorial-engineer pairing.
- **Accent**: burgundy `#7a2942`, close enough to fsck's ochre to carry
  editorial weight, distinct enough to avoid reading as a knockoff, and a clean
  break from the old `#ffef5c` yellow.
- **Color modes**: light and dark both designed first-class, tokens declared in
  the Tailwind v4 `@theme` block, accent contrast verified at ≥4.5:1 in both.
- **Distinctive moves (non-negotiable)**: italic-accented nameplate, date-stamp
  post-list tiles, accent-underline prose links, and the `↗` outbound glyph.
  Without these, the design ships looking like Stripe Press. With them, it looks
  like alexthola.com.

Supporting decisions:

| Decision | Resolution |
|----------|-----------|
| Accent color | Burgundy `#7a2942` (light) / `#c47c80` (dark, 6.7:1 AA) |
| Body type | Hybrid: serif headings, sans body, mono meta/code |
| `/activity` route | Promoted to a top-level `/notes` microblog |
| Header chrome | Unfixed: scrolls with the page so the reading column breathes |
| Newsletter | Slot designed, integration deferred |

## Alternatives Considered

The four directions were scored across eleven criteria. Direction D scored
highest on the four target adjectives and was the only one to land all four
without compromise.

- **A: Quiet Editorial** (fsck-anchored, serif-led, light-only V1). Strong on
  simple and elegant, weak on modern. Light-only would have left the dark-mode
  backlog half-done, a serif body fights Rust code snippets, and an ochre accent
  reads as an fsck.com knockoff.
- **B: Warm Engineer** (PostHog-anchored, single variable sans, keeps yellow).
  Closest to the existing identity and lowest migration shock, but weakest on
  elegant. Without distinctive moves it reads as "another competent Tailwind
  blog."
- **C: Indieweb Maximal** (blogosphere × fsck fusion). Strongest on simple and
  elegant but weakest on modern: lacking a sans face, dark mode, and a newsletter
  slot made it the hardest to evolve, and it read as deliberately retro for a
  consulting brand.

## Consequences

Direction D carries the highest implementation cost of the four (two color
modes, three font families, and a token system take more iterations), but the
marginal effort over the alternatives is small and the token system pays back on
every future feature. The accepted trade-offs and their mitigations:

- **Higher token discipline** → a lint rule fails the build on arbitrary
  `bg-[#hex]` values in components.
- **Three webfont families (~200KB)** → Latin subsetting, preload display and
  body, lazy-load mono. (The previous site shipped ~600KB across 28 Poppins files, so
  this is a net reduction.)
- **Two color modes designed up front** → both accent variants and their contrast
  ratios were computed before any component was touched.

The redesign was scoped tightly: theme-toggle UI polish, newsletter integration,
JSON Feed, comments, and a syntax-highlighting overhaul were all explicitly
deferred. See [`specification.md`](./specification.md) §8 for the full
out-of-scope list.
