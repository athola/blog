//! This crate provides the WebAssembly (WASM) entry point for the frontend
//! of the Leptos application.
//!
//! It is responsible for hydrating the server-rendered HTML, enabling client-side
//! interactivity, and initializing client-side logging.

#[wasm_bindgen::prelude::wasm_bindgen]
/// Hydrates the Leptos application on the client-side.
///
/// This function is the WASM entry point. It initializes `console_log` and
/// `console_error_panic_hook` for debugging, then mounts the main Leptos
/// component (`app::component`) to hydrate the DOM.
pub fn hydrate() {
    use app::component;
    // Initialize console logging for client-side debugging.
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(component);
}

#[cfg(test)]
mod tests {
    #[test]
    /// Smoke-test that `console_log` initialization is callable in tests.
    ///
    /// This may return `Err` if a logger was already installed by another test;
    /// we only require that this call does not panic.
    fn test_console_log_initialization() {
        let _ = console_log::init_with_level(log::Level::Debug);
    }
}
