//! Regression tests for issue #152: Russian "HH:MM по МСК" time expressions.

use link_calculator::Calculator;

fn assert_parses_as_msk_time(input: &str, expected_time: &str) {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal(input);

    assert!(result.success, "{input} should succeed: {:?}", result.error);
    assert!(
        result.result.contains(expected_time),
        "{input} should preserve the requested Moscow time, got: {}",
        result.result
    );
    assert!(
        result.result.contains("MSK"),
        "{input} should display MSK timezone, got: {}",
        result.result
    );
    assert!(
        result.lino_interpretation.contains("MSK"),
        "{input} should consume and normalize the timezone token, got: {}",
        result.lino_interpretation
    );
}

#[test]
fn parses_colon_time_by_moscow_time() {
    assert_parses_as_msk_time("11:00 по МСК", "11:00:00");
}

#[test]
fn parses_colon_time_by_moscow_time_with_lowercase_cyrillic_timezone() {
    assert_parses_as_msk_time("11:00 по мск", "11:00:00");
}

#[test]
fn parses_colon_time_by_moscow_time_with_latin_timezone() {
    assert_parses_as_msk_time("11:30 по MSK", "11:30:00");
}
