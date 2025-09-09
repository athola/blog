// Core application modules and components
use crate::components::{error_template, header, icons};
use chrono::{Datelike as _, Utc};
use leptos::{
    html::{a, body, div, footer, head, html, main, meta, p},
    prelude::*,
};
use leptos_meta::{
    Link, LinkProps, Meta, MetaProps, MetaTags, Stylesheet, StylesheetProps, Title, TitleProps,
    provide_meta_context,
};
use leptos_router::{
    ParamSegment, SsrMode, StaticSegment,
    components::{
        FlatRoutes, FlatRoutesProps, Route, RouteChildren, RouteProps, Router, RouterProps,
    },
};

pub mod api;
mod components;
mod contact;
mod home;
mod post;
mod references;
pub mod types;

#[expect(clippy::too_many_lines)]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let html_comp = html().lang("en").child((
        head().child((
            meta().charset("utf-8"),
            meta()
                .name("viewport")
                .content("width=device-width, initial-scale=1"))).child(
            AutoReload(AutoReloadProps::builder().options(options.clone()).build())).child(
            HydrationScripts(HydrationScriptsProps::builder().options(options).build())).child(
            MetaTags()).child((
            Stylesheet(
                StylesheetProps::builder()
                    .id("leptos")
                    .href("/pkg/blog.css")
                    .build(),
            ),
            Stylesheet(
                StylesheetProps::builder()
                    .id("katex")
                    .href("/katex.min.css")
                    .build(),
            ))).child(
            Title(
                TitleProps::builder()
                    .text("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                    .build(),
            )).child((
            Meta(
                MetaProps::builder()
                    .name("hostname")
                    .content("alexthola.com")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("expected-hostname")
                    .content("alexthola.com")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("description")
                    .content(
                        "Explore open-source Rust projects, learn innovative techniques, and connect with a passionate community. Get expert Rust development and consulting services.",
                    )
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("keywords")
                    .content("alexthola, rust, ai, mathematics, embedded, web, systems, programming")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("robots")
                    .content("index, follow")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("googlebot")
                    .content("index, follow")
                    .build(),
            ))).child((
            // Facebook
            Meta(
                MetaProps::builder()
                    .property("og:type")
                    .content("website")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:title")
                    .content("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:site_name")
                    .content("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:description")
                    .content(
                        "Explore open-source Rust projects, learn innovative techniques, and connect with a passionate community. Get expert Rust development and consulting services.",
                    )
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:url")
                    .content("https://alexthola.com/")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:image")
                    .content("https://static.alexthola.com/alexthola_custom_bg.png")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:image:type")
                    .content("image/png")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:image:width")
                    .content("1200")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .property("og:image:height")
                    .content("627")
                    .build(),
            ))).child((
            // Twitter
            Meta(
                MetaProps::builder()
                    .name("twitter:card")
                    .content("summary_large_image")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:title")
                    .content("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:description")
                    .content(
                        "Explore open-source Rust projects, learn innovative techniques, and connect with a passionate community. Get expert Rust development and consulting services.",
                    )
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:site")
                    .content("@alexthola")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:url")
                    .content("https://alexthola.com/")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:image")
                    .content("https://static.alexthola.com/alexthola_custom_bg.png")
                    .build(),
            ),
            Meta(
                MetaProps::builder()
                    .name("twitter:image:alt")
                    .content("alexthola logo")
                    .build(),
            ))).child((
            Link(
                LinkProps::builder()
                    .rel("preconnect")
                    .href("https://fonts.googleapis.com")
                    .build(),
            ),
            Link(
                LinkProps::builder()
                    .rel("preconnect")
                    .href("https://fonts.gstatic.com")
                    .build(),
        ))),
        body().class("bg-[#1e1e1e]").child(self::component),
    ));

    view! {
        <!DOCTYPE html>
        {html_comp}
    }
}

#[must_use]
pub fn component() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    Router(
        RouterProps::builder()
            .children(TypedChildren::to_children(move || {
                div().class("overflow-auto text-white font-poppins").child((
          header::component,
          main()
            .class("container flex flex-col gap-8 px-4 pt-10 pb-14 mx-auto mt-16 max-w-4xl md:px-0")
            .child(FlatRoutes(
              FlatRoutesProps::builder()
                .fallback(|| {
                  let mut outside_errors = Errors::default();
                  outside_errors.insert_with_default_key(error_template::AppError::NotFound);
                  error_template::component(
                      Some(outside_errors),
                      None
                  )
                })
                .children(RouteChildren::to_children(move || {
                  (
                    Route(
                      RouteProps::builder()
                        .path(StaticSegment(""))
                        .view(home::component)
                        .ssr(SsrMode::InOrder)
                        .build(),
                    ),
                    Route(
                      RouteProps::builder()
                        .path(StaticSegment("references"))
                        .view(references::component)
                        .build(),
                    ),
                    Route(
                      RouteProps::builder()
                        .path(StaticSegment("contact"))
                        .view(contact::component)
                        .build(),
                    ),
                    Route(
                      RouteProps::builder()
                        .path((StaticSegment("post"), ParamSegment("slug")))
                        .view(post::component)
                        .ssr(SsrMode::Async)
                        .build(),
                    ),
                  )
                }))
                .build(),
            )),
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
            ),
        ))
            }))
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        // Test shell function with default options
        let options = LeptosOptions::default();
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
        let options = LeptosOptions::default();
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
