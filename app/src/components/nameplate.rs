//! Nameplate — site title with italic-accent on the surname.
//!
//! Renders "alex *thola*" where the italicized "thola" carries `text-accent`
//! (burgundy in light, lightened-burgundy in dark). The whole element links
//! to the site root.
//!
//! Pattern borrowed from blog.fsck.com's italic-accented two-piece nameplate
//! (see docs/project-brief.md §5 Direction D).

use leptos::{
    html::{em, span},
    prelude::*,
};
use leptos_router::components::A;

/// Renders the italic-accented site nameplate. The whole nameplate is the
/// home link (`href="/"`).
pub fn component() -> impl IntoView {
    A(leptos_router::components::AProps::builder()
        .href("/")
        .children(ToChildren::to_children(move || {
            span()
                .class("font-display text-2xl sm:text-3xl tracking-tight font-medium text-ink hover:opacity-80 transition-opacity")
                .child((
                    "alex ",
                    em().class("italic text-accent")
                        .child("thola"),
                ))
        }))
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nameplate_component_structure() {
        let _: fn() -> _ = component;
    }
}
