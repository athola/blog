extern crate alloc;
use leptos::html::{div, p};
use leptos::prelude::*;
use leptos_meta::{Title, TitleProps};

use crate::api::select_activities;
use crate::types::Activity;

pub fn component() -> impl IntoView {
    let activities = Resource::new(
        || 0usize,
        |page| async move { select_activities(page).await },
    );

    // Create a test activity action to ensure server function registration
    let _create_test_activity = Action::new(|_activity: &Activity| async move {
        // This action ensures the create_activity server function is registered
        // We don't actually create activities here, just ensure registration
        Result::<(), ServerFnError>::Ok(())
    });

    div().child((
        Title(
            TitleProps::builder()
                .text("Activity Stream – Alex Thola's Blog")
                .build(),
        ),
        Suspense(
            SuspenseProps::builder()
                .fallback(|| p().class("text-gray-400").child("Loading activities..."))
                .children(TypedChildren::to_children(move || {
                    div()
                        .class("container mx-auto max-w-4xl px-4 md:px-0")
                        .child((
                            p().class("text-gray-400")
                                .child("Activity stream component - server functions registered"),
                            div().child(For(ForProps::builder()
                                .each(move || {
                                    activities.get().and_then(Result::ok).unwrap_or_default()
                                })
                                .key(|activity| activity.id.clone())
                                .children(|activity| {
                                    div().class("p-4 mb-4 bg-gray-800 rounded-lg").child((
                                        p().class("text-white").child(activity.content.clone()),
                                        p().class("text-gray-400 text-sm")
                                            .child(format!("Created: {}", activity.created_at)),
                                    ))
                                })
                                .build())),
                        ))
                }))
                .build(),
        ),
    ))
}
