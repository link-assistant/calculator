//! Tests for unit conversion features: data sizes, mass/weight, and cryptocurrencies.
//!
//! Covers issues #55 (data sizes) and #57 (mass units and crypto conversions).

use link_calculator::Calculator;

/// Tests for data size unit conversions (issue #55).
///
/// The examples from the issue:
/// - `741 KB as mebibytes`
/// - `741 KB as MB`
/// - `741 KiB as MiB`
mod data_size_tests {
    use super::*;

    /// Helper to parse result as f64, asserting it's approximately equal to expected.
    fn assert_approx(result: &str, unit_suffix: &str, expected: f64, tolerance: f64) {
        let result_trimmed = result.trim_end_matches(unit_suffix).trim();
        let parsed: f64 = result_trimmed
            .parse()
            .unwrap_or_else(|_| panic!("Could not parse '{result_trimmed}' as f64"));
        assert!(
            (parsed - expected).abs() < tolerance,
            "Expected ~{expected} {unit_suffix}, got {result}"
        );
    }

    // --- Issue #55 examples ---

    /// `741 KB as MB` → 0.741 MB (exact within SI system)
    #[test]
    fn test_741_kb_as_mb_issue_55() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("741 KB as MB");
        assert!(result.success, "Failed: {:?}", result.error);
        // 741 KB = 741 * 1000 bytes = 741000 bytes; 741000 / 1000000 = 0.741
        assert_eq!(result.result, "0.741 MB");
    }

    /// `741 KB as mebibytes` → ~0.706863... MiB (cross-standard conversion)
    #[test]
    fn test_741_kb_as_mebibytes_issue_55() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("741 KB as mebibytes");
        assert!(result.success, "Failed: {:?}", result.error);
        // 741000 bytes / 1048576 bytes per MiB ≈ 0.706863
        assert_approx(&result.result, "MiB", 741_000.0 / 1_048_576.0, 1e-6);
    }

    /// `741 KiB as MiB` → 741/1024 ≈ 0.72363... MiB (within IEC system)
    #[test]
    fn test_741_kib_as_mib_issue_55() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("741 KiB as MiB");
        assert!(result.success, "Failed: {:?}", result.error);
        // 741 KiB = 741 * 1024 bytes = 758784 bytes; 758784 / 1048576 = 741/1024
        assert_approx(&result.result, "MiB", 741.0 / 1024.0, 1e-6);
    }

    // --- Additional data size conversion tests ---

    /// Basic byte to kilobyte conversion.
    #[test]
    fn test_1000_bytes_as_kb() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1000 B as KB");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 KB");
    }

    /// Basic kibibyte to byte conversion.
    #[test]
    fn test_1_kib_as_bytes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 KiB as B");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1024 B");
    }

    /// Gigabyte to megabyte conversion (SI).
    #[test]
    fn test_1_gb_as_mb() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 GB as MB");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1000 MB");
    }

    /// Gibibyte to mebibyte conversion (IEC).
    #[test]
    fn test_1_gib_as_mib() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 GiB as MiB");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1024 MiB");
    }

    /// Full-name unit support.
    #[test]
    fn test_full_name_kilobytes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kilobytes as megabytes");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "0.001 MB");
    }

    /// Full-name unit support for binary prefixes.
    #[test]
    fn test_full_name_kibibytes_to_mebibytes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1024 kibibytes as mebibytes");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 MiB");
    }

    /// Cross-standard: GiB to GB.
    #[test]
    fn test_1_gib_as_gb() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 GiB as GB");
        assert!(result.success, "Failed: {:?}", result.error);
        // 1 GiB = 1073741824 bytes; 1073741824 / 1000000000 GB ≈ 1.073741824
        assert_approx(
            &result.result,
            "GB",
            1_073_741_824.0 / 1_000_000_000.0,
            1e-6,
        );
    }

    /// Bit conversions.
    #[test]
    fn test_8_bits_as_byte() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("8 b as B");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 B");
    }

    /// Links notation includes the target unit.
    #[test]
    fn test_lino_for_unit_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("741 KB as MB");
        assert!(result.success, "Failed: {:?}", result.error);
        // The lino should contain "as MB"
        assert!(
            result.lino_interpretation.contains("as MB"),
            "Lino should contain 'as MB', got: {}",
            result.lino_interpretation
        );
    }

    /// Arithmetic with data size units.
    #[test]
    fn test_data_size_arithmetic_then_convert() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(500 KB + 241 KB) as MB");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "0.741 MB");
    }
}

