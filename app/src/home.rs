//! This module defines the `home` component, which serves as the application's
//! homepage.
//!
//! It displays a paginated list of blog posts and provides a tag-based filtering
//! mechanism. The component fetches post data and available tags from the API,
//! managing state for selected tags and dynamically updating the displayed posts.

extern crate alloc;
use alloc::vec::Vec;
use core::clone::Clone;
use icondata::{BsCalendar, BsClock, BsEye, FiUser};
use leptos::{
    ev,
    html::{button, div, p, span},
    prelude::*,
    svg::svg,
};

use leptos_meta::{Title, TitleProps};
use leptos_router::components::{A, AProps};

use crate::{
    api::{select_posts, select_tags},
    components::loader,
};

/// Renders the homepage of the blog, displaying a list of posts and a tag filter.
///
/// This component manages the following reactive state:
/// - `selected_tags`: A `RwSignal` holding the list of tags currently selected by the user.
/// - `tags`: A `Resource` that fetches all available tags from the server once.
/// - `posts`: A `Resource` that fetches posts based on the `selected_tags` signal,
///   re-fetching whenever the selected tags change.
///
/// The component uses `Suspense` for loading states and `For` for efficiently rendering
/// lists of posts and tags.
#[expect(clippy::too_many_lines)] // This function is necessarily large due to Leptos view! macro expansion.
pub fn component() -> impl IntoView {
    let selected_tags = RwSignal::new(Vec::<String>::new());

    // Resource to fetch all available tags once.
    let tags = Resource::new_blocking(
        || (), // The source is unit, meaning it runs once.
        move |()| async move { select_tags().await.unwrap_or_default() },
    );

    // Resource to fetch posts based on currently selected tags.
    // This resource will re-run whenever `selected_tags` changes.
    let posts = Resource::new(
        move || selected_tags.get(),
        move |selected_tags| async move { select_posts(selected_tags).await },
    );

    div().child((
        Title(
            TitleProps::builder()
                .text("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                .build(),
        ),
        Suspense(
            // Fallback for post loading. An empty fallback is used here as the posts are
            // rendered within the main content area, and no specific loader is placed directly here.
            SuspenseProps::builder().fallback(|| ()).children(TypedChildren::to_children(move || {
                div()
                    .class("gap-4 columns-1 sm:columns-2")
                    .child(For(ForProps::builder()
                        .each(move || posts.get().and_then(Result::ok).unwrap_or_default())
                        .key(|post| format!("{:?}", post.id))
                        .children(|post| {
                            div().class("flex flex-col p-3 text-left text-white rounded-lg transition-all duration-500 cursor-pointer break-inside-avoid bg-card hover:text-[#ffef5c]").child(
                                    A(AProps::builder()
                                        .href(format!("/post/{}", post.slug.as_ref().map_or("", |v| v)))
                                        .children(ToChildren::to_children(move || {
                                            div()
                                                .child(
                                                (div().class("flex flex-col gap-1 mb-4 font-medium").child((
                                                p().class("text-base line-clamp-2").child(post.title.clone()),
                                                p().class("italic text-xxs").child(post.summary.clone()),
                                            )),
                                            div().class("flex flex-row gap-3 justify-start items-center text-xxs").child(
                                                div().class("flex flex-row gap-3").child((
                                                    div().class("flex flex-row gap-1 items-center").child((
                                                        svg().attr("viewBox", BsClock.view_box).attr("innerHTML", BsClock.data).attr("style", "filter: brightness(0) invert(1);").class("size-4"),
                                                        p().child(format!("{} min read", post.read_time)),
                                                    )),
                                                    div().class("flex flex-row gap-1 items-center").child((
                                                        svg().attr("viewBox", BsEye.view_box).attr("innerHTML", BsEye.data).attr("style", "filter: brightness(0) invert(1);").class("size-4"),
                                                        p().child(format!("{} views", post.total_views)),
                                                    )),
                                                    div().class("flex flex-row gap-1 items-center").child((
                                                        svg().attr("viewBox", BsCalendar.view_box).attr("innerHTML", BsCalendar.data).attr("style", "filter: brightness(0) invert(1);").class("size-4"),
                                                        p().child(post.created_at),
                                                    )),
                                                    div().class("flex flex-row gap-1 items-center").child((
                                                        svg().attr("viewBox", FiUser.view_box).attr("innerHTML", FiUser.data).attr("style", "filter: brightness(0) invert(1);").class("size-4"),
                                                        button().on(ev::click, move |e| {
                                                            e.prevent_default();
                                                            e.stop_propagation();
                                                            let _ = window().open_with_url_and_target(
                                                                post.author.github.as_ref().unwrap_or(&String::new()),
                                                                "_blank",
                                                            );
                                                        }).child(
                                                            span().class("text-xs font-semibold cursor-pointer hover:underline").child(post.author.name),
                                                        ),
                                                    )),
                                                )),
                                            )
                                        ))}))
                            .build())
                        )})
                    .build()))
                })
            ).build(),
        ),
        Suspense(SuspenseProps::builder()
            // Display a loader component while tags are being fetched.
            .fallback(|| loader::component)
            .children(TypedChildren::to_children(move || {
                div().class("flex flex-row flex-wrap gap-1 px-4 text-xs").child((
                    button().on(ev::click, move |_| selected_tags.update(Vec::clear))
                        .class("py-1 px-2 text-white rounded-lg transition-all duration-500 cursor-pointer bg-primary")
                        // Underline the "All" button if no tags are selected.
                        .class(("underline", move || selected_tags.get().is_empty()))
                        .child("All"),
                    For(ForProps::builder()
                        .each(move || tags.get().unwrap_or_default())
                        .key(Clone::clone)
                        .children(move |(tag, count)| {
                            button().on(ev::click, {
                                let tag = tag.clone();
                                move |_| {
                                    selected_tags.update(|prev| {
                                        // Toggle tag selection: add if not present, remove if present.
                                        if prev.contains(&tag) {
                                            *prev = prev.clone().into_iter().filter(|v| v != &tag).collect::<Vec<_>>();
                                        } else {
                                            *prev = prev.clone().into_iter().chain(core::iter::once(tag.clone())).collect();
                                        }
                                    });
                                }
                            })
                            .class("py-1 px-2 rounded-lg transition-all duration-500 cursor-pointer hover:text-black hover:bg-white")
                            .class((
                                "bg-white",
                                {
                                    let tag = tag.clone();
                                    move || selected_tags.get().contains(&tag)
                                }),
                            )
                            .class((
                                "text-black",
                                {
                                    let tag = tag.clone();
                                    move || selected_tags.get().contains(&tag)
                                }),
                            )
                            .child(tag + " (" + &count.to_string() + ")")
                        })
                        .build())
                ))
            })).build())
    ))
}
