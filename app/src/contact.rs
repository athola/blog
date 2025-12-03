//! This module defines the `contact` component, which renders the contact page
//! of the application.
//!
//! It includes an "about me" section (`whoami`) and an interactive contact form.
//! The form manages its state, handles submissions using a Leptos server action,
//! and provides user feedback with loading and success indicators.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::{ContactRequest, contact};

/// Renders the contact page, featuring an "about me" section and a contact form.
///
/// This component manages the contact form's local state using `RwSignal`s for
/// form fields (`state`), submission status (`sent`), and a loading indicator (`loader`).
/// The form submission triggers a `Leptos Action` that calls the `contact` server function.
/// Visual feedback is provided during submission and upon successful completion.
pub fn component() -> impl IntoView {
    let state = RwSignal::new(ContactRequest::default());
    let sent = RwSignal::new(false);
    let loader = RwSignal::new(false);
    let submit = Action::new(move |data: &ContactRequest| {
        loader.set(true);
        let data = data.clone();

        async move {
            // Attempt to send the contact request via the server function.
            // Errors are handled within the `contact` server function's retry logic.
            let _ = contact(data).await;
            state.set(ContactRequest::default()); // Clear form fields on success.
            sent.set(true); // Indicate message was sent.
            loader.set(false); // Hide loading indicator.
        }
    });

    view! {
        <div class="min-h-screen text-white bg-[#1e1e1e]">
            <section class="px-4 pt-12 pb-24 sm:px-6 lg:px-8">
                <div class="mx-auto max-w-5xl">
                    <h1 class="mb-6 text-5xl font-extrabold leading-tight sm:text-6xl md:text-7xl text-[#ffef5c]">
                        "Rust Development"
                        <br/>
                        "for the Modern Era"
                    </h1>
                            <p class="mb-6 text-gray-300">"I design scalable architectures that are built for performance and reliability."</p>
                    <div class="inline-flex items-center text-lg font-semibold hover:underline text-[#ffef5c]">
                        <A href="/references">
                            "Explore my work"
                            <svg class="ml-2 size-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3"/>
                            </svg>
                        </A>
                    </div>
                </div>
            </section>



            <section class="py-20 px-4 sm:px-6 lg:px-8">
                <div class="mx-auto max-w-5xl">
                    <h2 class="mb-12 text-3xl font-bold text-[#ffef5c]">"whoami"</h2>
                    <div class="grid grid-cols-1 gap-12 md:grid-cols-1">
                        <div class="flex items space-x-6">
                            <img src="https://avatars.githubusercontent.com/u/9769290?s=400&u=84fd5ddd993912372430ed97b768bf202827989b&v=4" alt="Alex Thola" width="100" height="100" class="rounded-full"/>
                            <div>
                                <h3 class="mb-1 text-xl font-semibold text-white">"Alex Thola"</h3>
                                <p class="mb-2 text-gray-300">"Staff Software Engineer"</p>
                                <a href="https://www.linkedin.com/in/alexthola/" target="_blank" rel="noopener noreferrer" class="text-sm hover:underline text-[#ffef5c]">"LinkedIn Profile"</a>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="py-20 px-4 sm:px-6 lg:px-8 bg-[#2a2a2a]">
                <div class="mx-auto max-w-3xl">
                    <h2 class="mb-8 text-3xl font-bold text-[#ffef5c]">"Get In Touch"</h2>
                    <form class="space-y-6" on:submit=move |ev| {
                        ev.prevent_default();
                        let _ = submit.dispatch(state.get());
                    }>
                        <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                            <input
                                id="name"
                                name="name"
                                placeholder="Your Name"
                                type="text"
                                autocomplete="name"
                                prop:value=move || state.get().name
                                on:input=move |ev| {
                                    let name = event_target_value(&ev);
                                    state.update(|prev| prev.name = name);
                                }
                                class="py-3 px-4 w-full placeholder-gray-400 text-white transition-shadow focus:ring-2 focus:outline-none bg-[#1e1e1e] focus:ring-[#ffef5c]"
                            />
                            <input
                                id="email"
                                name="email"
                                placeholder="Your Email"
                                type="email"
                                autocomplete="email"
                                prop:value=move || state.get().email
                                on:input=move |ev| {
                                    let email = event_target_value(&ev);
                                    state.update(|prev| prev.email = email);
                                }
                                class="py-3 px-4 w-full placeholder-gray-400 text-white transition-shadow focus:ring-2 focus:outline-none bg-[#1e1e1e] focus:ring-[#ffef5c]"
                            />
                        </div>
                        <input
                            id="subject"
                            name="subject"
                            placeholder="Subject"
                            type="text"
                            autocomplete="subject"
                            prop:value=move || state.get().subject
                            on:input=move |ev| {
                                let subject = event_target_value(&ev);
                                state.update(|prev| prev.subject = subject);
                            }
                            class="py-3 px-4 w-full placeholder-gray-400 text-white transition-shadow focus:ring-2 focus:outline-none bg-[#1e1e1e] focus:ring-[#ffef5c]"
                        />
                        <textarea
                            id="message"
                            name="message"
                            placeholder="Your Message"
                            autocomplete="off"
                            prop:value=move || state.get().message
                            on:input=move |ev| {
                                let message = event_target_value(&ev);
                                state.update(|prev| prev.message = message);
                            }
                            rows="6"
                            class="py-3 px-4 w-full placeholder-gray-400 text-white transition-shadow focus:ring-2 focus:outline-none bg-[#1e1e1e] focus:ring-[#ffef5c]"
                        />
                        <button type="submit" class="flex justify-center items-center py-3 px-6 w-full text-lg font-semibold transition-colors bg-[#ffef5c] text-[#1e1e1e] hover:bg-[#ffef5c]/90">
                            <Show when=move || loader.get() fallback=|| "Send Message".into_any()>
                                <svg class="w-8 h-8 animate-spin fill-black" aria-hidden="true" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/>
                                </svg>
                            </Show>
                        </button>
                        <Show when=move || sent.get() fallback=|| ().into_any()>
                            <p class="text-[#ffef5c]">"Message sent successfully! I'll get back to you shortly."</p>
                        </Show>
                    </form>
                </div>
            </section>
        </div>
    }
}
