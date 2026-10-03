//! Issue #222: general expression evaluation without a consumer-side fallback.
//!
//! Exact arbitrary-precision arithmetic, standard operator precedence, and
//! errors instead of silent wrong answers. Expressions come from the
//! link-assistant/formal-ai fallback evaluator tests.

use link_calculator::Calculator;

fn eval(input: &str) -> String {
    let result = Calculator::new().calculate_internal(input);
    assert!(
        result.success,
        "{input:?} should evaluate, got error: {:?}",
        result.error
    );
    result.result
}

fn eval_err(input: &str) -> String {
    let result = Calculator::new().calculate_internal(input);
    assert!(
        !result.success,
        "{input:?} should fail, got {}",
        result.result
    );
    result.error.unwrap_or_default()
}

#[test]
fn big_integer_product_is_exact() {
    assert_eq!(
        eval("123123980921093128 * 2348023048230429324"),
        "289097944992610309590250530826085472"
    );
}

#[test]
fn repeated_big_integer_product_is_exact() {
    let expr = ["123123980921093128 * 2348023048230429324"; 9].join(" * ");
    assert_eq!(
        eval(&expr),
        "14106037729072578219732303763058131893558797911438976263759723254861297645532782853698438305271884662855522699368411530645666807157944703912711825919309276503681820752258928464177288947927128964537894542565939212004155831409388576062134381260971721670118984372679158715813314225808627206297070554572201889629074559598592"
    );
}

#[test]
fn big_powers_are_exact() {
    assert_eq!(eval("10^100"), format!("1{}", "0".repeat(100)));
    assert_eq!(eval("2^64"), "18446744073709551616");
    assert_eq!(eval("99999999999999999999 + 1"), "100000000000000000000");
}

#[test]
fn power_is_right_associative() {
    assert_eq!(eval("2^3^2"), "512");
}

#[test]
fn unary_minus_binds_looser_than_power() {
    assert_eq!(eval("-2^2"), "-4");
    assert_eq!(eval("-2²"), "-4");
    assert_eq!(eval("(-2)^2"), "4");
    assert_eq!(eval("2^-1"), "0.5");
    assert_eq!(eval("2 * -3^2"), "-18");
    assert_eq!(eval("3 - -2"), "5");
}

#[test]
fn factorial_is_exact() {
    assert_eq!(eval("25!"), "15511210043330985984000000");
    assert_eq!(eval("30!"), "265252859812191058636308480000000");
    assert_eq!(eval("factorial(5)"), "120");
    assert_eq!(eval("0!"), "1");
}

#[test]
fn out_of_range_results_are_errors_not_zero() {
    assert!(eval_err("exp(70)").contains("overflow"));
    assert!(eval_err("100001!").contains("overflow"));
    assert!(eval_err("(-1)!").contains("non-negative"));
}

#[test]
fn formal_ai_fallback_arithmetic() {
    for (input, expected) in [
        ("3 multiplied by 4", "12"),
        ("55 * 8% of 500", "2200"),
        ("8% of 500", "40"),
        ("10 % 3", "1"),
        ("7 * (3 + 4)", "49"),
        ("10 plus 20 times 3", "70"),
        ("100 - 25 % 7", "96"),
        ("1.5 + 2.5", "4"),
        ("5 / 2", "2.5"),
        ("700 / (60 + 80)", "5"),
        ("2*2+2", "6"),
        ("sqrt(16)", "4"),
    ] {
        assert_eq!(eval(input), expected, "{input}");
    }
    assert!(eval_err("1 / 0")
        .to_lowercase()
        .contains("division by zero"));
    eval_err("");
}

#[test]
fn linear_equations_in_one_unknown() {
    assert_eq!(eval("2x + 3 = 11"), "x = 4");
    assert_eq!(eval("2 * x + 3 = 11"), "x = 4");
    assert_eq!(eval("10 = y / 3 + 1"), "y = 27");
    assert_eq!(eval("x * 2 = 123"), "x = 61.5");
    assert_eq!(eval("3 * (x - 1) = 2 * (x + 4)"), "x = 11");
}

#[test]
fn results_include_a_step_trace() {
    let result = Calculator::new().calculate_internal("2 + 3 * 4");
    assert!(result.success);
    assert!(!result.steps.is_empty(), "steps should show the derivation");
    let result = Calculator::new().calculate_internal("2 * x + 3 = 11");
    assert!(
        result.steps.iter().any(|s| s.contains("linear")),
        "{:?}",
        result.steps
    );
}
