//! WebAssembly bindings for the calculator.

use wasm_bindgen::prelude::*;

/// Initializes the WASM module. Call this once before using other functions.
#[wasm_bindgen(start)]
pub fn wasm_init() {
    // Set up better panic messages in WASM
    console_error_panic_hook::set_once();
}

/// Returns the current version of the calculator library.
#[wasm_bindgen]
pub fn get_version() -> String {
    crate::VERSION.to_string()
}

/// A simple health check function.
#[wasm_bindgen]
pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
