//! Home page — the editorial entry point.
//!
//! Layout (spec §4.1):
//!   1. Featured post (DateStamp Featured + title + excerpt + meta)
//!   2. Recent posts list (PostListRow Default × 5-7)
//!   3. + archive → link
//!   4. Latest notes strip (3 most recent)
//!   5. + notes → link
//!   6. TagStrip filter (filters the recent posts list reactively)

extern crate alloc;
use alloc::vec::Vec;

use leptos::{
    html::{div, p, span},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};
use leptos_router::components::{A, AProps};

use crate::api::{select_activities, select_posts, select_tags};
use crate::components::{
    date_stamp::{self, DateStampSize},
    loader,
    post_list_row::{self, PostListSize},
    tag_strip,
};
use crate::types::Post;

/// Renders the home page.
#[expect(clippy::too_many_lines)]
pub fn component() -> impl IntoView {
    let selected_tags = RwSignal::new(Vec::<String>::new());

    // All available tags for the inline category strip
    let tags = Resource::new_blocking(
        || (),
        move |()| async move { select_tags().await.unwrap_or_default() },
    );

    // Posts filtered by selected tags — first item becomes Featured, rest are recent
    let posts = Resource::new(
        move || selected_tags.get(),
        move |selected_tags| async move { select_posts(selected_tags).await },
    );

    // Latest 3 notes (page 0 returns up to 10; we slice to 3)
    let notes = Resource::new_blocking(
        || (),
        move |()| async move { select_activities(0).await.unwrap_or_default() },
    );

    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("Alex Thola — Tech Insights & Consulting")
                .build(),
        ),
        // ─── Posts (featured + recent + archive link) ────────────────
        Suspense(
            SuspenseProps::builder()
                .fallback(loader::component)
                .children(TypedChildren::to_children(move || {
                    move || {
                        let post_list: Vec<Post> = posts
                            .get()
                            .and_then(Result::ok)
                            .unwrap_or_default();

                        if post_list.is_empty() {
                            return div().class(
                                "font-mono text-xs uppercase tracking-[0.08em] text-ink-3 py-12 text-center",
                            ).child("No posts match the selected tags.").into_any();
                        }

                        let featured = post_list[0].clone();
                        let recent: Vec<Post> = post_list[1..].iter().take(7).cloned().collect();
                        let recent_len = recent.len();

                        div().class("flex flex-col").child((
                            // Featured post — full bleed within reading column
                            div()
                                .class("border-b border-rule-soft pb-2 mb-2")
                                .child(post_list_row::component(
                                    featured,
                                    PostListSize::Featured,
                                    false,
                                )),
                            // Recent posts list
                            div().class("flex flex-col").child(
                                recent
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, post)| {
                                        post_list_row::component(
                                            post,
                                            PostListSize::Default,
                                            i < recent_len.saturating_sub(1),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                            // Archive link
                            div().class("flex justify-end mt-2").child(
                                A(AProps::builder()
                                    .href("/archive".to_string())
                                    .children(ToChildren::to_children(move || {
                                        span()
                                            .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 hover:text-accent transition-colors")
                                            .child("+ archive →")
                                    }))
                                    .build()),
                            ),
                        )).into_any()
                    }
                }))
                .build(),
        ),
        // ─── Latest notes strip ──────────────────────────────────────
        Suspense(
            SuspenseProps::builder()
                .fallback(|| ())
                .children(TypedChildren::to_children(move || {
                    move || {
                        let note_list = notes.get().unwrap_or_default();
                        if note_list.is_empty() {
                            return div().into_any();
                        }
                        let three: Vec<_> = note_list.into_iter().take(3).collect();
                        div().class("flex flex-col gap-3 pt-8 border-t-2 border-rule").child((
                            // Section kicker
                            p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                                .child("notes — recent"),
                            // Note rows
                            div().class("flex flex-col").child(
                                three
                                    .into_iter()
                                    .map(|note| {
                                        let tag = note.tags.first().cloned().unwrap_or_default();
                                        div()
                                            .class("py-3 border-b border-rule-soft last:border-b-0 flex flex-col gap-1")
                                            .child((
                                                p().class("text-ink-2 text-base line-clamp-2")
                                                    .child(note.content.clone()),
                                                p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4")
                                                    .child((
                                                        note.created_at,
                                                        if !tag.is_empty() {
                                                            Some(format!(" · #{}", tag))
                                                        } else {
                                                            None
                                                        },
                                                    )),
                                            ))
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                            // Notes link
                            div().class("flex justify-end").child(
                                A(AProps::builder()
                                    .href("/notes".to_string())
                                    .children(ToChildren::to_children(move || {
                                        span()
                                            .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 hover:text-accent transition-colors")
                                            .child("+ notes →")
                                    }))
                                    .build()),
                            ),
                        )).into_any()
                    }
                }))
                .build(),
        ),
        // ─── Tag filter strip ────────────────────────────────────────
        Suspense(
            SuspenseProps::builder()
                .fallback(|| ())
                .children(TypedChildren::to_children(move || {
                    move || {
                        let tag_list: Vec<(String, u32)> = tags
                            .get()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(k, v)| (k, v as u32))
                            .collect();
                        div()
                            .class("pt-6 border-t border-rule-soft")
                            .child(tag_strip::component(tag_list, selected_tags))
                    }
                }))
                .build(),
        ),
        // ─── Quick stamp at the very bottom (decorative) ─────────────
        div().class("flex justify-center pt-4").child(date_stamp::component(
            "ALEXTHOLA.COM".to_string(),
            "—".to_string(),
            "EST. 2024".to_string(),
            DateStampSize::Compact,
        )),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_component_signature() {
        let _: fn() -> _ = component;
    }
}
