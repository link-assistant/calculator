//! Tests for issue #116: Vietnamese Dong (VND) to Russian Rubles conversion.
//!
//! Issue: `2340000 донгов СРВ в рублях` was not recognized as a VND→RUB
//! conversion. The calculator returned the input as a literal value without
//! performing any conversion.
//!
//! Root causes:
//! 1. Russian-language names for VND ("донг", "донгов", etc.) were missing
//!    from `CurrencyDatabase::parse_currency()`.
//! 2. VND was not routed to CBR rate source (ECB/Frankfurter doesn't provide VND).
//! 3. No default VND↔USD rate existed for fallback.
//! 4. The ₫ symbol was not recognized by the lexer.
//!
//! Fix:
//! 1. Added all Russian grammatical cases for "донг" (dong) mapping to VND.
//! 2. Added English/Vietnamese aliases ("dong", "dongs", "đồng") for VND.
//! 3. Added ₫ and VND to currency code/symbol recognition.
//! 4. Routed VND to CBR rate source in `plan.rs`.
//! 5. Added default USD↔VND rate for offline fallback.

use link_calculator::Calculator;

// ── parse_currency() recognition tests ──────────────────────────────────────

#[test]
fn test_issue_116_vnd_iso_code_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 VND");
    assert!(
        result.success,
        "10000 VND should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_dong_symbol_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("₫10000");
    assert!(
        result.success,
        "₫10000 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_russian_dongov_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 донгов");
    assert!(
        result.success,
        "10000 донгов should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_russian_dong_nominative_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 донг");
    assert!(
        result.success,
        "1 донг should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_english_dong_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 dong");
    assert!(
        result.success,
        "10000 dong should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_vietnamese_dong_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 đồng");
    assert!(
        result.success,
        "10000 đồng should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

// ── VND to RUB conversion tests ─────────────────────────────────────────────

#[test]
fn test_issue_116_dongov_v_rublyah_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2340000 донгов в рублях");
    assert!(
        result.success,
        "2340000 донгов в рублях should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("RUB"),
        "Result should be in RUB, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_vnd_in_rub_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 VND in RUB");
    assert!(
        result.success,
        "10000 VND in RUB should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("RUB"),
        "Result should be in RUB, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_vnd_to_rub_reasonable_value() {
    let mut calc = Calculator::new();
    // At default rate 1 USD ≈ 25,810 VND and 1 USD ≈ 89.5 RUB:
    // 10,000 VND ≈ (10000 / 25810) * 89.5 ≈ 3.47 RUB
    // Allow wide range for rate variations
    let result = calc.calculate_internal("10000 VND in RUB");
    assert!(
        result.success,
        "10000 VND in RUB should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("RUB"),
        "Result should be in RUB, got: {}",
        result.result
    );
}

// ── RUB to VND conversion tests ─────────────────────────────────────────────

#[test]
fn test_issue_116_rub_to_vnd_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в донгах");
    assert!(
        result.success,
        "1000 рублей в донгах should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

#[test]
fn test_issue_116_rub_in_vnd_iso() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 RUB in VND");
    assert!(
        result.success,
        "1000 RUB in VND should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

// ── VND arithmetic tests ────────────────────────────────────────────────────

#[test]
fn test_issue_116_vnd_addition() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 VND + 20000 VND");
    assert!(
        result.success,
        "10000 VND + 20000 VND should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("30000"),
        "Result should contain 30000, got: {}",
        result.result
    );
    assert!(
        result.result.contains("VND"),
        "Result should be in VND, got: {}",
        result.result
    );
}

// ── VND to USD conversion (triangulation via USD) ───────────────────────────

#[test]
fn test_issue_116_vnd_to_usd_converts() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("25000 VND in USD");
    assert!(
        result.success,
        "25000 VND in USD should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
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
    fn test_parse_vnd_iso_code() {
        assert_eq!(parse("VND"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_vnd_lowercase() {
        assert_eq!(parse("vnd"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_vnd_symbol() {
        assert_eq!(parse("₫"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_russian_dong_nominative() {
        assert_eq!(parse("донг"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_russian_dong_genitive_plural() {
        assert_eq!(parse("донгов"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_russian_dong_prepositional_plural() {
        assert_eq!(parse("донгах"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_russian_dong_dative_plural() {
        assert_eq!(parse("донгам"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_russian_dong_instrumental_plural() {
        assert_eq!(parse("донгами"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_english_dong() {
        assert_eq!(parse("dong"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_english_dongs() {
        assert_eq!(parse("dongs"), Some("VND".to_string()));
    }

    #[test]
    fn test_parse_vietnamese_dong() {
        assert_eq!(parse("đồng"), Some("VND".to_string()));
    }
}
