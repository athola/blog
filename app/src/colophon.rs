//! Colophon — site stack, fonts, source, license.
//!
//! Layout (spec §4.8):
//!   1. Page title in italic display ("colophon")
//!   2. Stack section
//!   3. Fonts section (each font linked to its source)
//!   4. Source repo link
//!   5. License

use leptos::{
    html::{a, div, h1, h2, p, section, ul},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};

/// Renders the /colophon route.
pub fn component() -> impl IntoView {
    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("Colophon — Alex Thola")
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("colophon"),
            p().class("text-ink-2 leading-relaxed max-w-prose")
                .child("How this site is built. The stack is intentionally small — Rust on both ends, one variable font per role, no analytics, no client-side framework beyond Leptos hydration."),
        )),
        // Stack section
        section().class("flex flex-col gap-3 max-w-prose").child((
            h2().class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3").child("stack"),
            ul().class("flex flex-col gap-2 text-ink leading-relaxed list-none pl-0").child((
                stack_row("Leptos", "Rust full-stack framework with WASM hydration", "https://leptos.dev"),
                stack_row("Axum", "HTTP server backing the SSR routes and API", "https://github.com/tokio-rs/axum"),
                stack_row("SurrealDB", "Embedded-then-remote database for posts, tags, activities", "https://surrealdb.com"),
                stack_row("Tailwind v4", "CSS-first design tokens via @theme block", "https://tailwindcss.com"),
                stack_row("cargo-leptos", "Build orchestration for the WASM + server bundle", "https://github.com/leptos-rs/cargo-leptos"),
                stack_row("Caddy", "Reverse proxy fronting SurrealDB on the droplet", "https://caddyserver.com"),
                stack_row("DigitalOcean App Platform", "Deployment target for the SSR Rust app", "https://www.digitalocean.com/products/app-platform"),
            )),
        )),
        // Fonts
        section().class("flex flex-col gap-3 max-w-prose").child((
            h2().class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3").child("fonts"),
            ul().class("flex flex-col gap-2 text-ink leading-relaxed list-none pl-0").child((
                stack_row("Fraunces", "Display serif — used for h1, h2, and the italic-accent nameplate", "https://fonts.google.com/specimen/Fraunces"),
                stack_row("Inter", "Body sans — variable, set to a custom 470 weight", "https://fonts.google.com/specimen/Inter"),
                stack_row("JetBrains Mono", "Monospace — used for code, post metadata, and footer", "https://fonts.google.com/specimen/JetBrains+Mono"),
            )),
        )),
        // Source + license
        section().class("flex flex-col gap-2 max-w-prose pt-6 border-t border-rule-soft").child((
            p().class("text-ink").child((
                "Source: ",
                a().href("https://github.com/athola/blog")
                    .class("text-ink hover:text-accent border-b border-accent transition-colors")
                    .child("github.com/athola/blog"),
            )),
            p().class("text-ink-2 text-sm").child("Licensed under AGPL-3.0."),
        )),
    ))
}

fn stack_row(name: &'static str, description: &'static str, href: &'static str) -> impl IntoView {
    leptos::html::li().child((
        a().href(href)
            .class(
                "font-medium text-ink hover:text-accent border-b border-accent transition-colors",
            )
            .child(name),
        " — ",
        leptos::html::span().class("text-ink-2").child(description),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colophon_component_signature() {
        let _: fn() -> _ = component;
    }
}
