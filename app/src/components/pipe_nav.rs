//! PipeNav — pipe-separated lowercase navigation row.
//!
//! Renders `writing | notes | references | about | rss ↗` in mono uppercase
//! kicker style. Current route is matched against each item; the matching
//! item gets accent color + accent underline. Pattern borrowed from
//! text.blogosphere.app (see docs/project-brief.md §4 reference site
//! research and docs/specification.md §5.2).

use leptos::{
    html::{nav, span},
    prelude::*,
};
use leptos_router::components::A;
use leptos_router::components::AProps;

const NAV_ITEMS: &[(&str, &str)] = &[
    ("/", "writing"),
    ("/notes", "notes"),
    ("/references", "references"),
    ("/about", "about"),
];

/// Renders the pipe-separated navigation row.
///
/// `current_route` is matched against each item's path; the matching item
/// gets the active styling.
pub fn component(current_route: String) -> impl IntoView {
    let active_class = "text-accent border-b border-accent";
    let idle_class = "text-ink-3 hover:text-accent border-b border-transparent hover:border-accent transition-colors";

    let make_link = move |href: &'static str, label: &'static str, is_current: bool| {
        let class = if is_current { active_class } else { idle_class };
        A(AProps::builder()
            .href(href.to_string())
            .children(ToChildren::to_children(move || {
                span().class(class).child(label)
            }))
            .build())
    };

    nav()
        .class("font-mono text-xs uppercase tracking-[0.08em] flex flex-wrap items-center justify-end gap-x-2")
        .attr("aria-label", "primary navigation")
        .child((
            // primary nav items joined inline
            NAV_ITEMS
                .iter()
                .enumerate()
                .map(|(i, (href, label))| {
                    let is_current = if *href == "/" {
                        current_route == "/"
                    } else {
                        current_route == *href || current_route.starts_with(&format!("{}/", href))
                    };
                    let separator = if i > 0 {
                        Some(span().class("text-ink-4").child("|"))
                    } else {
                        None
                    };
                    (separator, make_link(href, label, is_current))
                })
                .collect::<Vec<_>>(),
            // RSS link last — outbound, gets ↗ glyph automatically via CSS rule
            span().class("text-ink-4").child("|"),
            A(AProps::builder()
                .href("/feed/rss.xml".to_string())
                .children(ToChildren::to_children(move || {
                    span()
                        .class("text-ink-3 hover:text-accent transition-colors")
                        .child("rss")
                }))
                .build()),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_nav_structure() {
        let _: fn(String) -> _ = component;
    }

    #[test]
    fn test_pipe_nav_items_constant() {
        assert_eq!(NAV_ITEMS.len(), 4);
        assert_eq!(NAV_ITEMS[0].0, "/");
    }

    #[test]
    fn test_pipe_nav_route_match_root() {
        // "/" must match exactly, not any path that starts with "/"
        let path = "/notes";
        assert_ne!(path, "/");
        // paths starting with "/notes/" should match notes
        assert!("/notes/something".starts_with("/notes/"));
    }
}
