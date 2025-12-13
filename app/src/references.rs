//! This module defines the `references` component, which renders a portfolio
//! or list of project references.
//!
//! It fetches project data (title, description, tech stack) from the API and
//! displays each project with its technical details and skill percentages.

use crate::api::select_references;
use leptos::{
    html::{div, h1, h3, p, section, span},
    prelude::*,
};

/// Renders the references page, displaying a portfolio of projects.
///
/// This component fetches project data via the `references` resource,
/// which calls the `select_references` server function. It then iterates
/// through these projects, rendering each with its title, description,
/// and a visual representation of its tech stack.
pub fn component() -> impl IntoView {
    // Resource to fetch project references from the server.
    // #[expect(clippy::redundant_closure_call)] // This clippy warning is suppressed because of the way Leptos Resources are typically defined.
    let references = Resource::new_blocking(
        || (),
        move |()| async move { select_references().await.unwrap_or_default() },
    );

    div().class("container py-12 px-4 mx-auto").child((
        section().id("about").class("mx-auto mb-16 max-w-4xl text-center").child((
            h1().class("mb-8 text-5xl font-bold md:text-7xl text-[#ffef5c]").child("Project References"),
            p().class("mb-8 text-lg text-gray-300 md:text-xl").child("Explore my portfolio of successful projects. I enjoy solving network and OS problems with high performance solutions."),
        )),
        section().id("projects").class("mx-auto max-w-5xl").child(
            div().class("grid gap-8").child(
                Suspense(
                    SuspenseProps::builder()
                        // No specific fallback content is rendered here; the component
                        // simply waits for data to load before rendering anything.
                        .fallback(|| ())
                        .children(TypedChildren::to_children(move || {
                            For(
                                ForProps::builder()
                                    .each(move || references.get().unwrap_or_default())
                                    .key(|r| format!("{:?}", r.id))
                                    .children(|r| {
                                        // Render each project reference as a stylized card.
                                        div().class("relative p-6 rounded-2xl transition-colors duration-500 group bg-[#ffef5c]/8 hover:bg-[#ffef5c]/10").child((
                                            div().class("absolute inset-0 rounded-2xl -z-10 blur-2xl"),
                                            div().class("absolute inset-2 rounded-xl border shadow-lg -z-10 bg-[#ffef5c]/10 backdrop-blur-xl shadow-[#ffef5c]/5 border-[#ffef5c]/20"),
                                            div().class("absolute inset-2 rounded-xl border -z-10 backdrop-blur-2xl bg-white/5 border-white/10").child(
                                                div().class("absolute inset-0 bg-[linear-gradient(0deg,transparent_24px,rgba(255,255,255,0.03)_25px),linear-gradient(90deg,transparent_24px,rgba(255,255,255,0.03)_25px)] bg-[size:25px_25px]"),
                                            ),
                                            div().class("flex relative flex-col").child((
                                                h3().class("mb-2 text-xl font-bold text-[#ffef5c]").child(r.title),
                                                p().class("flex-grow mb-4 text-sm text-gray-300").child(r.description),
                                                div().class("grid grid-cols-2 gap-4").child(
                                                    For(
                                                        ForProps::builder()
                                                            // Combine tech stack names with their corresponding percentages.
                                                            .each(move || {
                                                                r.tech_stack
                                                                    .clone()
                                                                    .into_iter()
                                                                    .zip(r.teck_stack_percentage.clone())
                                                                    .collect::<Vec<_>>()
                                                            })
                                                            .key(|tech| tech.0.to_string())
                                                            .children(|tech| {
                                                                div().child(
                                                                    (
                                                                        div().class("flex justify-between items-center mb-1")
                                                                            .child((
                                                                                span().class("text-xs font-medium text-[#ffef5c]").child(tech.0.to_string()),
                                                                                span().class("text-xs text-gray-400").child(format!("{}%", tech.1))
                                                                            )),
                                                                        div().class("overflow-hidden h-1.5 rounded-full bg-black/40 backdrop-blur-sm")
                                                                            .child(
                                                                                div()
                                                                                    .class("h-full bg-gradient-to-r from-[#ffef5c] to-[#ffef5c]")
                                                                                    // Ensure the width does not exceed 100%.
                                                                                    .style(format!("width: {}%", tech.1.min(100))),
                                                                            )
                                                                ))
                                                            })
                                                            .build(),
                                                    ),
                                                ),
                                            )),
                                        ))
                                    })
                                    .build()
                            )
                        }))
                        .build()
                )
            )
        )
    ))
}
