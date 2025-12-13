//! `app` crate provides the core Leptos application logic, including the main shell,
//! root component, routing, and shared UI components.
//!
//! This crate is responsible for:
//! - Setting up the HTML shell and meta context.
//! - Defining the main application component and its routes.
//! - Integrating various sub-components like headers, footers, and page-specific views.
//! - Handling global error fallbacks.

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
use std::sync::Arc;

mod activity;
pub mod api; // API endpoints and types
mod components; // Reusable UI components
mod contact; // Contact page logic and components
mod home; // Homepage logic and components
mod post; // Post display logic and components
mod references; // References page logic and components
pub mod types; // Shared type definitions

/// Renders the HTML shell for the application, including `<head>` and `<body>` content.
///
/// This function sets up the basic HTML structure, integrates meta tags, stylesheets,
/// and hydration scripts, and then renders the main application component within the `<body>`.
pub fn shell(options: Arc<LeptosOptions>) -> impl IntoView {
    // Unwraps the Arc to get LeptosOptions. If the Arc has multiple strong references,
    // it clones the options; otherwise, it takes ownership.
    let options = Arc::try_unwrap(options).unwrap_or_else(|shared| shared.as_ref().clone());

    // Provides context for managing stylesheets, titles, meta tags, etc., throughout the app.
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

/// The root component of the application, responsible for defining the main layout and routing.
///
/// This component sets up the router, global layout (header, main content area, footer),
/// and defines the application's routes. It also includes a fallback for 404 (Not Found) errors.
#[must_use]
pub fn component() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <div class="overflow-auto text-white font-poppins">
                {move || header::component}
                <main class="container flex flex-col gap-8 px-4 pt-10 pb-14 mx-auto mt-16 max-w-4xl md:px-0">
                    <FlatRoutes fallback=|| {
                        // Handle 404 (Not Found) errors by rendering a specific error template.
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

/// Renders the application's footer component.
///
/// This includes copyright information, a link to the author's GitHub, and dynamically
/// displays the current year. It also conditionally shows icons on smaller screens.
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
    /// Test the shell function to ensure it creates a view without panicking.
    fn test_shell_creation() {
        // Create default LeptosOptions for testing.
        let options = LeptosOptions::builder().output_name("blog").build();
        let shell_view = shell(Arc::new(options));
        // Consume the view to confirm successful creation and prevent unused variable warnings.
        drop(shell_view);
    }

    #[test]
    /// Verify that core component functions (`shell`, `component`) exist with correct signatures.
    fn test_component_function_signatures() {
        // Ensure function signatures compile and are callable.
        let _shell_fn: fn(Arc<LeptosOptions>) -> _ = shell;
        let _component_fn: fn() -> _ = component;

        // Test that LeptosOptions can be created and has expected default values.
        let options = LeptosOptions::builder().output_name("blog").build();
        assert_eq!(options.site_addr.port(), 3000); // Check default port.
        assert_eq!(options.site_addr.ip().to_string(), "127.0.0.1"); // Check default IP.
    }

    #[cfg(feature = "ssr")]
    #[test]
    /// Confirm that server function signatures remain consistent after retry implementation.
    fn test_server_functions_integration() {
        use crate::api::*;

        // Verify server function signatures are unchanged by retry logic.
        let _posts_fn: fn(Vec<String>) -> _ = select_posts;
        let _tags_fn: fn() -> _ = select_tags;
        let _post_fn: fn(String) -> _ = select_post;
        let _views_fn: fn(String) -> _ = increment_views;
        let _contact_fn: fn(ContactRequest) -> _ = contact;
        let _refs_fn: fn() -> _ = select_references;

        // Ensure a ContactRequest can be created with expected default field values.
        let request = ContactRequest::default();
        assert_eq!(request.name, "");
        assert_eq!(request.email, "");
        assert_eq!(request.subject, "");
        assert_eq!(request.message, "");
    }
}
