//! DateStamp — newspaper-stamp tile with mono kicker, serif day numeral,
//! and 2px accent left rail.
//!
//! Three sizes: Featured (48px day, used for the homepage hero post),
//! Default (32px day, used for post-list rows and archive), and Compact
//! (24px day, used in the home notes strip).
//!
//! Pattern borrowed from blog.fsck.com (see docs/project-brief.md §4
//! reference site research and docs/specification.md §5.3).

use leptos::{
    html::{div, span},
    prelude::*,
};

/// Size variant for the date-stamp tile. Affects the day numeral size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateStampSize {
    /// Largest variant — for the home page featured post.
    Featured,
    /// Default — for post-list rows on home and archive.
    Default,
    /// Compact — for tight spaces. Spec'd for the home notes strip but
    /// currently the notes strip uses inline rows; kept for the explicit
    /// spec contract and future use.
    #[allow(dead_code)]
    Compact,
}

impl DateStampSize {
    fn day_class(self) -> &'static str {
        match self {
            DateStampSize::Featured => {
                "font-display text-5xl font-medium leading-none tracking-tight text-ink"
            }
            DateStampSize::Default => {
                "font-display text-3xl font-medium leading-none tracking-tight text-ink"
            }
            DateStampSize::Compact => {
                "font-display text-2xl font-medium leading-none tracking-tight text-ink"
            }
        }
    }

    fn kicker_class(self) -> &'static str {
        match self {
            DateStampSize::Featured => "font-mono text-xs uppercase tracking-[0.08em] text-ink-3",
            _ => "font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3",
        }
    }
}

/// Render a date-stamp tile.
///
/// `kicker` is the mono uppercase prefix (e.g. "MOST RECENT", "APR" for
/// month-only stamps). `day` is the day numeral. `year_or_meta` is small
/// secondary text below the day (e.g. "2026" or a relative time).
pub fn component(
    kicker: String,
    day: String,
    year_or_meta: String,
    size: DateStampSize,
) -> impl IntoView {
    div()
        .class("border-l-2 border-accent pl-4 flex flex-col gap-1 select-none")
        .child((
            span().class(size.kicker_class()).child(kicker),
            span().class(size.day_class()).child(day),
            span()
                .class("font-mono text-[10px] uppercase tracking-[0.08em] text-ink-4")
                .child(year_or_meta),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_stamp_structure() {
        let _: fn(String, String, String, DateStampSize) -> _ = component;
    }

    #[test]
    fn test_date_stamp_size_variants_distinct() {
        assert_ne!(DateStampSize::Featured, DateStampSize::Default);
        assert_ne!(DateStampSize::Default, DateStampSize::Compact);
        assert_ne!(DateStampSize::Featured, DateStampSize::Compact);
    }

    #[test]
    fn test_date_stamp_size_classes_distinct() {
        assert_ne!(
            DateStampSize::Featured.day_class(),
            DateStampSize::Default.day_class()
        );
    }
}
