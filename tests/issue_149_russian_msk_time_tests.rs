//! Regression tests for issue #149: Russian "по мск" time expressions.

use link_calculator::Calculator;

#[test]
fn parses_hour_by_moscow_time() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("11 по мск");

    assert!(
        result.success,
        "11 по мск should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("11:00:00"),
        "Result should preserve the requested Moscow hour, got: {}",
        result.result
    );
    assert!(
        result.result.contains("MSK"),
        "Result should display MSK timezone, got: {}",
        result.result
    );
    assert!(
        result.lino_interpretation.contains("MSK"),
        "Links notation should consume the timezone token, got: {}",
        result.lino_interpretation
    );
}