/// Tests for mass unit conversions (issue #57).
///
/// Core mass conversions:
/// - `10 tons to kg` → 10000 kg
/// - `1 kg as pounds` → ~2.2046 lb
/// - `1000 g as kg` → 1 kg
mod mass_unit_tests {
    use super::*;

    /// Helper to parse result as f64 and check approximate equality.
    fn assert_approx(result: &str, unit_suffix: &str, expected: f64, tolerance: f64) {
        let result_trimmed = result.trim_end_matches(unit_suffix).trim();
        let parsed: f64 = result_trimmed
            .parse()
            .unwrap_or_else(|_| panic!("Could not parse '{result_trimmed}' as f64"));
        assert!(
            (parsed - expected).abs() < tolerance,
            "Expected ~{expected} {unit_suffix}, got '{result}'"
        );
    }

    /// `10 tons to kg` → 10000 kg (metric ton disambiguation).
    #[test]
    fn test_tons_to_kg_issue_57() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("10 tons to kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "10000 kg");
    }

    /// `1 tonne to kg` → 1000 kg.
    #[test]
    fn test_tonne_to_kg() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 tonne to kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1000 kg");
    }

    /// `1000 g as kg` → 1 kg.
    #[test]
    fn test_grams_to_kg() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1000 g as kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 kg");
    }

    /// `1 kg as grams` → 1000 g.
    #[test]
    fn test_kg_to_grams() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kg as grams");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1000 g");
    }

    /// `1 kg as pounds` → approximately 2.2046 lb.
    #[test]
    fn test_kg_to_pounds() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kg as pounds");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_approx(&result.result, "lb", 2.204_622_6, 1e-3);
    }

    /// `1 lb as kg` → approximately 0.4536 kg.
    #[test]
    fn test_pounds_to_kg() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 lb as kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_approx(&result.result, "kg", 0.453_592_4, 1e-3);
    }

    /// `1 kg as oz` → approximately 35.274 oz.
    #[test]
    fn test_kg_to_oz() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kg as oz");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_approx(&result.result, "oz", 35.274, 0.01);
    }

    /// `1000 kg as tonnes` → 1 t.
    #[test]
    fn test_kg_to_tonnes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1000 kg as tonnes");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_approx(&result.result, "t", 1.0, 1e-9);
    }

    /// `1 kg in kilograms` — same unit, should stay 1 kg.
    #[test]
    fn test_kg_in_kg_no_change() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kg in kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 kg");
    }

    /// `500 g + 500 g in kg` → 1 kg (arithmetic then conversion).
    #[test]
    fn test_mass_arithmetic_then_convert() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(500 g + 500 g) in kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(result.result, "1 kg");
    }

    /// Natural language: `kilograms in pounds`.
    #[test]
    fn test_kilograms_word_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("1 kilograms in pounds");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_approx(&result.result, "lb", 2.204_622_6, 1e-3);
    }
}

/// Tests for cryptocurrency conversions (issue #57).
///
/// Crypto conversions rely on exchange rates being loaded via `update_crypto_rates_from_api`.
/// These tests use a fixed mock rate (1 TON = 5.42 USD) injected via that method.
mod crypto_unit_tests {
    use super::*;

    /// Helper: sets up calculator with a mock TON/USD rate.
    fn calc_with_ton_rate() -> Calculator {
        let mut calc = Calculator::new();
        let rates_json = r#"{"TON": 5.42, "BTC": 95000.0, "ETH": 3500.0}"#;
        calc.update_crypto_rates_from_api("USD", "2026-01-26", rates_json);
        calc
    }

