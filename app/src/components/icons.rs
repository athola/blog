use leptos::{
    html::{a, div, span},
    prelude::*,
};

pub fn component() -> impl IntoView {
    div().class("flex flex-row gap-3 items-center h-10").child((
        a().href("https://github.com/athola/")
            .rel("noopener noreferrer")
            .target("_blank")
            .aria_label("GitHub")
            .class("transition-all text-white duration-500 size-6 hover:text-[#ffef5c]")
            .child(
                span().class("text-white size-6").child("GH"), // GitHub text
            ),
        a().href("https://x.com/alexthola")
            .rel("noopener noreferrer")
            .target("_blank")
            .aria_label("X")
            .class("transition-all text-white duration-500 size-6 hover:text-[#ffef5c]")
            .child(
                span().class("text-white size-6").child("𝕏"), // Twitter X symbol
            ),
        a().href("https://www.linkedin.com/in/alexthola")
            .rel("noopener noreferrer")
            .target("_blank")
            .aria_label("LinkedIn")
            .class("transition-all text-white duration-500 size-6 hover:text-[#ffef5c]")
            .child(
                span().class("text-white size-6").child("in"), // LinkedIn text
            ),
        a().href("/rss.xml")
            .rel("noopener noreferrer")
            .target("_blank")
            .aria_label("rss")
            .class("transition-all text-white duration-500 size-6 hover:text-[#ffef5c]")
            .child(
                span().class("text-white size-6").child("RSS"), // RSS text
            ),
    ))
}
