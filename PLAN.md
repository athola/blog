# Roadmap

This document outlines planned features and technical improvements for the blog engine.

## Site Redesign 0.2.0 — Direction D (in flight, `site-redesign-0.2.0`)

The site is being redesigned per the **Dual-Mode Editorial Engineer** direction
synthesized from blogosphere.app, posthog.com, and blog.fsck.com.

- **Brief**: [`docs/project-brief.md`](./docs/project-brief.md)
- **Specification**: [`docs/specification.md`](./docs/specification.md)
- **Implementation plan**: [`docs/implementation-plan.md`](./docs/implementation-plan.md)
- **Design system reference**: [`docs/design-system.md`](./docs/design-system.md)

Already shipped on the redesign branch:
- Tailwind v4 `@theme` token system (light + dark modes), Fraunces + Inter +
  JetBrains Mono webfonts, removal of 28 Poppins font-faces, removal of
  global `svg{fill:white!important}` rule.
- Eight new components: Nameplate, PipeNav, DateStamp, PostListRow, TagStrip,
  Footer, Toc, plus the outbound `↗` glyph CSS rule.
- All five existing routes refactored to tokens (home, post, references,
  contact, notes-renamed-from-activity).
- Three new routes: `/archive` (year-grouped chronological), `/about`
  (lifted from contact whoami), `/colophon` (stack + fonts + license).
- Server-side: `/random` stumble redirect, `/post/:slug.md` raw markdown
  alternate, `/feed/feed.xml` Atom feed, `/feed/rss.xml` RSS alias,
  `/activity` → `/notes` 301 permanent redirect.
- SEO: per-post canonical + markdown alternate `<link>`, Schema.org
  Article and Person JSON-LD.
- A11y: skip-link, `:focus-visible` outline, `prefers-reduced-motion`,
  AA contrast in both modes.
- Token discipline guard: `make lint-tokens` blocks `bg-[#hex]` arbitrary
  values in component code.

## Q1 2026: Reader Experience

This quarter improves the core reading experience.

-   **Theme Toggle UI**: The token system already supports dark mode (Sprint 0
    of the redesign); the remaining work is a UI control that flips
    `[data-theme]` on `<html>` and persists to `localStorage` under key
    `alexthola-theme`.
-   **Syntax Highlighting**: Add server-side syntax highlighting for code blocks to improve readability and performance. This will likely involve integrating a Rust library like `syntect` during the Markdown rendering process.
-   **Post Search**: Implement a fast, server-side search feature. This will allow readers to find posts by keyword without relying on a third-party search provider. The initial implementation will likely use a simple full-text search index within SurrealDB.
-   **Related Articles**: Display a list of related posts at the end of each article to encourage further reading. The relationship will be determined by shared tags or content similarity. (The redesign added a placeholder "more from this tag →" link; this Q1 item upgrades it to a real recommendation.)

## Q2 2026: Community Features

This quarter adds features for reader interaction.

-   **Comments**: A self-hosted commenting system will be added to allow for discussions on posts. The implementation will focus on privacy and performance, avoiding third-party tracking. It will be built directly within the existing Rust and SurrealDB stack.
-   **Social Sharing**: Implement lightweight social sharing links that use direct URLs rather than third-party JavaScript widgets. This will allow readers to share content easily without sacrificing page load performance or privacy.
-   **Newsletter Signup**: Add a newsletter signup form, likely integrated with a self-hosted or privacy-focused email provider. The focus will be on a non-intrusive design. (The redesign already designs the placement; this Q2 item adds the integration.)

## Backlog

These items are under consideration for future quarters.

-   **Admin Interface**: A simple, secure web interface for creating, editing, and managing posts. This would reduce the need to interact directly with the database or filesystem for content management.
-   **Offline Reading**: Implement basic Progressive Web App (PWA) functionality to allow articles to be read offline. This would improve the experience for readers with intermittent connectivity.
-   **AI-Powered Suggestions**: Experiment with using local, privacy-preserving AI models to suggest tags or generate summaries for new posts. This is a low-priority exploration, contingent on the maturity and performance of local AI models.