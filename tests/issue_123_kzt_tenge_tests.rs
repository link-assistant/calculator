//! Tests for issue #123: Kazakh Tenge (KZT) conversion support.
//!
//! Issue: `5550000.00 тенге в долларах` failed to parse with error:
//! "Invalid operation: Cannot convert тенге to USD"
//!
//! Root causes:
//! 1. Russian-language name "тенге" (and all its grammatical forms) was missing
//!    from `CurrencyDatabase::parse_currency()`.
//! 2. KZT was not registered as a known currency in `CurrencyDatabase`.
//! 3. No default KZT↔USD fallback rate existed.
//! 4. KZT was not routed to CBR rate source in `plan.rs`.
//!
//! Fix:
//! 1. Added all Russian grammatical cases for "тенге" (tenge) mapping to KZT.
//! 2. Registered KZT as a known currency in `CurrencyDatabase`.
//! 3. Added default USD↔KZT fallback rate (~470 KZT per USD).
//! 4. Added KZT to CBR rate source routing in `plan.rs`.
//! 5. Added KZT to the CBR download script and worker lino fallback pairs.

use link_calculator::Calculator;

// ── Core issue: "5550000.00 тенге в долларах" should convert KZT to USD ───────

/// The exact expression from the issue report should now work.
#[test]
fn test_issue_123_tenge_in_dollars_russian() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5550000.00 тенге в долларах");
    assert!(
        result.success,
        "5550000.00 тенге в долларах should succeed (convert KZT to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// The converted value should be reasonable (~11,809 USD at ~470 KZT/USD).
#[test]
fn test_issue_123_tenge_to_usd_value_is_reasonable() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5550000.00 тенге в долларах");
    assert!(
        result.success,
        "5550000.00 тенге в долларах should succeed, got error: {:?}",
        result.error
    );
    // At default rate of ~470 KZT/USD, 5,550,000 KZT ≈ 11,808 USD
    // Allow wide range (1000–25000 USD) to account for rate changes
    let result_value: f64 = result
        .result
        .replace("USD", "")
        .replace(",", "")
        .trim()
        .parse()
        .unwrap_or(0.0);
    assert!(
        result_value > 1_000.0 && result_value < 25_000.0,
        "5,550,000 KZT should be between 1,000 and 25,000 USD at any reasonable rate, got: {}",
        result.result
    );
}

// ── KZT ISO code recognition ─────────────────────────────────────────────────

#[test]
fn test_issue_123_kzt_iso_code_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 KZT");
    assert!(
        result.success,
        "1000 KZT should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

#[test]
fn test_issue_123_kzt_to_usd_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("470 KZT in USD");
    assert!(
        result.success,
        "470 KZT in USD should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

#[test]
fn test_issue_123_usd_to_kzt_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 USD in KZT");
    assert!(
        result.success,
        "100 USD in KZT should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

// ── Russian-language tenge name recognition ──────────────────────────────────

#[test]
fn test_issue_123_russian_tenge_nominative_singular() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 тенге");
    assert!(
        result.success,
        "1 тенге should succeed (тенге is invariable in Russian), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

#[test]
fn test_issue_123_russian_tenge_recognized_as_kzt() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 тенге");
    assert!(
        result.success,
        "100 тенге should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

#[test]
fn test_issue_123_tenge_to_rub_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 тенге в рублях");
    assert!(
        result.success,
        "1000 тенге в рублях should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("RUB"),
        "Result should be in RUB, got: {}",
        result.result
    );
}

#[test]
fn test_issue_123_rub_to_tenge_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в тенге");
    assert!(
        result.success,
        "1000 рублей в тенге should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

// ── KZT arithmetic ───────────────────────────────────────────────────────────

#[test]
fn test_issue_123_kzt_addition() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 KZT + 2000 KZT");
    assert!(
        result.success,
        "1000 KZT + 2000 KZT should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("3000"),
        "Result should contain 3000, got: {}",
        result.result
    );
    assert!(
        result.result.contains("KZT"),
        "Result should be in KZT, got: {}",
        result.result
    );
}

// ── KZT via USD triangulation ────────────────────────────────────────────────

#[test]
fn test_issue_123_kzt_to_eur_via_usd_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 KZT in EUR");
    assert!(
        result.success,
        "10000 KZT in EUR should succeed (triangulate via USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

// ── parse_currency unit tests ───────────────────────────────────────────────

#[cfg(test)]
mod parse_currency_tests {
    use link_calculator::types::CurrencyDatabase;

    fn parse(s: &str) -> Option<String> {
        CurrencyDatabase::parse_currency(s)
    }

    #[test]
    fn test_parse_kzt_iso_code() {
        assert_eq!(parse("KZT"), Some("KZT".to_string()));
    }

    #[test]
    fn test_parse_kzt_lowercase() {
        assert_eq!(parse("kzt"), Some("KZT".to_string()));
    }

    #[test]
    fn test_parse_russian_tenge() {
        // "тенге" is invariable in Russian (same form for all cases)
        assert_eq!(parse("тенге"), Some("KZT".to_string()));
    }

    #[test]
    fn test_parse_english_tenge() {
        assert_eq!(parse("tenge"), Some("KZT".to_string()));
    }

    #[test]
    fn test_parse_english_tenges() {
        assert_eq!(parse("tenges"), Some("KZT".to_string()));
    }

    #[test]
    fn test_parse_kazakh_tenge() {
        // Kazakhstani/Kazakh name: теңге
        assert_eq!(parse("теңге"), Some("KZT".to_string()));
    }
}