    /// `update_crypto_rates_from_api` returns correct count.
    #[test]
    fn test_update_crypto_rates_returns_count() {
        let mut calc = Calculator::new();
        let rates_json = r#"{"TON": 5.42, "BTC": 95000.0, "ETH": 3500.0}"#;
        let count = calc.update_crypto_rates_from_api("USD", "2026-01-26", rates_json);
        assert_eq!(count, 3);
    }

    /// `update_crypto_rates_from_api` with invalid JSON returns 0.
    #[test]
    fn test_update_crypto_rates_invalid_json() {
        let mut calc = Calculator::new();
        let count = calc.update_crypto_rates_from_api("USD", "2026-01-26", "not json");
        assert_eq!(count, 0);
    }

    /// `19 TON as USD` — ticker-based crypto conversion.
    #[test]
    fn test_ton_as_usd_ticker() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("19 TON as USD");
        assert!(result.success, "Failed: {:?}", result.error);
        // 19 * 5.42 = 102.98
        assert!(
            result.result.contains("USD"),
            "Result should contain USD, got: {}",
            result.result
        );
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 102.98).abs() < 0.01,
            "19 TON should be 102.98 USD, got {value}"
        );
    }

    /// `19 ton in usd` — the exact use case from issue #57 (lowercase, `in` keyword).
    #[test]
    fn test_ton_in_usd_lowercase_issue_57() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("19 ton in usd");
        assert!(result.success, "Failed: {:?}", result.error);
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 102.98).abs() < 0.01,
            "19 ton in usd should be 102.98 USD, got {value}"
        );
    }

    /// `19 ton to usd` — `to` keyword variant.
    #[test]
    fn test_ton_to_usd_issue_57() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("19 ton to usd");
        assert!(result.success, "Failed: {:?}", result.error);
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 102.98).abs() < 0.01,
            "19 ton to usd should be 102.98 USD, got {value}"
        );
    }

    /// `19 ton in dollars` — natural language alias for USD.
    #[test]
    fn test_ton_in_dollars_issue_57() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("19 ton in dollars");
        assert!(result.success, "Failed: {:?}", result.error);
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 102.98).abs() < 0.01,
            "19 ton in dollars should be 102.98 USD, got {value}"
        );
    }

    /// `19 toncoin in usd` — natural language crypto name.
    #[test]
    fn test_toncoin_in_usd_issue_57() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("19 toncoin in usd");
        assert!(result.success, "Failed: {:?}", result.error);
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 102.98).abs() < 0.01,
            "19 toncoin in usd should be 102.98 USD, got {value}"
        );
    }

    /// `10 tons to kg` should NOT use crypto rates — it's a mass conversion.
    #[test]
    fn test_tons_mass_not_crypto_issue_57() {
        let mut calc = calc_with_ton_rate();
        // Even with crypto rates loaded, plural `tons` → mass unit
        let result = calc.calculate_internal("10 tons to kg");
        assert!(result.success, "Failed: {:?}", result.error);
        // Mass conversion: 10 metric tons = 10000 kg
        assert_eq!(
            result.result, "10000 kg",
            "10 tons to kg should give 10000 kg (mass), got {}",
            result.result
        );
    }

    /// `1 BTC as USD` — bitcoin conversion.
    #[test]
    fn test_btc_as_usd() {
        let mut calc = calc_with_ton_rate();
        let result = calc.calculate_internal("1 BTC as USD");
        assert!(result.success, "Failed: {:?}", result.error);
        let value: f64 = result
            .result
            .trim_end_matches("USD")
            .trim()
            .parse()
            .unwrap_or(f64::NAN);
        assert!(
            (value - 95000.0).abs() < 1.0,
            "1 BTC should be 95000 USD, got {value}"
        );
    }
}

/// Tests for unit ambiguity detection and resolution (issue #104).
///
/// When a unit identifier like "ton" has multiple valid interpretations
/// (metric ton mass unit vs Toncoin cryptocurrency), the calculator should
/// surface both as alternative interpretations and resolve contextually
/// when a conversion target provides clarity.
mod unit_ambiguity_tests {
    use super::*;

