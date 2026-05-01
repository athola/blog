//! References (portfolio) page.
//!
//! Layout (spec §4.5):
//!   1. Page title in italic display ("references")
//!   2. Subtitle paragraph
//!   3. Project list — single column rows (NOT cards); each row has
//!      title, description, and tech-stack as inline mono bars with
//!      ▰/▱ glyphs.
//!
//! Replaces the prior glassmorphism + grid-bg cards with editorial rows.

use leptos::{
    html::{div, h1, h2, p, section, span},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};

use crate::api::select_references;

/// Renders the references / portfolio page.
pub fn component() -> impl IntoView {
    let references = Resource::new_blocking(
        || (),
        move |()| async move { select_references().await.unwrap_or_default() },
    );

    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("References — Alex Thola")
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("references"),
            p().class("text-ink-2 text-base leading-relaxed max-w-prose")
                .child("A portfolio of selected projects. I enjoy solving network and OS problems with high-performance solutions, and consult on Rust architecture, observability, and runtime tuning."),
        )),
        // Projects list
        section()
            .attr("aria-label", "projects")
            .class("flex flex-col")
            .child(Suspense(
                SuspenseProps::builder()
                    .fallback(|| ())
                    .children(TypedChildren::to_children(move || {
                        move || {
                            let refs = references.get().unwrap_or_default();
                            if refs.is_empty() {
                                return div()
                                    .class("font-mono text-xs uppercase tracking-[0.08em] text-ink-3 py-8 text-center")
                                    .child("No projects yet.")
                                    .into_any();
                            }
                            let total = refs.len();
                            div().class("flex flex-col").child(
                                refs.into_iter()
                                    .enumerate()
                                    .map(|(i, r)| {
                                        let divider = i < total - 1;
                                        render_project_row(r, divider)
                                    })
                                    .collect::<Vec<_>>(),
                            ).into_any()
                        }
                    }))
                    .build(),
            )),
    ))
}

fn render_project_row(r: crate::types::Reference, divider: bool) -> impl IntoView {
    let wrapper_class = if divider {
        "py-8 border-b border-rule-soft"
    } else {
        "py-8"
    };

    let tech_zip: Vec<(String, u8)> = r
        .tech_stack
        .iter()
        .cloned()
        .zip(r.teck_stack_percentage.iter().cloned())
        .collect();

    div().class(wrapper_class).child((
        // Title
        h2()
            .class("font-display text-2xl sm:text-3xl italic font-medium leading-tight text-ink mb-3")
            .child(r.title.clone()),
        // Description
        p().class("text-ink-2 text-base leading-relaxed mb-5 max-w-prose")
            .child(r.description.clone()),
        // Tech stack — mono bars
        div().class("grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-2 font-mono text-[11px] uppercase tracking-[0.08em]")
            .child(
                tech_zip
                    .into_iter()
                    .map(|(name, pct)| {
                        let pct = pct.min(100);
                        let bars = pct_to_bars(pct);
                        div().class("flex items-center justify-between gap-3").child((
                            span().class("text-ink-2 truncate").child(name),
                            div().class("flex items-center gap-2").child((
                                span().class("text-accent leading-none").child(bars),
                                span().class("text-ink-4 tabular-nums").child(format!("{}%", pct)),
                            )),
                        ))
                    })
                    .collect::<Vec<_>>(),
            ),
    ))
}

/// Convert a 0..=100 percentage to a 10-cell mono bar `▰▰▰▰▰▱▱▱▱▱`.
fn pct_to_bars(pct: u8) -> String {
    let pct = pct.min(100) as usize;
    let filled = pct / 10;
    let empty = 10 - filled;
    let mut s = String::with_capacity(10);
    for _ in 0..filled {
        s.push('▰');
    }
    for _ in 0..empty {
        s.push('▱');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_references_component_signature() {
        let _: fn() -> _ = component;
    }

    #[test]
    fn test_pct_to_bars_zero() {
        assert_eq!(pct_to_bars(0), "▱▱▱▱▱▱▱▱▱▱");
    }

    #[test]
    fn test_pct_to_bars_full() {
        assert_eq!(pct_to_bars(100), "▰▰▰▰▰▰▰▰▰▰");
    }

    #[test]
    fn test_pct_to_bars_half() {
        assert_eq!(pct_to_bars(50), "▰▰▰▰▰▱▱▱▱▱");
    }

    #[test]
    fn test_pct_to_bars_clamps_overflow() {
        assert_eq!(pct_to_bars(120), "▰▰▰▰▰▰▰▰▰▰");
    }

    #[test]
    fn test_pct_to_bars_low() {
        assert_eq!(pct_to_bars(15), "▰▱▱▱▱▱▱▱▱▱"); // 15 / 10 = 1
    }
}
