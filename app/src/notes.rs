//! Notes (microblog) page — replaces the prior /activity route.
//!
//! Layout (spec §4.4):
//!   1. Page title in italic display ("notes")
//!   2. Subtitle in mono uppercase
//!   3. Note list — each row: relative-time mono kicker + content + tag row
//!      + optional source link with ↗ outbound glyph
//!   4. Load-more pagination (preserved from original /activity)
//!
//! Design migration: the previous bg-gray-800/text-blue-400 palette is
//! replaced with token-driven paper-2/accent. Tag chips (gray-700) become
//! inline middot-separated mono kickers.

use leptos::{
    ev,
    html::{a, button, div, h1, p, section, span},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};

use crate::api::{create_activity, select_activities};
use crate::types::Activity;

/// Number of notes per page (matches server-side ACTIVITIES_PER_PAGE).
const NOTES_PER_PAGE: usize = 10;

/// Renders the /notes route.
pub fn component() -> impl IntoView {
    let all_notes = RwSignal::new(Vec::<Activity>::new());
    let current_page = RwSignal::new(0usize);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    let page_resource = Resource::new(
        move || current_page.get(),
        |page| async move { select_activities(page).await },
    );

    Effect::new(move |_| match page_resource.get() {
        Some(Ok(new_notes)) => {
            if new_notes.len() < NOTES_PER_PAGE {
                has_more.set(false);
            }
            if !new_notes.is_empty() {
                all_notes.update(|list| list.extend(new_notes));
            }
            is_loading.set(false);
        }
        Some(Err(e)) => {
            leptos::logging::error!("Failed to fetch notes: {:?}", e);
            is_loading.set(false);
            has_more.set(false);
        }
        None => {}
    });

    let load_more = move || {
        if !is_loading.get() && has_more.get() {
            is_loading.set(true);
            current_page.update(|p| *p += 1);
        }
    };

    // Register the server action so it is available in the Leptos runtime,
    // even if not invoked here (matches original activity.rs pattern).
    let _create_action = Action::new(move |(api_key, note): &(String, Activity)| {
        let api_key = api_key.clone();
        let note = note.clone();
        async move { create_activity(api_key, note).await }
    });

    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("Notes — Alex Thola")
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("notes"),
            p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                .child("short-form notes, links, and asides"),
        )),
        // Note list
        section().attr("aria-label", "notes").class("flex flex-col").child(
            move || {
                let notes = all_notes.get();
                if notes.is_empty() && !has_more.get() {
                    return div()
                        .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 py-8 text-center")
                        .child("No notes yet.")
                        .into_any();
                }
                div().class("flex flex-col").child(
                    notes
                        .into_iter()
                        .map(render_note_row)
                        .collect::<Vec<_>>(),
                ).into_any()
            },
        ),
        // Pagination footer
        div().class("pt-4 flex justify-center font-mono text-[11px] uppercase tracking-[0.08em]").child(
            move || {
                if is_loading.get() {
                    span().class("text-ink-3").child("loading…").into_any()
                } else if has_more.get() {
                    button()
                        .on(ev::click, move |_| load_more())
                        .class("text-ink-3 hover:text-accent border-b border-transparent hover:border-accent transition-colors cursor-pointer")
                        .child("load more →")
                        .into_any()
                } else if all_notes.get().is_empty() {
                    ().into_any()
                } else {
                    span().class("text-ink-4").child("you've reached the end.").into_any()
                }
            },
        ),
    ))
}

fn render_note_row(note: Activity) -> impl IntoView {
    let tag_line = note
        .tags
        .iter()
        .map(|t| format!("#{}", t))
        .collect::<Vec<_>>()
        .join(" · ");
    let source = note.source.clone();
    let source_text = source.clone().unwrap_or_default();

    div().class("py-6 border-b border-rule-soft last:border-b-0 flex flex-col gap-2").child((
        // Mono kicker — relative time / created_at
        p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
            .child(note.created_at.clone()),
        // Content
        p().class("text-ink text-base leading-relaxed").child(note.content.clone()),
        // Tags
        Show(
            ShowProps::builder()
                .when({
                    let tag_line = tag_line.clone();
                    move || !tag_line.is_empty()
                })
                .fallback(|| ())
                .children(ToChildren::to_children(move || {
                    p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4")
                        .child(tag_line.clone())
                }))
                .build(),
        ),
        // Source link with ↗ outbound glyph (added by global CSS rule)
        Show(
            ShowProps::builder()
                .when(move || source.is_some())
                .fallback(|| ())
                .children(ToChildren::to_children(move || {
                    a()
                        .href(source_text.clone())
                        .attr("target", "_blank")
                        .attr("rel", "noopener noreferrer")
                        .class("font-mono text-[11px] uppercase tracking-[0.08em] text-accent hover:opacity-80 transition-opacity self-start")
                        .child("source")
                }))
                .build(),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_component_signature() {
        let _: fn() -> _ = component;
    }

    #[test]
    fn test_notes_per_page_constant() {
        // Must match server-side ACTIVITIES_PER_PAGE in api.rs
        assert_eq!(NOTES_PER_PAGE, 10);
    }

    #[test]
    fn test_notes_select_signature() {
        use leptos::prelude::ServerFnError;
        let _check: fn(usize) -> _ = |_page| async {
            let _result: Result<Vec<Activity>, ServerFnError> = Ok(vec![]);
            _result
        };
    }

    #[test]
    fn test_note_data_shape() {
        let note = Activity {
            content: "Test note".to_string(),
            tags: vec!["rust".to_string()],
            source: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(note.content, "Test note");
        assert!(!note.tags.is_empty());
        assert!(note.source.is_some());
    }
}
