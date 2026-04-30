//! TagStrip — inline category strip rendered as a single line of
//! middot-separated text rather than chips/pills.
//!
//! `topics: all · rust · leptos · surrealdb · …` with the selected tag
//! showing accent color + underline. Pattern borrowed from
//! text.blogosphere.app (see docs/project-brief.md §4 reference site
//! research and docs/specification.md §5.5).

use leptos::{
    ev,
    html::{button, div, span},
    prelude::*,
};

const ACTIVE_CLASS: &str = "text-accent border-b border-accent transition-colors cursor-pointer";
const IDLE_CLASS: &str = "text-ink-3 hover:text-accent border-b border-transparent hover:border-accent transition-colors cursor-pointer";

/// Render the inline category strip.
///
/// `tags` is the list of `(tag, count)` pairs. `selected` is the reactive
/// signal holding the user's current tag selection (empty = "all").
pub fn component(tags: Vec<(String, u32)>, selected: RwSignal<Vec<String>>) -> impl IntoView {
    div()
        .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 flex flex-wrap items-center gap-x-2 gap-y-1")
        .child((
            span().class("text-ink-4").child("topics:"),
            // "all" reset button — class swaps reactively based on whether
            // any tag is selected.
            button()
                .on(ev::click, move |_| selected.update(Vec::clear))
                .class(ACTIVE_CLASS)
                .class((IDLE_CLASS, move || !selected.get().is_empty()))
                .child("all"),
            // each tag rendered as button + middot separator
            tags.into_iter()
                .map(move |(tag, _count)| {
                    let tag_for_click = tag.clone();
                    let tag_for_active = tag.clone();
                    let tag_for_idle = tag.clone();
                    let tag_for_render = tag;
                    (
                        span().class("text-ink-4").child("·"),
                        button()
                            .on(ev::click, move |_| {
                                let t = tag_for_click.clone();
                                selected.update(|prev| {
                                    if prev.contains(&t) {
                                        prev.retain(|v| v != &t);
                                    } else {
                                        prev.push(t);
                                    }
                                });
                            })
                            .class(IDLE_CLASS)
                            .class((
                                ACTIVE_CLASS,
                                move || selected.get().contains(&tag_for_active),
                            ))
                            .class((
                                "",
                                move || !selected.get().contains(&tag_for_idle),
                            ))
                            .child(tag_for_render),
                    )
                })
                .collect::<Vec<_>>(),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_strip_structure() {
        let _: fn(Vec<(String, u32)>, RwSignal<Vec<String>>) -> _ = component;
    }

    #[test]
    fn test_tag_strip_class_constants_distinct() {
        assert_ne!(ACTIVE_CLASS, IDLE_CLASS);
        assert!(ACTIVE_CLASS.contains("text-accent"));
        assert!(IDLE_CLASS.contains("hover:text-accent"));
    }
}
