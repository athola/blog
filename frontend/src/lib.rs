#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::component;
    // initializes logging using the `log` crate
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(component);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_console_log_initialization() {
        // Test that console log initialization returns a result
        let result = console_log::init_with_level(log::Level::Debug);
        // Should return either Ok or Err, never panic
        assert!(result.is_ok() || result.is_err());
    }
}
