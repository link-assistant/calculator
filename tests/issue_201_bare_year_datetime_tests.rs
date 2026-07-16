//! Regression tests for issue #201: bare years in datetime subtraction.

use link_calculator::Calculator;

#[test]
fn reported_now_minus_bare_year_converts_to_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("(now - 2023) in months");

    assert!(
        result.success,
        "reported expression should succeed: {:?}",
        result.error
    );
    assert!(result.result.ends_with(" months"), "{}", result.result);
}

#[test]
fn bare_year_is_january_first_in_datetime_subtraction() {
    let mut calc = Calculator::new();

    let forward = calc.calculate_internal("2024-07-01 - 2023 in months");
    assert!(forward.success, "forward subtraction: {:?}", forward.error);
    let explicit_forward = calc.calculate_internal("2024-07-01 - 2023-01-01 in months");
    assert_eq!(forward.result, explicit_forward.result);

    let reverse = calc.calculate_internal("2023 - 2024-07-01 in months");
    assert!(reverse.success, "reverse subtraction: {:?}", reverse.error);
    let explicit_reverse = calc.calculate_internal("2023-01-01 - 2024-07-01 in months");
    assert_eq!(reverse.result, explicit_reverse.result);
}

#[test]
fn bare_year_coercion_is_limited_to_datetime_subtraction() {
    let mut calc = Calculator::new();

    let arithmetic = calc.calculate_internal("2023 - 10");
    assert!(
        arithmetic.success,
        "numeric arithmetic: {:?}",
        arithmetic.error
    );
    assert_eq!(arithmetic.result, "2013");

    let invalid_year = calc.calculate_internal("2024-07-01 - 10000");
    assert!(!invalid_year.success, "out-of-range year must stay numeric");
}
