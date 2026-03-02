//! Tests for issue #75: Russian language currency conversion expressions.
//!
//! Issue: `1000 рублей в долларах` was not recognized as a conversion to USD.
//! The expression was treated as `1000 RUB` (literal value only) instead of
//! converting to USD.
//!
//! Root cause 1: The Russian preposition "в" (meaning "in/into") was not
//! recognized as the `In` keyword in the lexer.
//!
//! Root cause 2: Russian grammatical forms of currency names like "долларах"
//! (locative case of "доллар") and other cases were not in the currency parser.
//!
//! Fix:
//! 1. Added "в" as a Russian-language alias for the `In` keyword in the lexer.
//! 2. Added all Russian grammatical cases for USD (доллар/доллары), EUR (евро),
//!    GBP (фунт), CNY (юань), and JPY (иена) to `parse_currency()`.

use link_calculator::Calculator;

// ── Core issue: "1000 рублей в долларах" should convert RUB to USD ────────────

/// The exact expression from the issue report should now work.
#[test]
fn test_issue_75_rub_in_dollars_russian() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в долларах");
    assert!(
        result.success,
        "1000 рублей в долларах should succeed (convert RUB to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// The converted value should be reasonable (around 11 USD at ~89.5 RUB/USD).
#[test]
fn test_issue_75_rub_to_usd_value_is_reasonable() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в долларах");
    assert!(
        result.success,
        "1000 рублей в долларах should succeed, got error: {:?}",
        result.error
    );
    // At default rate of ~89.5 RUB/USD, 1000 RUB ≈ 11.17 USD
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

// ── Russian preposition "в" as "in" keyword ───────────────────────────────────

/// Russian "в" should work as the unit conversion keyword (like "in").
#[test]
fn test_issue_75_russian_v_as_in_keyword() {
    let mut calc = Calculator::new();
    // "1000 RUB в USD" should be identical to "1000 RUB in USD"
    let result_russian = calc.calculate_internal("1000 RUB в USD");
    let result_english = calc.calculate_internal("1000 RUB in USD");

    assert!(
        result_russian.success,
        "1000 RUB в USD should succeed, got error: {:?}",
        result_russian.error
    );
    assert!(
        result_english.success,
        "1000 RUB in USD should succeed, got error: {:?}",
        result_english.error
    );
    assert_eq!(
        result_russian.result, result_english.result,
        "Russian 'в' and English 'in' should produce the same result"
    );
}

// ── Russian forms of "dollar" (USD) ─────────────────────────────────────────

/// Russian nominative singular "доллар" should be recognized as USD.
#[test]
fn test_issue_75_russian_dollar_nominative() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 RUB в доллар");
    assert!(
        result.success,
        "1000 RUB в доллар should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Russian genitive plural "долларов" (as in "конвертировать в долларов").
#[test]
fn test_issue_75_russian_dollar_genitive_plural() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в долларов");
    assert!(
        result.success,
        "1000 рублей в долларов should succeed (convert to USD), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Russian prepositional plural "долларах" (the most natural form in "в долларах").
#[test]
fn test_issue_75_russian_dollar_prepositional_plural() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("500 RUB в долларах");
    assert!(
        result.success,
        "500 RUB в долларах should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

// ── Russian forms of "euro" (EUR) ─────────────────────────────────────────────

/// Russian "евро" (indeclinable word) should be recognized as EUR.
#[test]
fn test_issue_75_russian_evro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в евро");
    assert!(
        result.success,
        "1000 рублей в евро should succeed (convert RUB to EUR), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("EUR"), "Result should be in EUR");
}

/// Reverse: EUR to RUB using Russian.
#[test]
fn test_issue_75_russian_evro_to_rub() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 евро в рублях");
    assert!(
        result.success,
        "100 евро в рублях should succeed (convert EUR to RUB), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("RUB"), "Result should be in RUB");
}

// ── Russian forms of "pound" (GBP) ───────────────────────────────────────────

/// Russian "фунт" should be recognized as GBP.
#[test]
fn test_issue_75_russian_funt() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в фунтах");
    assert!(
        result.success,
        "1000 рублей в фунтах should succeed (convert RUB to GBP), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("GBP"), "Result should be in GBP");
}

// ── Russian forms of "yuan" (CNY) ─────────────────────────────────────────────

/// Russian "юань" should be recognized as CNY.
#[test]
fn test_issue_75_russian_yuan() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в юанях");
    assert!(
        result.success,
        "1000 рублей в юанях should succeed (convert RUB to CNY), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("CNY"), "Result should be in CNY");
}

// ── Russian forms of "yen" (JPY) ─────────────────────────────────────────────

/// Russian "иена" should be recognized as JPY.
#[test]
fn test_issue_75_russian_yen() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в иенах");
    assert!(
        result.success,
        "1000 рублей в иенах should succeed (convert RUB to JPY), got error: {:?}",
        result.error
    );
    assert!(result.result.contains("JPY"), "Result should be in JPY");
}

// ── Mixed Russian and English ─────────────────────────────────────────────────

/// Mix of Russian currency name with English conversion keyword should work.
#[test]
fn test_issue_75_russian_currency_with_english_in() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей in USD");
    assert!(
        result.success,
        "1000 рублей in USD should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Russian currency name with "as" conversion keyword should work.
#[test]
fn test_issue_75_russian_currency_with_as() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей as USD");
    assert!(
        result.success,
        "1000 рублей as USD should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}

/// Russian ruble amount with Russian "в" and ISO code target should work.
#[test]
fn test_issue_75_rub_v_usd_iso_code() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 рублей в USD");
    assert!(
        result.success,
        "1000 рублей в USD should succeed, got error: {:?}",
        result.error
    );
    assert!(result.result.contains("USD"), "Result should be in USD");
}
