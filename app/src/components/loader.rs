//! This module defines the `loader` component, which displays a loading spinner
//! with an accompanying text message.
//!
//! It's typically used to provide visual feedback to users while content is being
//! fetched or processed asynchronously.

use leptos::{
    html::{div, img, p},
    prelude::*,
};

/// Renders a loading spinner component.
///
/// This component displays an animated Rust logo and a "Loading..." message,
/// providing visual feedback during asynchronous operations.
pub fn component() -> impl IntoView {
    div()
        .class("flex absolute inset-0 flex-col gap-1 justify-center items-center m-auto")
        .child((
            img()
                .src("/rust_color.webp")
                .width(32)
                .height(32)
                .class("animate-spin"),
            p().class("text-sm italic text-muted-foreground")
                .child("Loading..."),
        ))
}
