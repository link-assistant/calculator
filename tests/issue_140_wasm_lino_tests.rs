//! Tests for issue #140: RUB/INR conversion at a specific date fails because
//! `load_rates_from_consolidated_lino` was not exported via WASM bindings.
//!
//! Root cause: The method existed only in the non-`#[wasm_bindgen]` `impl Calculator`
//! block, so in the browser the web worker's call to
//! `calculator.load_rates_from_consolidated_lino(content)` silently threw
//! `TypeError: calculator.load_rates_from_consolidated_lino is not a function`.
//! Historical .lino rates were never actually loaded, causing all dated RUB
//! conversions to fail.
//!
//! Fix: Added a `#[wasm_bindgen]`-annotated wrapper in the wasm impl block that
//! delegates to the internal implementation, making the method available in WASM.

use link_calculator::Calculator;

/// Verify that load_rates_from_consolidated_lino works end-to-end as called from
/// the web worker (returns usize, not Result).
#[test]
fn load_rates_returns_count_not_result() {
    let mut calc = Calculator::new();
    let content = "conversion:
  from INR
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2026-04-11 0.830794
    2026-04-14 0.816646";

    let count = calc.load_rates_from_consolidated_lino(content);
    assert_eq!(count, 2, "Should return count of loaded rates as usize");
}

/// Verify that after loading via the WASM-compatible method, historical conversions work.
#[test]
fn issue_140_rub_to_inr_on_april_11_2026_succeeds() {
    let mut calc = Calculator::new();

    // Simulate what the web worker does: call the WASM method with inr-rub.lino content
    let inr_rub_content = include_str!("../data/currency/inr-rub.lino");
    let count = calc.load_rates_from_consolidated_lino(inr_rub_content);
    assert!(count > 0, "inr-rub.lino should have loadable rates");

    // The original issue expression should now succeed
    let result = calc.calculate_internal("22822 рублей в рупиях на 11 апреля 2026");
    assert!(
        result.success,
        "issue #140: RUB/INR conversion on 2026-04-11 should succeed after loading .lino data. Error: {:?}",
        result.error
    );
}

/// Verify that zero is returned for invalid content (previously would panic or return Err).
#[test]
fn load_rates_returns_zero_for_empty_content() {
    let mut calc = Calculator::new();
    let count = calc.load_rates_from_consolidated_lino("");
    assert_eq!(count, 0, "Empty content should return 0");
}
