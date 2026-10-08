//! Regression tests for silently misread grouping and decimal separators (#233).

use link_calculator::grammar::{Lexer, TokenKind};
use link_calculator::Calculator;

fn assert_result(input: &str, expected: &str) {
    let result = Calculator::new().calculate_internal(input);
    assert!(result.success, "{input:?}: {:?}", result.error);
    assert_eq!(result.result, expected, "{input:?}");
}

#[test]
fn reported_grouped_operands_are_not_decimal_fallbacks() {
    for (input, expected) in [
        ("3,000 minus 12", "2988"),
        ("1,000 divided by 200", "5"),
        ("100,000 + 200,000", "300000"),
        ("56.7% of 1,234 participants", "699.678 participants"),
    ] {
        assert_result(input, expected);
    }
}

#[test]
fn grouping_is_resolved_by_the_lexer() {
    let tokens = Lexer::new("3,000 minus 12").tokenize().unwrap();
    assert!(matches!(&tokens[0].kind, TokenKind::Number(n) if n == "3000"));
    assert_eq!(tokens[0].text, "3,000");
    assert_eq!((tokens[0].start, tokens[0].end), (0, 5));
}

#[test]
fn locale_forms_have_deterministic_values() {
    for (input, expected) in [
        ("1,234.5", "1234.5"),
        ("1.234,5", "1234.5"),
        ("1 234,5", "1234.5"),
        ("1'234.5", "1234.5"),
        ("1’234.5", "1234.5"),
        ("12,34,567", "1234567"),
        ("12,34,567.89", "1234567.89"),
        ("1,234", "1234"),
        ("1,1", "11"),
        ("1,2", "1.2"),
        ("12,3 + 4,5", "16.8"),
        ("1,234 + 1.5", "1235.5"),
        ("1,234e-2", "12.34"),
        ("1,5e2", "150"),
        ("-1,234.5", "-1234.5"),
        ("1.000 000", "1"),
        ("123_456\u{2009}789", "123456789"),
    ] {
        assert_result(input, expected);
    }
}

#[test]
fn function_commas_are_always_argument_separators() {
    for (input, expected) in [
        ("max(1,2)", "2"),
        ("max(1,1)", "1"),
        ("max(1,234)", "234"),
        ("min(12,34,567)", "12"),
        ("pow(2,3)", "8"),
        ("max(1, min(2,3))", "2"),
        ("round(1'234.5)", "1235"),
        ("max(1,2) + 3,000", "3002"),
    ] {
        assert_result(input, expected);
    }
    assert!(!Calculator::new().calculate_internal("round(1,234)").success);
    let integral = Calculator::new().calculate_internal("integrate(x^2,x,0,3)");
    assert!(integral.success, "{:?}", integral.error);
    assert!((integral.result.parse::<f64>().unwrap() - 9.0).abs() < 1e-10);
}

#[test]
fn malformed_groups_are_rejected() {
    for input in ["1,23,4567", "1'23", "1 23,4", "1,234,", "1,,234"] {
        let result = Calculator::new().calculate_internal(input);
        assert!(
            !result.success,
            "{input:?} was accepted as {}",
            result.result
        );
    }
}

#[test]
fn reported_decimal_comma_uncertainty_is_preserved() {
    assert_result("12,3 ± 4,5", "12.3 ± 4.5");
    assert_result("12.3 ± 4.5", "12.3 ± 4.5");
    assert!(!Calculator::new().calculate_internal("12,3 ± -4,5").success);
    assert_result("-(12.3 ± 4.5)", "-12.3 ± 4.5");
    assert_result("12.3 kg ± 4.5 kg", "12.3 ± 4.5 kg");
    assert!(
        !Calculator::new()
            .calculate_internal("12.3 kg ± 4.5 g")
            .success
    );
    assert!(
        !Calculator::new()
            .calculate_internal("(12.3 ± 4.5) + 1")
            .success
    );
}

#[test]
fn german_billion_abbreviation_and_compound_unit_are_preserved() {
    assert_result("1,08 Mrd. km/h", "1080000000 km/h");
    assert_result("1,08 Mio. participants", "1080000 participants");
}

#[test]
fn compound_units_are_not_silently_discarded_by_unsupported_algebra() {
    assert_result("2 km/h * 2", "4 km/h");
    assert_result("2 * 2 km/h", "4 km/h");
    assert_result("4 km/h / 2", "2 km/h");
    assert_result("4 km/h / 2 km/h", "2");
    assert_result("2 km/h + 1 km/h", "3 km/h");
    for input in [
        "5 kg * 9.8 m/s^2",
        "1 kg * (299792458 m/s)^2",
        "5 kg * 2 km/h",
        "1 / 2 km/h",
        "2 km/h / 2 kg",
        "sqrt(9 km/h)",
    ] {
        assert!(
            !Calculator::new().calculate_internal(input).success,
            "{input}"
        );
    }
}

#[test]
fn large_word_scales_report_overflow_without_panicking() {
    assert!(
        !Calculator::new()
            .calculate_internal("100000000000000000000 Mrd.")
            .success
    );
}

#[test]
fn spaces_in_dates_and_time_components_remain_token_boundaries() {
    assert_result("Jan 17 2027", "2027-01-17");
    assert_result("17 Jan 2027", "2027-01-17");
    assert_result("15.10.2025 + 1 day", "2025-10-16");
    assert_result("Jan 17,2027", "2027-01-17");
}

#[test]
fn radix_fractions_are_not_mistaken_for_locale_grouped_custom_units() {
    assert_result("0b100.001", "4.125");
    assert_result("0o100.001", "64.001953125");
    assert_result("0x100.001", "256.000244140625");
    for (input, expected) in [
        ("0b1.", "1"),
        ("0o1.", "1"),
        ("0x1.", "1"),
        ("0b1e10", "4"),
        ("0o1e10", "16777216"),
        ("0b1e-110", "0.015625"),
    ] {
        assert_result(input, expected);
    }
}