    /// `19 ton` should produce two interpretations: mass (primary) and crypto (alternative).
    #[test]
    fn test_ton_standalone_has_alternatives_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 ton");
        assert!(plan.success, "Plan should succeed");
        assert_eq!(plan.lino_interpretation, "(19 t)", "Primary should be mass");
        assert!(
            plan.alternative_lino.is_some(),
            "Should have alternative interpretations"
        );
        let alts = plan.alternative_lino.unwrap();
        assert!(
            alts.iter().any(|a| a.contains("TON")),
            "Alternatives should include TON crypto: {alts:?}"
        );
    }

    /// `19 TON` (uppercase) should produce crypto as primary and mass as alternative.
    #[test]
    fn test_ton_uppercase_has_alternatives_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 TON");
        assert!(plan.success, "Plan should succeed");
        assert_eq!(
            plan.lino_interpretation, "(19 TON)",
            "Primary should be crypto for uppercase TON"
        );
        assert!(
            plan.alternative_lino.is_some(),
            "Should have alternative interpretations"
        );
        let alts = plan.alternative_lino.unwrap();
        assert!(
            alts.iter().any(|a| a.contains("(19 t)")),
            "Alternatives should include mass unit: {alts:?}"
        );
    }

    /// `19 ton in usd` should contextually resolve to crypto (TON).
    #[test]
    fn test_ton_in_usd_resolves_to_crypto_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 ton in usd");
        assert!(plan.success, "Plan should succeed");
        assert!(
            plan.lino_interpretation.contains("TON"),
            "Should resolve to TON crypto for currency conversion: {}",
            plan.lino_interpretation
        );
    }

    /// `19 ton in kg` should contextually resolve to mass (metric ton).
    #[test]
    fn test_ton_in_kg_resolves_to_mass_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 ton in kg");
        assert!(plan.success, "Plan should succeed");
        assert!(
            plan.lino_interpretation.contains("(19 t)"),
            "Should resolve to mass for mass conversion: {}",
            plan.lino_interpretation
        );
    }

    /// `19 ton in kg` should compute correct mass conversion.
    #[test]
    fn test_ton_in_kg_computes_correctly_issue_104() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("19 ton in kg");
        assert!(result.success, "Failed: {:?}", result.error);
        assert_eq!(
            result.result, "19000 kg",
            "19 metric tons should equal 19000 kg"
        );
    }

    /// `19 tons` (plural) should be unambiguous mass with no alternatives.
    #[test]
    fn test_tons_plural_no_ambiguity_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 tons");
        assert!(plan.success, "Plan should succeed");
        assert_eq!(plan.lino_interpretation, "(19 t)", "Should be mass");
        assert!(
            plan.alternative_lino.is_none(),
            "Plural 'tons' should be unambiguous: {:?}",
            plan.alternative_lino
        );
    }

    /// `19 tonne` should be unambiguous mass with no alternatives.
    #[test]
    fn test_tonne_no_ambiguity_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 tonne");
        assert!(plan.success, "Plan should succeed");
        assert_eq!(plan.lino_interpretation, "(19 t)", "Should be mass");
        assert!(
            plan.alternative_lino.is_none(),
            "'tonne' should be unambiguous: {:?}",
            plan.alternative_lino
        );
    }

    /// `19 tonnes` should be unambiguous mass with no alternatives.
    #[test]
    fn test_tonnes_no_ambiguity_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 tonnes");
        assert!(plan.success, "Plan should succeed");
        assert_eq!(plan.lino_interpretation, "(19 t)", "Should be mass");
        assert!(
            plan.alternative_lino.is_none(),
            "'tonnes' should be unambiguous: {:?}",
            plan.alternative_lino
        );
    }

    /// `19 toncoin in usd` should remain unambiguous crypto (natural language name).
    #[test]
    fn test_toncoin_natural_language_no_ambiguity_issue_104() {
        let calc = Calculator::new();
        let plan = calc.plan_internal("19 toncoin in usd");
        assert!(plan.success, "Plan should succeed");
        assert!(
            plan.lino_interpretation.contains("TON"),
            "'toncoin' should be crypto: {}",
            plan.lino_interpretation
        );
    }
}
