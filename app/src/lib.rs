// Core application modules and components
use crate::components::{error_template, header, icons};
use chrono::{Datelike as _, Utc};
use leptos::{
    html::{a, body, div, footer, head, html, main, meta, p},
    prelude::*,
};
use leptos_meta::{MetaTags, Stylesheet, StylesheetProps, Title, TitleProps, provide_meta_context};
use leptos_router::{
    ParamSegment, SsrMode, StaticSegment,
    components::{FlatRoutes, Route, Router},
};

mod activity;
pub mod api;
mod components;
mod contact;
mod home;
mod post;
mod references;
pub mod types;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let html_comp = html().lang("en").child((
        head().child((
            meta().charset("utf-8"),
            meta()
                .name("viewport")
                .content("width=device-width, initial-scale=1"),
            // AutoReload(AutoReloadProps::builder().options(options.clone()).build()),
            HydrationScripts(HydrationScriptsProps::builder().options(options).build()),
            MetaTags(),
            Stylesheet(
                StylesheetProps::builder()
                    .id("leptos")
                    .href("/pkg/blog.css")
                    .build(),
            ),
            Stylesheet(
                StylesheetProps::builder()
                    .id("katex")
                    .href("/public/katex.min.css")
                    .build(),
            ),
            Title(
                TitleProps::builder()
                    .text("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                    .build(),
            ),
        )),
        body().class("bg-[#1e1e1e]").child(self::component),
    ));

    view! {
        <!DOCTYPE html>
        {html_comp}
    }
}

#[must_use]
pub fn component() -> impl IntoView {
    view! {
        <Router>
            <div class="overflow-auto text-white font-poppins">
                {header::component}
                <main class="container flex flex-col gap-8 px-4 pt-10 pb-14 mx-auto mt-16 max-w-4xl md:px-0">
                    <FlatRoutes fallback=|| {
                        let mut outside_errors = Errors::default();
                        outside_errors.insert_with_default_key(error_template::AppError::NotFound);
                        error_template::component(Some(outside_errors), None)
                    }>
                        <Route path=StaticSegment("") view=home::component ssr=SsrMode::InOrder/>
                        <Route path=StaticSegment("references") view=references::component/>
                        <Route path=StaticSegment("contact") view=contact::component/>
                        <Route path=(StaticSegment("post"), ParamSegment("slug")) view=post::component ssr=SsrMode::Async/>
                        <Route path=StaticSegment("activity") view=activity::component/>
                    </FlatRoutes>
                </main>
                {footer_component()}
            </div>
        </Router>
    }
}

fn footer_component() -> impl IntoView {
    footer()
        .class("fixed right-0 bottom-0 left-0 z-10 py-2 text-center md:py-4 bg-[#1e1e1e]/80 backdrop-blur-md")
        .child(
            div().class("flex flex-col gap-1 justify-center items-center").child((
                p().class("text-gray-400").child((
                    "Powered by",
                    a()
                        .href("https://github.com/athola")
                        .class("hover:underline text-[#ffef5c]")
                        .child(" athola"),
                    format!(" \u{a9} {}", Utc::now().year()),
                )),
                div().class("block md:hidden").child(icons::component),
            )),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        // Test shell function with default options
        let options = LeptosOptions::builder().output_name("blog").build();
        let shell_view = shell(options);
        // Verify the shell returns a non-null view
        // We can't easily test the rendered content without a full Leptos context,
        // but we can verify the function executes without panicking
        drop(shell_view); // Explicitly consume the view to verify it was created
    }

    #[test]
    fn test_component_function_signatures() {
        // Test that component functions exist with correct signatures
        // Following Leptos best practices: test logic separately, not component rendering

        // Verify function signatures compile and are callable
        let _shell_fn: fn(LeptosOptions) -> _ = shell;
        let _component_fn: fn() -> _ = component;

        // Test that LeptosOptions can be created (this is the testable logic)
        let options = LeptosOptions::builder().output_name("blog").build();
        assert_eq!(options.site_addr.port(), 3000); // Default port
        assert_eq!(options.site_addr.ip().to_string(), "127.0.0.1"); // Default IP
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_server_functions_integration() {
        // Test that server functions maintain correct signatures after retry implementation

        use crate::api::*;

        // Verify server function signatures haven't changed due to retry logic
        let _posts_fn: fn(Vec<String>) -> _ = select_posts;
        let _tags_fn: fn() -> _ = select_tags;
        let _post_fn: fn(String) -> _ = select_post;
        let _views_fn: fn(String) -> _ = increment_views;
        let _contact_fn: fn(ContactRequest) -> _ = contact;
        let _refs_fn: fn() -> _ = select_references;

        // Test that ContactRequest can be created and has expected fields
        let request = ContactRequest::default();
        assert_eq!(request.name, "");
        assert_eq!(request.email, "");
        assert_eq!(request.subject, "");
        assert_eq!(request.message, "");
    }
}
