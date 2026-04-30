//! PostListRow — one row in a post list. Combines DateStamp + title +
//! excerpt + meta with a hairline rule separator.
//!
//! Used on the home page (recent posts) and the archive page. Two sizes:
//! Featured (large title, full excerpt, used for top-of-home post) and
//! Default (smaller title, single-line excerpt).
//!
//! Pattern borrowed from blog.fsck.com (see docs/specification.md §5.4).

use crate::components::date_stamp::{self, DateStampSize};
use crate::types::Post;
use leptos::{
    html::{div, p, span},
    prelude::*,
};
use leptos_router::components::{A, AProps};

/// Size variant for the post row. Featured uses a larger title and excerpt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostListSize {
    /// Larger variant for top-of-home featured post.
    Featured,
    /// Default size for recent-posts and archive lists.
    Default,
}

impl PostListSize {
    fn title_class(self) -> &'static str {
        match self {
            PostListSize::Featured => {
                "font-display text-2xl sm:text-3xl italic font-medium leading-tight text-ink hover:text-accent transition-colors"
            }
            PostListSize::Default => {
                "font-display text-xl italic font-medium leading-tight text-ink hover:text-accent transition-colors"
            }
        }
    }

    fn excerpt_class(self) -> &'static str {
        match self {
            PostListSize::Featured => "text-ink-2 text-base line-clamp-2",
            PostListSize::Default => "text-ink-2 text-base line-clamp-1",
        }
    }

    fn stamp_size(self) -> DateStampSize {
        match self {
            PostListSize::Featured => DateStampSize::Featured,
            PostListSize::Default => DateStampSize::Default,
        }
    }
}

/// Render a single post row with date stamp, title, excerpt, and meta.
///
/// `divider` controls whether a hairline rule renders below the row (default
/// true; opt-out via `false` for the last row in a list).
pub fn component(post: Post, size: PostListSize, divider: bool) -> impl IntoView {
    // The wrapper always has a class — when `divider` is false, the class is
    // empty, keeping a uniform return type across branches.
    let wrapper_class = if divider {
        "border-b border-rule-soft"
    } else {
        ""
    };

    let created = post.created_at.clone();
    let (kicker, day, meta) = parse_date_pieces(&created, size == PostListSize::Featured);

    let slug = post.slug.clone().unwrap_or_default();
    let title = post.title.clone();
    let summary = post.summary.clone();
    let read_time = post.read_time;
    let primary_tag = post.tags.first().cloned();

    div().class(wrapper_class).child(
        div()
            .class("grid grid-cols-[110px_1fr] sm:grid-cols-[130px_1fr] gap-6 items-start py-6")
            .child((
                date_stamp::component(kicker, day, meta, size.stamp_size()),
                div().class("flex flex-col gap-2").child((
                    A(AProps::builder()
                        .href(format!("/post/{}", slug))
                        .children(ToChildren::to_children(move || {
                            span().class(size.title_class()).child(title)
                        }))
                        .build()),
                    p().class(size.excerpt_class()).child(summary),
                    p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4")
                        .child((
                            format!("{} min read", read_time),
                            primary_tag.map(|t| format!(" · {}", t)),
                        )),
                )),
            )),
    )
}

/// Best-effort parse of the `created_at` string into (kicker, day, meta).
/// Accepts ISO-ish strings (e.g. "2026-04-29" or "2026-04-29T12:00:00Z").
/// Falls back to ("POSTED", "·", raw) if the format is unexpected.
fn parse_date_pieces(created_at: &str, featured: bool) -> (String, String, String) {
    let kicker = if featured {
        "MOST RECENT".to_string()
    } else {
        "POSTED".to_string()
    };

    let date_part = created_at.split('T').next().unwrap_or(created_at);
    let parts: Vec<&str> = date_part.split('-').collect();
    // Only accept ISO-shaped YYYY-MM-DD where YYYY is 4 digits, MM is a known
    // month, and DD is 1-2 digits. Otherwise fall through to the safe default.
    if parts.len() == 3
        && parts[0].len() == 4
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
        && !parts[2].is_empty()
        && parts[2].len() <= 2
        && month_abbr(parts[1]).is_some()
    {
        let year = parts[0];
        let month_name = month_abbr(parts[1]).unwrap();
        let day = parts[2].trim_start_matches('0').to_string();
        (kicker, day, format!("{} {}", month_name, year))
    } else {
        (kicker, "·".to_string(), created_at.to_string())
    }
}

fn month_abbr(month_num: &str) -> Option<&'static str> {
    match month_num {
        "01" => Some("JAN"),
        "02" => Some("FEB"),
        "03" => Some("MAR"),
        "04" => Some("APR"),
        "05" => Some("MAY"),
        "06" => Some("JUN"),
        "07" => Some("JUL"),
        "08" => Some("AUG"),
        "09" => Some("SEP"),
        "10" => Some("OCT"),
        "11" => Some("NOV"),
        "12" => Some("DEC"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_list_row_structure() {
        let _: fn(Post, PostListSize, bool) -> _ = component;
    }

    #[test]
    fn test_parse_date_iso_basic() {
        let (k, d, m) = parse_date_pieces("2026-04-29", false);
        assert_eq!(k, "POSTED");
        assert_eq!(d, "29");
        assert_eq!(m, "APR 2026");
    }

    #[test]
    fn test_parse_date_iso_with_time() {
        let (_, d, m) = parse_date_pieces("2026-12-01T12:00:00Z", true);
        assert_eq!(d, "1");
        assert_eq!(m, "DEC 2026");
    }

    #[test]
    fn test_parse_date_featured_kicker() {
        let (k, _, _) = parse_date_pieces("2026-04-29", true);
        assert_eq!(k, "MOST RECENT");
    }

    #[test]
    fn test_parse_date_unknown_format() {
        let (k, d, m) = parse_date_pieces("not-a-date", false);
        assert_eq!(k, "POSTED");
        assert_eq!(d, "·");
        assert_eq!(m, "not-a-date");
    }
}
