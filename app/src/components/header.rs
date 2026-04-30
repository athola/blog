//! This module defines the `header` component, which provides the main navigation
//! bar for the application.
//!
//! It includes links for different sections of the blog and integrates
//! the `icons` component for social media or other branding elements.

use crate::icons;
use leptos::prelude::*;
use leptos_router::components::A;

/// Renders the application's header, including navigation links and optional icons.
///
/// Uses `leptos_router::components::A` for client-side navigation to prevent full page reloads.
pub fn component() -> impl IntoView {
    // NOTE: This is the transitional Sprint 0 header. Sprint 1 T07/T08 replace
    // it with Nameplate + PipeNav components per spec §3.2.
    view! {
        <header class="relative py-6 px-4 md:px-6 border-b-2 border-rule">
            <div class="container mx-auto max-w-5xl">
                <div class="flex flex-row justify-between items-center text-ink">
                    <div class="flex flex-row gap-4">
                        <div class="text-lg font-bold transition-colors sm:text-3xl hover:text-accent">
                            <A href="/">"blog"</A>
                        </div>
                        <div class="text-lg font-bold transition-colors sm:text-3xl hover:text-accent">
                            <A href="/references">"references"</A>
                        </div>
                        <div class="text-lg font-bold transition-colors sm:text-3xl hover:text-accent">
                            <A href="/contact">"contact"</A>
                        </div>
                    </div>
                    <div class="hidden md:block">
                        {icons::component()}
                    </div>
                </div>
            </div>
        </header>
    }
}
