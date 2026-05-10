//! Archive page — full chronological list of posts, year-grouped, with
//! optional `?tag=<slug>` filter.
//!
//! Layout (spec §4.3):
//!   1. Page title in italic display ("writing")
//!   2. Total-count mono kicker
//!   3. Tag filter strip (URL-driven via ?tag=foo)
//!   4. Year-grouped post list — each year header is a `<h2>` anchor
//!
//! Pagination is not yet implemented at this scale; the full list renders
//! in a single SSR pass. Add cursor pagination once the corpus exceeds
//! ~200 posts.

use leptos::{
    html::{div, h1, h2, p, section, span},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};
use leptos_router::hooks::use_query_map;

use crate::api::select_posts;
use crate::components::post_list_row::{self, PostListSize};
use crate::types::Post;

/// Renders the /archive route.
pub fn component() -> impl IntoView {
    // Read ?tag=foo from the URL once per route render
    let query = use_query_map();
    let initial_tag = query
        .with_untracked(|q| q.get("tag").map(|s| s.to_string()))
        .unwrap_or_default();

    let initial_tags: Vec<String> = if initial_tag.is_empty() {
        Vec::new()
    } else {
        vec![initial_tag.clone()]
    };

    let posts = Resource::new_blocking(
        move || initial_tags.clone(),
        move |tags| async move { select_posts(tags).await.unwrap_or_default() },
    );

    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text(if initial_tag.is_empty() {
                    "Writing — Alex Thola".to_string()
                } else {
                    format!("Writing tagged #{} — Alex Thola", initial_tag)
                })
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("writing"),
            Suspense(
                SuspenseProps::builder()
                    .fallback(|| ())
                    .children(TypedChildren::to_children(move || {
                        move || {
                            let count = posts.get().map(|v| v.len()).unwrap_or(0);
                            p()
                                .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                                .child(if initial_tag.is_empty() {
                                    format!("{} posts in the archive", count)
                                } else {
                                    format!("{} posts tagged #{}", count, initial_tag)
                                })
                        }
                    }))
                    .build(),
            ),
        )),
        // Year-grouped post list
        section().attr("aria-label", "archive").class("flex flex-col").child(
            Suspense(
                SuspenseProps::builder()
                    .fallback(|| ())
                    .children(TypedChildren::to_children(move || {
                        move || {
                            let post_list = posts.get().unwrap_or_default();
                            if post_list.is_empty() {
                                return div()
                                    .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 py-12 text-center")
                                    .child("No posts in this slice of the archive.")
                                    .into_any();
                            }
                            div().class("flex flex-col gap-12").child(
                                group_by_year(post_list)
                                    .into_iter()
                                    .map(|(year, year_posts)| render_year_group(year, year_posts))
                                    .collect::<Vec<_>>(),
                            ).into_any()
                        }
                    }))
                    .build(),
            ),
        ),
    ))
}

fn render_year_group(year: String, posts: Vec<Post>) -> impl IntoView {
    let total = posts.len();
    section().attr("id", year.clone()).class("flex flex-col").child((
        // Year header — italic display, anchor for in-page links
        h2()
            .class("font-display italic text-3xl sm:text-4xl font-medium text-ink mb-4 flex items-baseline gap-3")
            .child((
                year.clone(),
                span()
                    .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4 not-italic")
                    .child(format!("{} posts", total)),
            )),
        div().class("flex flex-col").child(
            posts
                .into_iter()
                .enumerate()
                .map(|(i, post)| {
                    post_list_row::component(
                        post,
                        PostListSize::Default,
                        i < total.saturating_sub(1),
                    )
                })
                .collect::<Vec<_>>(),
        ),
    ))
}

/// Group posts by their year (extracted from `created_at`). Returns a
/// vector of (year, posts) ordered newest year first. Posts within each
/// year retain the order received from the API (already DESC by date).
fn group_by_year(posts: Vec<Post>) -> Vec<(String, Vec<Post>)> {
    let mut groups: Vec<(String, Vec<Post>)> = Vec::new();
    for post in posts {
        let year = post.created_at.split('-').next().unwrap_or("").to_string();
        let year = if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
            year
        } else {
            "Undated".to_string()
        };
        match groups.iter_mut().find(|(y, _)| y == &year) {
            Some((_, list)) => list.push(post),
            None => groups.push((year, vec![post])),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Author, Post};
    use surrealdb::sql::Thing;

    fn mk_post(slug: &str, year: i32, month: u8, day: u8) -> Post {
        Post {
            id: Thing::from(("post", slug)),
            slug: Some(slug.to_string()),
            title: format!("Post {}", slug),
            summary: "summary".into(),
            body: "body".into(),
            tags: vec!["rust".into()],
            author: Author::default(),
            read_time: 1,
            total_views: 0,
            created_at: format!("{year:04}-{month:02}-{day:02}"),
            updated_at: format!("{year:04}-{month:02}-{day:02}"),
            is_published: true,
            header_image: None,
        }
    }

    #[test]
    fn test_archive_component_signature() {
        let _: fn() -> _ = component;
    }

    #[test]
    fn test_group_by_year_separates() {
        let posts = vec![
            mk_post("a", 2026, 4, 1),
            mk_post("b", 2026, 1, 1),
            mk_post("c", 2025, 12, 31),
            mk_post("d", 2024, 6, 15),
        ];
        let groups = group_by_year(posts);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].0, "2026");
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, "2025");
        assert_eq!(groups[2].0, "2024");
    }

    #[test]
    fn test_group_by_year_undated_fallback() {
        let mut p = mk_post("e", 2024, 1, 1);
        p.created_at = "garbage".to_string();
        let groups = group_by_year(vec![p]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "Undated");
    }

    #[test]
    fn test_group_by_year_empty() {
        let groups = group_by_year(vec![]);
        assert!(groups.is_empty());
    }
}
