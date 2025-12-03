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
    view! {
        <header class="fixed top-0 right-0 left-0 z-10 py-6 px-4 md:px-6 bg-[#1e1e1e]/80 backdrop-blur-md">
            <div class="container mx-auto max-w-5xl">
                <div class="flex flex-row justify-between items-center text-white">
                    <div class="flex flex-row gap-4">
                        <div class="text-lg font-bold transition-all duration-500 sm:text-3xl hover:text-[#ffef5c]">
                            <A href="/">"blog"</A>
                        </div>
                        <div class="text-lg font-bold transition-all duration-500 sm:text-3xl hover:text-[#ffef5c]">
                            <A href="/references">"references"</A>
                        </div>
                        <div class="text-lg font-bold transition-all duration-500 sm:text-3xl hover:text-[#ffef5c]">
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
