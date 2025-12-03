//! This crate provides functionality for processing Markdown content, including
//! syntax highlighting for code blocks and KaTeX rendering for math.

use leptos::prelude::ServerFnError;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use regex::Regex;
use std::borrow::Cow;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::SyntaxSet;

struct MathEventProcessor {
    display_style_opts: katex::Opts,
}

impl MathEventProcessor {
    fn new() -> Self {
        let opts = katex::Opts::builder().display_mode(true).build().unwrap();
        Self {
            display_style_opts: opts,
        }
    }

    fn process_math_event<'a>(&'a self, event: Event<'a>) -> Event<'a> {
        match event {
            Event::InlineMath(math_exp) => {
                Event::InlineHtml(CowStr::from(katex::render(&math_exp).unwrap()))
            }
            Event::DisplayMath(math_exp) => Event::Html(CowStr::from(
                katex::render_with_opts(&math_exp, &self.display_style_opts).unwrap(),
            )),
            _ => event,
        }
    }
}

/// Process markdown content with optimized image handling using Cow to reduce allocations.
/// Returns Cow<str> to avoid unnecessary string allocations when no images are found.
fn process_images_with_cow(markdown: &str) -> Cow<'_, str> {
    let re_img = Regex::new(r"!\[.*?\]\((.*?\.(svg|png|jpe?g|gif|bmp|webp))\)").unwrap();

    let caps: Vec<_> = re_img.captures_iter(markdown).collect();
    if caps.is_empty() {
        // No images found, return original content without allocation
        return Cow::Borrowed(markdown);
    }

    // Images found, need to allocate new string
    let mut result = String::with_capacity(markdown.len() + 256); // Pre-allocate with extra space for HTML wrappers
    let mut last_end = 0;

    for cap in caps {
        if let Some(full_match) = cap.get(0) {
            result.push_str(&markdown[last_end..full_match.start()]);
            let img_path = &cap[1];
            let img_format = &cap[2];

            let img_html = if img_format == "svg" {
                format!(
                    r#"<div style="display: flex; justify-content: center;"><img src="{img_path}" style="filter: invert(100%); width: 100%;"></div>"#
                )
            } else {
                format!(
                    r#"<div style="display: flex; justify-content: center;"><img src="{img_path}" style="width: 100%;"></div>"#
                )
            };
            result.push_str(&img_html);
            last_end = full_match.end();
        }
    }
    result.push_str(&markdown[last_end..]);
    Cow::Owned(result)
}

/// Process code block content with optimized highlighting, using pre-allocated capacity
/// to reduce string reallocations during HTML generation.
fn highlight_code_block_optimized(
    content: &str,
    language: &str,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Result<String, ServerFnError> {
    let syntax = ps
        .find_syntax_by_token(language)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, theme);

    // Pre-allocate capacity to avoid reallocations
    let mut highlighted_html = String::with_capacity(content.len() * 3);
    highlighted_html.push_str(
        r#"<pre style="background-color: #2b303b; padding: 8px; border-radius: 8px"><code>"#,
    );

    for line in content.lines() {
        let ranges = h.highlight_line(line, ps)?;
        let escaped = styled_line_to_highlighted_html(&ranges, IncludeBackground::No)?;
        highlighted_html.push_str(&escaped);
        highlighted_html.push('\n');
    }
    highlighted_html.push_str("</code></pre>");

    Ok(highlighted_html)
}

pub fn process_markdown(markdown: &str) -> Result<String, ServerFnError> {
    // Initialize syntax highlighting components from `syntect`.
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-eighties.dark"];

    // Process images with Cow optimization to reduce allocations
    let processed_markdown = process_images_with_cow(markdown);

    // Configure pulldown-cmark parser.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(&processed_markdown, options);
    let mep = MathEventProcessor::new();

    let mut events = Vec::new();
    let mut code_block_language: Option<String> = None;
    let mut code_block_content = String::new();
    let mut in_code_block = false;
    let mut skip_image = false;

    for event in TextMergeStream::new(parser) {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_content.clear();
                code_block_language = match kind {
                    CodeBlockKind::Fenced(info) => {
                        let lang = info.split_whitespace().next().unwrap_or("").to_owned();
                        Some(lang)
                    }
                    CodeBlockKind::Indented => None,
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let language = code_block_language.as_deref().unwrap_or("plaintext");

                let highlighted_html =
                    highlight_code_block_optimized(&code_block_content, language, &ps, theme)?;
                events.push(Event::Html(CowStr::from(highlighted_html)));
                code_block_language = None;
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else {
                    events.push(Event::Text(text));
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                let img_path = dest_url.into_string();
                let img_format = img_path.split('.').next_back().unwrap_or("").to_lowercase();
                let img_html = if img_format == "svg" {
                    format!(
                        r#"<div style="display: flex; justify-content: center;"><img src="{img_path}" style="filter: invert(100%); width: 100%;"></div>"#
                    )
                } else {
                    format!(
                        r#"<div style="display: flex; justify-content: center;"><img src="{img_path}" style="width: 100%;"></div>"#
                    )
                };
                events.push(Event::Html(CowStr::from(img_html)));
                skip_image = true;
            }
            Event::End(TagEnd::Image) => {
                if !skip_image {
                    events.push(Event::End(TagEnd::Image));
                }
                skip_image = false;
            }
            other => {
                let processed = mep.process_math_event(other);
                events.push(processed);
            }
        }
    }

    let mut html_output = String::new();
    push_html(&mut html_output, events.into_iter());

    Ok(html_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_markdown_basic() {
        let markdown = "# Hello World\n\nThis is a test.";
        let html = process_markdown(markdown).unwrap();
        assert!(html.contains("<h1"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("<p"));
        assert!(html.contains("This is a test"));
    }

    #[test]
    fn test_process_markdown_code_block() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let html = process_markdown(markdown).unwrap();
        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("main"));
    }

    #[test]
    fn test_process_markdown_empty() {
        let markdown = "";
        let html = process_markdown(markdown).unwrap();
        assert!(html.is_empty() || html.trim().is_empty());
    }

    #[test]
    fn test_process_markdown_math() {
        let markdown = "This is inline math: $x^2$\n\n$$\\int_0^1 x \\, dx$$";
        let html = process_markdown(markdown).unwrap();
        assert!(html.contains("x^2") || html.contains("math"));
    }

    #[test]
    fn test_markdown_formatting() {
        let markdown = "**bold** and *italic*";
        let html = process_markdown(markdown).unwrap();
        assert!(html.contains("<strong>") || html.contains("<b>"));
        assert!(html.contains("<em>") || html.contains("<i>"));
    }
}
