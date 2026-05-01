//! About page — author bio, role, links, and colophon link.
//!
//! Layout (spec §4.6):
//!   1. Page title in italic display ("about")
//!   2. Author block: avatar + name + role + italic one-line bio
//!   3. Long-form bio paragraphs
//!   4. Links row (GitHub, LinkedIn, X, RSS)
//!   5. Colophon link

use leptos::{
    html::{a, div, h1, img, p, section, span},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};
use leptos_router::components::{A, AProps};

const AVATAR_URL: &str = "https://avatars.githubusercontent.com/u/9769290?s=400&u=84fd5ddd993912372430ed97b768bf202827989b&v=4";

/// Renders the /about route.
pub fn component() -> impl IntoView {
    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("About — Alex Thola")
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("about"),
        )),
        // Author chip — avatar + name + role + italic one-line bio
        section().class("flex flex-col sm:flex-row gap-6 items-start").child((
            img()
                .src(AVATAR_URL)
                .alt("Alex Thola")
                .attr("width", "96")
                .attr("height", "96")
                .attr("loading", "lazy")
                .class("rounded-full border border-rule-soft flex-shrink-0"),
            div().class("flex flex-col gap-2 max-w-prose").child((
                span()
                    .class("font-display text-2xl sm:text-3xl font-medium text-ink leading-tight")
                    .child("Alex Thola"),
                p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                    .child("staff software engineer · rust consultant"),
                p().class("text-ink-2 italic leading-relaxed")
                    .child("I design scalable architectures built for performance and reliability — and I write about it here."),
            )),
        )),
        // Long-form bio
        section().class("flex flex-col gap-4 max-w-prose").child((
            p().class("text-ink leading-[1.7]")
                .child("I'm Alex Thola — a staff software engineer focused on Rust, distributed systems, and observability. I enjoy solving network and OS-level problems with high-performance solutions, and I consult on Rust architecture, runtime tuning, and engineering productivity for teams shipping systems software."),
            p().class("text-ink leading-[1.7]")
                .child("This blog is a long-running notebook of what I've learned, what I'm working on, and what I think holds up after a few years of practice. Posts trend toward Rust internals, async runtimes, and the engineering ergonomics that make small teams effective."),
        )),
        // Links row
        section().class("flex flex-col gap-3").child((
            p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                .child("find me elsewhere"),
            div().class("flex flex-wrap gap-x-6 gap-y-2 text-base").child((
                a().href("https://github.com/athola").class("text-ink hover:text-accent border-b border-accent transition-colors").child("GitHub"),
                a().href("https://www.linkedin.com/in/alexthola").class("text-ink hover:text-accent border-b border-accent transition-colors").child("LinkedIn"),
                a().href("https://x.com/alexthola").class("text-ink hover:text-accent border-b border-accent transition-colors").child("X"),
                a().href("/feed/rss.xml").class("text-ink hover:text-accent border-b border-accent transition-colors").child("RSS"),
                a().href("/feed/feed.xml").class("text-ink hover:text-accent border-b border-accent transition-colors").child("Atom"),
            )),
        )),
        // Colophon link
        section().class("border-t border-rule-soft pt-6 max-w-prose").child(
            A(AProps::builder()
                .href("/colophon".to_string())
                .children(ToChildren::to_children(move || {
                    span()
                        .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3 hover:text-accent transition-colors")
                        .child("read about how this site is built →")
                }))
                .build()),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_component_signature() {
        let _: fn() -> _ = component;
    }

    #[test]
    fn test_avatar_url_is_https() {
        assert!(AVATAR_URL.starts_with("https://"));
    }
}
