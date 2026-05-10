//! Footer — sitemap-style three-column footer with social row and copyright.
//!
//! Replaces the Sprint 0 transitional footer in app/src/lib.rs with the
//! full sitemap layout per spec §3.2. Three columns (Writing, References,
//! About) collapse to a single column on mobile. Social icons row appears
//! below columns; copyright in mono uppercase below social.
//!
//! Pattern borrowed from posthog.com (see docs/project-brief.md §4
//! reference site research).

use crate::components::icons;
use chrono::{Datelike as _, Utc};
use leptos::{
    html::{div, footer as footer_el, h3, p, ul},
    prelude::*,
};
use leptos_router::components::{A, AProps};

/// Render the sitemap-style footer.
pub fn component() -> impl IntoView {
    footer_el()
        .class("relative mt-24 pt-12 pb-10 border-t-2 border-rule")
        .child(
            div()
                .class("container mx-auto max-w-4xl px-4 md:px-0")
                .child((
                    div()
                        .class("grid grid-cols-1 sm:grid-cols-3 gap-8 mb-10")
                        .child((
                            footer_column(
                                "Writing",
                                vec![
                                    ("/", "Latest"),
                                    ("/archive", "Archive"),
                                    ("/notes", "Notes"),
                                ],
                            ),
                            footer_column(
                                "References",
                                vec![("/references", "Portfolio"), ("/contact", "Contact")],
                            ),
                            footer_column(
                                "About",
                                vec![
                                    ("/about", "Bio"),
                                    ("/colophon", "Colophon"),
                                    ("/feed/rss.xml", "RSS"),
                                    ("/feed/feed.xml", "Atom"),
                                ],
                            ),
                        )),
                    div()
                        .class("flex justify-center mb-8")
                        .child(icons::component()),
                    p().class(
                        "font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4 text-center",
                    )
                    .child(format!(
                        "© 2024–{} ALEX THOLA. POWERED BY RUST + LEPTOS.",
                        Utc::now().year()
                    )),
                )),
        )
}

fn footer_column(title: &'static str, links: Vec<(&'static str, &'static str)>) -> impl IntoView {
    div().child((
        h3().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3 mb-3")
            .child(title),
        ul().class("flex flex-col gap-1.5").child(
            links
                .into_iter()
                .map(|(href, label)| {
                    leptos::html::li().child(A(AProps::builder()
                        .href(href.to_string())
                        .children(ToChildren::to_children(move || {
                            leptos::html::span()
                                .class("text-ink-2 text-sm hover:text-accent transition-colors")
                                .child(label)
                        }))
                        .build()))
                })
                .collect::<Vec<_>>(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_structure() {
        let _: fn() -> _ = component;
    }
}
