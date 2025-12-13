//! This module defines the `error_template` component, which provides a standardized
//! way to display error messages within the Leptos application.
//!
//! It includes an `AppError` enum for specific application-level errors (e.g., Not Found)
//! and a component that renders a user-friendly error page, setting the appropriate
//! HTTP status code on the server.

use http::status::StatusCode;
use leptos::{
    html::{div, h1},
    prelude::*,
    svg::{path, svg},
};
use leptos_router::components::{A, AProps};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
}

impl AppError {
    /// Returns the HTTP status code associated with the error.
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

/// Renders a generic error page based on the provided errors.
///
/// This component is designed to be used with Leptos error boundaries. It extracts
/// `AppError` instances from the `Errors` context and displays them.
/// On the server-side, it also sets the HTTP response status code based on the
/// first encountered `AppError`.
///
/// # Arguments
///
/// * `outside_errors` - An `Option<Errors>` containing errors passed from outside
///   the component, typically from a server-side rendering context.
/// * `errors` - An `Option<RwSignal<Errors>>` for client-side error handling,
///   representing errors managed reactively.
///
/// # Returns
///
/// An `impl IntoView` representing the rendered error page.
pub fn component(
    outside_errors: Option<Errors>,
    errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    // Determine the source of errors: prioritize `outside_errors` for SSR,
    // otherwise use the `errors` signal for client-side.
    let errors = outside_errors.map_or_else(
        || errors.unwrap_or_else(|| panic!("No Errors found and we expected errors!")),
        |e| RwSignal::new(e),
    );
    // Retrieve errors from the signal without subscribing to changes.
    let errors: Vec<AppError> = errors
        .get_untracked()
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();

    // On the server, set the HTTP response status code based on the first error.
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    div().class("grid place-content-center px-4 h-full antialiased").child((
        h1().class("mb-6 text-center").child(if errors.len() > 1 { "Errors" } else { "Error" }),
        For(
            ForProps::builder()
                .each(move || errors.clone().into_iter().enumerate())
                .key(|(index, _error)| *index)
                .children(|error| {
                    let error_string = error.1.to_string();
                    let error_code = error.1.status_code();

                    div().class("flex flex-col gap-1 justify-center items-center").child((
                        h1().class("text-xl tracking-widest text-gray-400 uppercase").child(
                            format!("{error_code}| {error_string}")
                        ),
                        div().class("flex gap-1 justify-center items-center mt-6 text-center duration-200 hover:text-[#68b5fc]").child(
                            A(AProps::builder()
                                .href("/")
                                .children(ToChildren::to_children(move || {
                                    vec![
                                        svg().attr("width", "1.1em").attr("height", "1.1em").attr("viewBox", "0 0 24 24").attr("fill", "currentColor").attr("role", "graphics-symbol").attr("data-hk", "0-0-0-98").child(
                                            path().attr("d", "M21 11H6.414l5.293-5.293-1.414-1.414L2.586 12l7.707 7.707 1.414-1.414L6.414 13H21z"),
                                        ).into_any(),
                                        "Go back home".into_any(),
                                    ]
                                }))
                                .build()
                            )
                        )
                    ))
                }).build(),
        ),
    ))
}
