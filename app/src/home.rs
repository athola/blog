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

#[expect(clippy::too_many_lines)]
pub fn component() -> impl IntoView {
    let selected_tags = RwSignal::new(Vec::<String>::new());
    let tags = Resource::new_blocking(
        || (),
        move |()| async move { select_tags().await.unwrap_or_default() },
    );
    #[expect(clippy::all)]
    let posts = Resource::new(
        move || return selected_tags.get(),
        move |selected_tags| return async move { return select_posts(selected_tags).await },
    );

    div().child((
        Title(
            TitleProps::builder()
                .text("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                .build(),
        ),
        Suspense(
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
        Suspense(SuspenseProps::builder().fallback(|| loader::component).children(TypedChildren::to_children(move || {
            div().class("flex flex-row flex-wrap gap-1 px-4 text-xs").child((
                button().on(ev::click, move |_| selected_tags.update(Vec::clear))
                    .class("py-1 px-2 text-white rounded-lg transition-all duration-500 cursor-pointer bg-primary")
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
