//! Post detail page — the heart of the site.
//!
//! Layout (spec §4.2):
//!   1. Pre-title meta row (mono uppercase): time · read time · views
//!   2. Italic display date above title
//!   3. H1 — Fraunces 56px / 40px mobile
//!   4. Tag-byline + tiny author chip (between hairline rules)
//!   5. In-flow TOC for posts with ≥ 4 h2 headings
//!   6. Article body with editorial prose styling (ochre accent links etc.)
//!   7. Post foot — prev/next + more-from-tag + raw-md/copy/share

use leptos::{
    html::{a, article, div, img, p, section, span, time},
    prelude::*,
};
use leptos_meta::{Link, LinkProps, Title, TitleProps};
use leptos_router::{
    components::{A, AProps},
    hooks::use_params_map,
};

use crate::{
    api::{increment_views, select_post},
    components::{
        loader,
        toc::{self, TocHeading},
    },
    types::Post,
};

/// Renders the post detail page.
#[expect(clippy::too_many_lines)]
pub fn component() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());

    let post = Resource::new_blocking(
        || (),
        move |()| async move { select_post(slug()).await.unwrap_or_default() },
    );

    let _increment_view = Action::new(move |id: &String| {
        let id = id.clone();
        async move {
            let _ = increment_views(id.to_string()).await;
        }
    });

    Effect::new(move |_| {
        #[cfg(not(debug_assertions))]
        if post.get().is_some() {
            _increment_view.dispatch(format!("{:?}", post.get().as_ref().unwrap().id));
        }
    });

    let render_post = move |post_data: Post| {
        let headings = extract_h2_headings(&post_data.body);
        let (kicker, day_line, year_line) = format_post_date(&post_data.created_at);
        let primary_tag = post_data.tags.first().cloned().unwrap_or_default();
        let slug_for_links = post_data.slug.clone().unwrap_or_default();
        let tag_byline = post_data
            .tags
            .iter()
            .map(|t| t.to_uppercase())
            .collect::<Vec<_>>()
            .join(" · ");

        article().class("flex flex-col mx-auto max-w-prose pt-8 sm:pt-12").child((
            // Optional header image
            Show(
                ShowProps::builder()
                    .when({
                        let header_image = post_data.header_image.clone();
                        move || header_image.is_some()
                    })
                    .fallback(|| ())
                    .children(ToChildren::to_children({
                        let header_image = post_data.header_image.clone();
                        let title = post_data.title.clone();
                        move || {
                            img()
                                .alt(title.clone())
                                .class("w-full h-auto rounded-sm border border-rule-soft mb-8")
                                .src(header_image.clone().unwrap_or_default())
                        }
                    }))
                    .build(),
            ),
            // Pre-title meta row
            p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3 mb-2")
                .child(format!(
                    "{} · {} MIN READ · {} VIEWS",
                    kicker, post_data.read_time, post_data.total_views
                )),
            // Italic display date above title
            time()
                .attr("datetime", post_data.created_at.clone())
                .class("font-display italic text-2xl text-ink-3 leading-none mb-4")
                .child(format!("{} {}", day_line, year_line)),
            // H1
            Title(TitleProps::builder().text(post_data.title.clone()).build()),
            leptos::html::h1()
                .class("font-display text-4xl sm:text-5xl md:text-6xl font-semibold leading-[1.05] tracking-[-0.02em] text-ink mb-6")
                .child(post_data.title.clone()),
            // Tag-byline + author chip
            div()
                .class("flex flex-wrap justify-between items-center py-3 border-t border-b border-rule-soft mb-8 gap-3")
                .child((
                    span()
                        .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                        .child(if tag_byline.is_empty() {
                            "UNTAGGED".to_string()
                        } else {
                            tag_byline
                        }),
                    div().class("flex items-center gap-2").child((
                        span()
                            .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                            .child("by"),
                        span()
                            .class("text-sm text-ink-2 font-medium")
                            .child(post_data.author.name.clone()),
                    )),
                )),
            // In-flow TOC (renders only when ≥ TOC_MIN_HEADINGS = 4 h2 headings)
            toc::component(headings),
            // Article body with editorial prose styling
            div().class(
                "prose prose-lg max-w-none \
                 prose-headings:font-display prose-headings:text-ink \
                 prose-h2:italic prose-h2:font-medium prose-h2:text-3xl prose-h2:mt-12 prose-h2:mb-4 \
                 prose-h3:font-mono prose-h3:text-xs prose-h3:uppercase prose-h3:tracking-[0.08em] prose-h3:text-accent prose-h3:font-normal \
                 prose-p:text-ink prose-p:leading-[1.7] \
                 prose-a:text-ink prose-a:no-underline prose-a:border-b prose-a:border-accent hover:prose-a:text-accent \
                 prose-strong:text-ink prose-strong:font-semibold \
                 prose-em:text-ink-2 \
                 prose-blockquote:border-l-[3px] prose-blockquote:border-accent prose-blockquote:italic prose-blockquote:text-ink-2 prose-blockquote:not-italic prose-blockquote:font-normal \
                 prose-code:text-ink prose-code:bg-paper-2 prose-code:font-mono prose-code:rounded-sm prose-code:px-1 prose-code:py-0.5 prose-code:text-[0.92em] prose-code:before:content-none prose-code:after:content-none \
                 prose-pre:bg-paper-2 prose-pre:border prose-pre:border-rule-soft prose-pre:rounded-sm prose-pre:font-mono prose-pre:text-sm prose-pre:leading-[1.5] \
                 prose-img:rounded-sm prose-img:border prose-img:border-rule-soft \
                 prose-li:text-ink prose-ul:text-ink prose-ol:text-ink \
                 prose-table:text-ink prose-th:text-ink prose-thead:border-rule prose-th:border-rule-soft prose-td:border-rule-soft prose-tr:border-rule-soft",
            )
                .inner_html(post_data.body.clone()),
            // Post foot
            section()
                .class("mt-16 pt-8 border-t-2 border-rule flex flex-col gap-4")
                .child((
                    // Tags row
                    div().class("flex flex-wrap gap-2 items-center").child((
                        span().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-4").child("tags:"),
                        post_data.tags
                            .iter()
                            .enumerate()
                            .map(|(i, tag)| {
                                let tag = tag.clone();
                                let sep = if i > 0 {
                                    Some(span().class("text-ink-4").child("·"))
                                } else {
                                    None
                                };
                                let tag_for_href = tag.clone();
                                (
                                    sep,
                                    A(AProps::builder()
                                        .href(format!("/archive?tag={}", tag_for_href))
                                        .children(ToChildren::to_children(move || {
                                            span()
                                                .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3 hover:text-accent transition-colors")
                                                .child(tag.clone())
                                        }))
                                        .build()),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )),
                    // Prev/Next placeholder — links to /archive (deferred to post-graph API)
                    div().class("flex justify-between items-center font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3").child((
                        A(AProps::builder()
                            .href("/archive".to_string())
                            .children(ToChildren::to_children(move || {
                                span().class("hover:text-accent transition-colors").child("← all writing")
                            }))
                            .build()),
                        A(AProps::builder()
                            .href(format!("/archive?tag={}", primary_tag))
                            .children(ToChildren::to_children(move || {
                                span().class("hover:text-accent transition-colors").child("more from this tag →")
                            }))
                            .build()),
                    )),
                    // Tools: raw markdown link (copy/share deferred — would add web-sys feature flags)
                    div().class("flex flex-wrap gap-4 items-center font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3").child((
                        a()
                            .href(format!("/post/{}.md", slug_for_links))
                            .class("hover:text-accent transition-colors")
                            .child("raw markdown"),
                    )),
                )),
            // T28: per-post <link rel="alternate" type="text/markdown"> injected
            // into <head> via leptos_meta. Pairs with the server route
            // /post/{slug}.md (T25) to expose raw markdown source.
            Link(
                LinkProps::builder()
                    .rel("alternate")
                    .type_("text/markdown")
                    .href(format!("/post/{}.md", slug_for_links))
                    .build(),
            ),
            // Canonical URL helps search engines disambiguate.
            Link(
                LinkProps::builder()
                    .rel("canonical")
                    .href(format!("https://alexthola.com/post/{}", slug_for_links))
                    .build(),
            ),
        ))
    };

    Suspense(
        SuspenseProps::builder()
            .fallback(loader::component)
            .children(TypedChildren::to_children(move || {
                move || {
                    post.with(|loaded| {
                        let p = loaded.clone().unwrap_or_default();
                        render_post(p)
                    })
                }
            }))
            .build(),
    )
}

/// Best-effort string-parse extractor for `<h2 id="anchor">Text</h2>` elements.
/// Pulldown-cmark + GitHub-style anchor injection produces this shape.
/// Fall-through: if no h2s have an `id` attribute, returns empty (TOC then
/// renders nothing per Toc component's TOC_MIN_HEADINGS gate).
fn extract_h2_headings(html: &str) -> Vec<TocHeading> {
    let mut headings = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    while i + 8 <= bytes.len() {
        // Match <h2 with optional whitespace before id="
        if &bytes[i..i + 3] == b"<h2" {
            let after_h2 = i + 3;
            // Find the closing > of the opening tag
            let close_tag = match html[after_h2..].find('>') {
                Some(p) => after_h2 + p,
                None => break,
            };
            let opening_tag = &html[after_h2..close_tag];
            // Try to extract id="..."
            let anchor = if let Some(id_start) = opening_tag.find("id=\"") {
                let abs_id_start = after_h2 + id_start + 4;
                let rel = &html[abs_id_start..];
                rel.find('"')
                    .map(|end| html[abs_id_start..abs_id_start + end].to_string())
            } else {
                None
            };
            let text_start = close_tag + 1;
            if let Some(end) = html[text_start..].find("</h2>") {
                let raw_text = &html[text_start..text_start + end];
                let text = strip_html_tags(raw_text);
                if let Some(anchor) = anchor {
                    headings.push(TocHeading {
                        level: 2,
                        text,
                        anchor,
                    });
                }
                i = text_start + end + 5;
                continue;
            }
            break;
        }
        i += 1;
    }
    headings
}

/// Crude tag stripper for headings (handles `<em>foo</em>` and similar).
fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Returns (kicker, day_line, year_line) for the pre-title meta row.
/// Falls back to ("PUBLISHED", &raw, "") for unparsable dates.
fn format_post_date(created_at: &str) -> (String, String, String) {
    let date_part = created_at.split('T').next().unwrap_or(created_at);
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3
        && parts[0].len() == 4
        && parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[2].chars().all(|c| c.is_ascii_digit())
    {
        let year = parts[0];
        let month = month_long(parts[1]).unwrap_or(parts[1]);
        let day = parts[2].trim_start_matches('0');
        (
            "PUBLISHED".to_string(),
            format!("{} {},", month, day),
            year.to_string(),
        )
    } else {
        (
            "PUBLISHED".to_string(),
            created_at.to_string(),
            String::new(),
        )
    }
}

fn month_long(month_num: &str) -> Option<&'static str> {
    match month_num {
        "01" => Some("January"),
        "02" => Some("February"),
        "03" => Some("March"),
        "04" => Some("April"),
        "05" => Some("May"),
        "06" => Some("June"),
        "07" => Some("July"),
        "08" => Some("August"),
        "09" => Some("September"),
        "10" => Some("October"),
        "11" => Some("November"),
        "12" => Some("December"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_component_signature() {
        let _: fn() -> _ = component;
    }

    #[test]
    fn test_extract_h2_with_id() {
        let html = r#"<h2 id="intro">Introduction</h2>
        <p>blah</p>
        <h2 id="next-steps">Next steps</h2>"#;
        let h = extract_h2_headings(html);
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].anchor, "intro");
        assert_eq!(h[0].text, "Introduction");
        assert_eq!(h[1].anchor, "next-steps");
    }

    #[test]
    fn test_extract_h2_with_inline_em() {
        let html = r#"<h2 id="why">Why <em>this</em> matters</h2>"#;
        let h = extract_h2_headings(html);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].text, "Why this matters");
    }

    #[test]
    fn test_extract_h2_without_id_skipped() {
        let html = r#"<h2>No anchor</h2><h2 id="ok">Yes anchor</h2>"#;
        let h = extract_h2_headings(html);
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].anchor, "ok");
    }

    #[test]
    fn test_format_post_date_iso() {
        let (k, d, y) = format_post_date("2026-04-29");
        assert_eq!(k, "PUBLISHED");
        assert_eq!(d, "April 29,");
        assert_eq!(y, "2026");
    }

    #[test]
    fn test_format_post_date_with_time() {
        let (_, d, y) = format_post_date("2026-12-01T10:00:00Z");
        assert_eq!(d, "December 1,");
        assert_eq!(y, "2026");
    }

    #[test]
    fn test_format_post_date_unparsable() {
        let (_, d, y) = format_post_date("yesterday");
        assert_eq!(d, "yesterday");
        assert_eq!(y, "");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("plain"), "plain");
        assert_eq!(strip_html_tags("<em>foo</em>"), "foo");
        assert_eq!(strip_html_tags("<a href=\"x\">link</a> text"), "link text");
    }
}
