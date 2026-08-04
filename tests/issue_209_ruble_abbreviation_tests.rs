//! Regression tests for issue #209.
//!
//! The abbreviation "руб" (and its dotted form "руб.") is the most common way
//! to write rubles in Russian. It used to be parsed as a custom unit, so
//! `20000 руб + 25000 рублей` failed with "cannot add 'руб' and 'RUB'".

use link_calculator::Calculator;

#[test]
fn abbreviated_and_spelled_out_rubles_add_up() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("20000 руб + 120000 руб + 25000 рублей");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "165000 RUB");
}

#[test]
fn abbreviation_with_trailing_dot_is_recognized() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 руб. + 500 руб");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "1500 RUB");
}

#[test]
fn abbreviated_rubles_convert_to_other_currencies() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 руб в долларах");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert!(
        result.result.ends_with(" USD"),
        "unexpected result: {}",
        result.result
    );
}

#[test]
fn trailing_dot_also_works_for_non_currency_units() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5 кг.");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "5 kg");
}

#[test]
fn decimal_numbers_are_unaffected_by_trailing_dot_handling() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("3.5 + 1");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "4.5");
}

#[test]
fn bare_cyrillic_letter_is_still_a_variable_not_a_currency() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 р + 100 р");

    // "р" alone must not be treated as RUB, otherwise it could not be used as
    // a variable name; it stays a custom unit and still adds to itself.
    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "200 р");
}
