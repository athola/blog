//! Toc — in-flow table of contents for long posts.
//!
//! Renders an `<aside>` block listing h2 anchors with mono uppercase
//! "ON THIS PAGE" kicker. Conditional: returns an empty view when fewer
//! than 4 headings are passed (short posts don't need a TOC).
//!
//! Pattern borrowed from posthog.com's in-flow TOC at the top of long
//! blog posts (see docs/project-brief.md §4 and docs/specification.md §5.6).

use leptos::{
    either::Either,
    html::{aside, h3, li, ol},
    prelude::*,
};

/// One heading entry in the TOC.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TocHeading {
    /// Heading level (2 = h2, 3 = h3). Only h2 renders by default.
    pub level: u8,
    /// Display text of the heading.
    pub text: String,
    /// Anchor id (without `#` prefix).
    pub anchor: String,
}

/// Minimum number of headings required to render the TOC. Posts with fewer
/// headings get no TOC.
pub const TOC_MIN_HEADINGS: usize = 4;

/// Render the TOC — or an empty view if `headings.len() < TOC_MIN_HEADINGS`.
pub fn component(headings: Vec<TocHeading>) -> impl IntoView {
    if headings.len() < TOC_MIN_HEADINGS {
        return Either::Left(().into_view());
    }

    Either::Right(
        aside()
            .class("hidden md:block border-l-2 border-accent pl-5 my-8 max-w-prose")
            .attr("aria-label", "table of contents")
            .child((
                h3().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3 mb-3")
                    .child("On this page"),
                ol().class("flex flex-col gap-1.5 list-none").child(
                    headings
                        .into_iter()
                        .filter(|h| h.level == 2)
                        .map(|h| {
                            li().child(
                                leptos::html::a()
                                    .href(format!("#{}", h.anchor))
                                    .class(
                                        "text-ink-2 text-sm hover:text-accent transition-colors no-underline",
                                    )
                                    .child(h.text),
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            ))
            .into_view(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_structure() {
        let _: fn(Vec<TocHeading>) -> _ = component;
    }

    #[test]
    fn test_toc_min_headings_constant() {
        assert_eq!(TOC_MIN_HEADINGS, 4);
    }

    #[test]
    fn test_toc_heading_clone_eq() {
        let h = TocHeading {
            level: 2,
            text: "Foo".into(),
            anchor: "foo".into(),
        };
        let h2 = h.clone();
        assert_eq!(h, h2);
    }
}
