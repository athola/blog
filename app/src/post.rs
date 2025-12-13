//! This module defines the `post` component, which is responsible for rendering
//! individual blog post pages.
//!
//! It fetches post data based on the URL slug, increments the post's view count,
//! and displays the post's content along with its metadata (author, date, read time).
//! It also supports displaying a header image if available.

use leptos::html::{article, div, img, p, span};
use leptos::{ev, prelude::*};
use leptos_meta::{Title, TitleProps};
use leptos_router::hooks::use_params_map;

use crate::{
    api::{increment_views, select_post},
    components::loader,
    types::Post,
};

/// Renders an individual blog post page.
///
/// This component:
/// 1. Extracts the post `slug` from URL parameters.
/// 2. Fetches the `post` data using a `Resource` based on the `slug`.
/// 3. Dispatches an `Action` to `increment_views` for the post (on the server-side,
///    and only in non-debug builds to avoid skewing counts during development).
/// 4. Displays the post's title, metadata (author, date, read time, views),
///    an optional header image, and the HTML-rendered body content.
/// 5. Uses `Suspense` to show a loader while post data is being fetched.
pub fn component() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|params| params.get("slug").unwrap_or_default());

    // Resource to fetch post data based on the URL slug.
    let post = Resource::new_blocking(
        || (), // Runs once on initial load.
        move |()| async move { select_post(slug()).await.unwrap() },
    );

    // Action to increment the post's view count.
    let _increment_view = Action::new(move |id: &String| {
        let id = id.clone();
        async move {
            let _ = increment_views(id.to_string()).await;
        }
    });

    // Effect to dispatch the view increment action once the post is loaded,
    // only in non-debug builds to prevent inflated view counts during development.
    Effect::new(move |_| {
        #[cfg(not(debug_assertions))]
        if post.get().is_some() {
            _increment_view.dispatch(format!("{:?}", post.get().as_ref().unwrap().id));
        }
    });

    // Closure to render the post's content once it's available.
    let children = move |post: Post| {
        div().class("flex flex-col items-center font-mono").child(
        (
            // Conditionally display the header image if available.
            Show(ShowProps::builder()
                   .when({
                       let header_image = post.header_image.clone();
                       move || {
                           header_image.is_some()
                       }
                   })
                   .fallback(|| ()) // No fallback content if image is not present.
                   .children(ToChildren::to_children(
                       move ||
                         img()
                           .alt("") // Alt text could be improved with actual post title.
                           .class("object-contain self-center mb-6 h-full rounded-lg w-fit aspect-auto")
                           .src(post.header_image.clone().unwrap()))
                   ).build(),
            ),
            Title(TitleProps::builder().text(post.title.to_string()).build()),
            article().child((
                div().class("flex flex-col gap-4 mx-auto max-w-3xl").child((
                    p().class("text-4xl font-semibold").child(post.title.clone()),
                    div().class("flex gap-3 justify-start items-center text-sm text-muted-foreground").child((
                        p().on(ev::click, move |e| {
                            // Prevent default link behavior and event propagation
                            // to handle navigation programmatically if author GitHub exists.
                            e.stop_propagation();
                            if let Some(github) = &post.author.github {
                                let _ = window().open_with_url_and_target(github, "_blank");
                            }
                        }).class("cursor-pointer hover:underline").child(("by ", span().class("ml-1 font-semibold").child(post.author.name.to_string()))),
                        p().child(post.created_at),
                        p().child(format!("{} min read", post.read_time)),
                        p().child(format!("{} views", post.total_views)),
                    )),
                )),
                // TailwindCSS `prose` classes apply typography styles to the raw HTML content.
                // Customizations are applied to ensure dark mode compatibility and specific styling.
                div().class("my-6 mx-auto max-w-3xl prose prose-h3:text-white prose-h4:text-white prose-code:before:content-none prose-th:text-white prose-li:marker:text-white prose-code:after:content-none prose-pre:bg-transparent prose-pre:rounded-lg prose-pre:p-0 prose-code:text-[#ffef5c] prose-strong:text-white prose-table:text-white prose-thead:text-white prose-li:text-white prose-ol:text-white prose-h1:text-white prose-h1:text-3xl prose-h2:text-white prose-h2:text-2xl prose-ul:text-white prose-p:text-white prose-a:text-[#ffef5c]")
                  .inner_html(post.body.to_string())
            )),
        ))
    };

    Suspense(
        SuspenseProps::builder()
            .fallback(loader::component) // Show a loader component while the post is being fetched.
            .children(TypedChildren::to_children(move || {
                move || {
                    post.with(|post| {
                        let post = post.clone().unwrap_or_default();
                        children(post)
                    })
                }
            }))
            .build(),
    )
}
