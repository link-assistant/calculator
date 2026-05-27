//! Regression tests for issue #164: Russian abbreviated month names in dates.
//!
//! The reported input `30 июл 2026` reached the low-level date parser only as
//! `30 июл`; the token parser treated `июл` as a custom unit because its
//! datetime detector did not know Russian abbreviated month names.

use link_calculator::{types::DateTime, Calculator};

#[test]
fn date_parser_accepts_reported_russian_abbreviation() {
    let dt = DateTime::parse("30 июл 2026").expect("date parser should accept Russian июл");

    assert_eq!(dt.to_string(), "2026-07-30");
}

#[test]
fn calculator_accepts_reported_russian_abbreviated_date() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("30 июл 2026");

    assert!(
        result.success,
        "30 июл 2026 should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "2026-07-30");
}

#[test]
fn calculator_accepts_neighboring_russian_month_forms() {
    let cases = [
        ("30 июн 2026", "2026-06-30"),
        ("30 июл 2026", "2026-07-30"),
        ("1 мая 2026", "2026-05-01"),
        ("1 сент 2026", "2026-09-01"),
    ];

    let mut calc = Calculator::new();
    for (input, expected) in cases {
        let result = calc.calculate_internal(input);
        assert!(
            result.success,
            "{input} should succeed, got error: {:?}",
            result.error
        );
        assert_eq!(result.result, expected, "{input} parsed incorrectly");
    }
}
