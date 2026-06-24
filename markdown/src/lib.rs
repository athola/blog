//! This crate provides functionality for processing Markdown content, including
//! syntax highlighting for code blocks and KaTeX rendering for math.

use leptos::prelude::ServerFnError;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::SyntaxSet;

/// KaTeX options for display-mode (block) math, built once on first use.
static DISPLAY_MATH_OPTS: LazyLock<katex::Opts> = LazyLock::new(|| {
    katex::Opts::builder()
        .display_mode(true)
        .build()
        .expect("static katex display options are valid")
});

/// Renders a math event to HTML, falling back to the raw expression rather than
/// panicking the worker thread when KaTeX rejects user-supplied math (undefined
/// commands, unbalanced braces, etc.).
fn process_math_event(event: Event<'_>) -> Event<'_> {
    match event {
        Event::InlineMath(math_exp) => match katex::render(&math_exp) {
            Ok(html) => Event::InlineHtml(CowStr::from(html)),
            Err(_) => Event::InlineHtml(CowStr::from(format!(
                "<code>{}</code>",
                escape_html(&math_exp)
            ))),
        },
        Event::DisplayMath(math_exp) => {
            match katex::render_with_opts(&math_exp, &*DISPLAY_MATH_OPTS) {
                Ok(html) => Event::Html(CowStr::from(html)),
                Err(_) => Event::Html(CowStr::from(format!(
                    "<pre><code>{}</code></pre>",
                    escape_html(&math_exp)
                ))),
            }
        }
        _ => event,
    }
}

/// Minimal HTML escaping for raw text emitted on a renderer fallback path.
fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Centered `<img>` wrapper shared by the markdown image pre-pass and the
/// pulldown-cmark `Tag::Image` branch. SVGs are colour-inverted for dark mode.
fn center_image_html(img_path: &str, img_format: &str) -> String {
    let extra_style = if img_format == "svg" {
        "filter: invert(100%); width: 100%;"
    } else {
        "width: 100%;"
    };
    format!(
        r#"<div style="display: flex; justify-content: center;"><img src="{img_path}" style="{extra_style}"></div>"#
    )
}

/// Rewrites Markdown image syntax into centered HTML, borrowing the input
/// unchanged when it contains no images (the common case).
fn preprocess_images(markdown: &str) -> Cow<'_, str> {
    static RE_IMG: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"!\[.*?\]\((.*?\.(svg|png|jpe?g|gif|bmp|webp))\)")
            .expect("static image regex is valid")
    });

    let caps: Vec<_> = RE_IMG.captures_iter(markdown).collect();
    if caps.is_empty() {
        return Cow::Borrowed(markdown);
    }

    let mut result = String::with_capacity(markdown.len() + 256);
    let mut last_end = 0;

    for cap in caps {
        if let Some(full_match) = cap.get(0) {
            result.push_str(&markdown[last_end..full_match.start()]);
            let img_path = &cap[1];
            let img_format = &cap[2];

            result.push_str(&center_image_html(img_path, img_format));
            last_end = full_match.end();
        }
    }
    result.push_str(&markdown[last_end..]);
    Cow::Owned(result)
}

/// Syntax-highlights a single fenced code block into inline-styled HTML.
fn highlight_code_block(
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
    let processed_markdown = preprocess_images(markdown);

    // Configure pulldown-cmark parser.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(&processed_markdown, options);

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
                    highlight_code_block(&code_block_content, language, &ps, theme)?;
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
                events.push(Event::Html(CowStr::from(center_image_html(
                    &img_path,
                    &img_format,
                ))));
                skip_image = true;
            }
            Event::End(TagEnd::Image) => {
                if !skip_image {
                    events.push(Event::End(TagEnd::Image));
                }
                skip_image = false;
            }
            other => {
                let processed = process_math_event(other);
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
        // KaTeX wraps successful renders in `<span class="katex">`; asserting on
        // that marker proves the math was actually rendered rather than passed
        // through as raw text via the fallback path.
        assert!(
            html.contains("katex"),
            "expected KaTeX-rendered output, got: {html}"
        );
    }

    #[test]
    fn test_process_markdown_malformed_math_does_not_panic() {
        // An undefined KaTeX control sequence makes katex::render return Err.
        // The renderer must degrade gracefully (raw expression preserved),
        // not panic the worker thread on user-supplied post content.
        let markdown = "inline $\\undefinedcmd{x}$ and block $$\\undefinedcmd{y}$$";
        let html = process_markdown(markdown).expect("malformed math must not error the page");
        assert!(
            html.contains("undefinedcmd"),
            "raw expression should survive as fallback, got: {html}"
        );
    }

    #[test]
    fn test_markdown_formatting() {
        let markdown = "**bold** and *italic*";
        let html = process_markdown(markdown).unwrap();
        // pulldown-cmark emits semantic tags (`<strong>`/`<em>`), never the
        // legacy `<b>`/`<i>` forms, so assert on the exact expected output.
        assert!(html.contains("<strong>bold</strong>"), "got: {html}");
        assert!(html.contains("<em>italic</em>"), "got: {html}");
    }
}
