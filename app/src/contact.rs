//! Contact page — form only.
//!
//! Layout (spec §4.7):
//!   1. Page title in italic display ("contact")
//!   2. Lead paragraph
//!   3. Contact form (name / email / subject / message)
//!
//! The whoami / bio / social links that previously lived here moved to
//! /about (see app/src/about.rs / T22).

use leptos::{
    ev,
    html::{div, form, h1, h2, input, label, p, section, textarea},
    prelude::*,
};
use leptos_meta::{Title, TitleProps};

use crate::api::{ContactRequest, contact};

/// Renders the contact page.
pub fn component() -> impl IntoView {
    let state = RwSignal::new(ContactRequest::default());
    let sent = RwSignal::new(false);
    let loader = RwSignal::new(false);
    let submit = Action::new(move |data: &ContactRequest| {
        loader.set(true);
        let data = data.clone();
        async move {
            let _ = contact(data).await;
            state.set(ContactRequest::default());
            sent.set(true);
            loader.set(false);
        }
    });

    div().class("flex flex-col gap-12").child((
        Title(
            TitleProps::builder()
                .text("Contact — Alex Thola")
                .build(),
        ),
        // Page header
        section().class("flex flex-col gap-3").child((
            h1().class("font-display italic text-4xl sm:text-5xl font-medium text-ink leading-tight tracking-tight")
                .child("contact"),
            p().class("text-ink-2 text-base leading-relaxed max-w-prose")
                .child("Get in touch about Rust consulting, technical review, or speaking engagements. I read every message."),
        )),
        // Form
        section().class("flex flex-col gap-6").child((
            h2().class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
                .child("send a message"),
            form()
                .class("flex flex-col gap-4")
                .on(ev::submit, move |ev| {
                    ev.prevent_default();
                    let _ = submit.dispatch(state.get());
                })
                .child((
                    div().class("grid grid-cols-1 sm:grid-cols-2 gap-4").child((
                        labeled_input("name", "Your name", "text", "name", state, |s, v| s.name = v),
                        labeled_input("email", "Your email", "email", "email", state, |s, v| s.email = v),
                    )),
                    labeled_input("subject", "Subject", "text", "subject", state, |s, v| s.subject = v),
                    labeled_textarea("message", "Your message", state, |s, v| s.message = v),
                    leptos::html::button()
                        .attr("type", "submit")
                        .class("self-start font-mono text-xs uppercase tracking-[0.08em] bg-accent text-paper py-3 px-6 hover:opacity-90 transition-opacity cursor-pointer disabled:opacity-50")
                        .child(move || {
                            if loader.get() {
                                "sending…".to_string()
                            } else {
                                "send message".to_string()
                            }
                        }),
                    Show(
                        ShowProps::builder()
                            .when(move || sent.get())
                            .fallback(|| ())
                            .children(ToChildren::to_children(|| {
                                p().class("font-mono text-[11px] uppercase tracking-[0.08em] text-accent")
                                    .child("✓ message sent — i'll get back to you shortly.")
                            }))
                            .build(),
                    ),
                )),
        )),
    ))
}

/// Helper that renders a labeled `<input>` bound to one field of `ContactRequest`.
fn labeled_input(
    id: &'static str,
    placeholder: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    state: RwSignal<ContactRequest>,
    setter: impl Fn(&mut ContactRequest, String) + Copy + 'static,
) -> impl IntoView {
    div().class("flex flex-col gap-1").child((
        label()
            .attr("for", id)
            .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
            .child(placeholder),
        input()
            .attr("id", id)
            .attr("name", id)
            .attr("type", input_type)
            .attr("autocomplete", autocomplete)
            .attr("required", "true")
            .class("py-3 px-4 w-full bg-paper-2 border border-rule-soft text-ink placeholder-ink-4 font-sans focus:outline-none focus:border-accent transition-colors")
            .prop("value", move || match id {
                "name" => state.get().name,
                "email" => state.get().email,
                "subject" => state.get().subject,
                _ => String::new(),
            })
            .on(ev::input, move |ev| {
                let v = event_target_value(&ev);
                state.update(|s| setter(s, v));
            }),
    ))
}

fn labeled_textarea(
    id: &'static str,
    placeholder: &'static str,
    state: RwSignal<ContactRequest>,
    setter: impl Fn(&mut ContactRequest, String) + Copy + 'static,
) -> impl IntoView {
    div().class("flex flex-col gap-1").child((
        label()
            .attr("for", id)
            .class("font-mono text-[11px] uppercase tracking-[0.08em] text-ink-3")
            .child(placeholder),
        textarea()
            .attr("id", id)
            .attr("name", id)
            .attr("rows", "6")
            .attr("required", "true")
            .class("py-3 px-4 w-full bg-paper-2 border border-rule-soft text-ink placeholder-ink-4 font-sans focus:outline-none focus:border-accent transition-colors resize-y")
            .prop("value", move || state.get().message)
            .on(ev::input, move |ev| {
                let v = event_target_value(&ev);
                state.update(|s| setter(s, v));
            }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_component_signature() {
        let _: fn() -> _ = component;
    }
}
