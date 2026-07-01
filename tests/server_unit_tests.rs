//! Workspace-level smoke tests for server-side rendering helpers that don't
//! require the full HTTP/database stack.
//!
//! The `server` binary crate exposes no library target, so these tests exercise
//! the `markdown` rendering crate the server renders post bodies through, rather
//! than asserting on local literals.

#[cfg(test)]
mod server_unit_tests {
    #[test]
    fn markdown_renders_headings_and_code() {
        let html = markdown::process_markdown("# Title\n\n```rust\nfn main() {}\n```")
            .expect("markdown should render");
        assert!(html.contains("<h1"), "expected heading, got: {html}");
        assert!(html.contains("Title"));
        assert!(html.contains("<pre"), "expected code block, got: {html}");
    }

    #[test]
    fn markdown_degrades_on_malformed_math() {
        // Malformed LaTeX must not panic the renderer; the raw expression
        // survives as a fallback instead of crashing the worker thread.
        let html = markdown::process_markdown("bad $\\nope{x}$ math")
            .expect("malformed math must not error the page");
        assert!(
            html.contains("nope"),
            "raw expression should survive: {html}"
        );
    }

    #[test]
    fn markdown_centers_images() {
        let html = markdown::process_markdown("![alt](photo.png)").expect("should render");
        assert!(html.contains("justify-content: center"));
        assert!(html.contains("photo.png"));
    }
}
