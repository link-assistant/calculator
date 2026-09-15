//! Compatibility tests for examples published by other calculator projects.
//!
//! The source inventory and unsupported examples live in
//! `docs/case-studies/issue-220/competitor-audit.md`. Keeping the supported,
//! deterministic subset executable prevents compatibility from becoming an
//! unverified feature claim.

use link_calculator::grammar::ExpressionParser;
use link_calculator::Calculator;

fn assert_result(source: &str, expression: &str, expected: &str) {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal(expression);

    assert!(
        result.success,
        "{source} example {expression:?} failed: {:?}",
        result.error
    );
    assert_eq!(result.result, expected, "{source} example {expression:?}");
}

fn assert_approx(source: &str, expression: &str, expected: f64) {
    let mut parser = ExpressionParser::new();
    let (value, _, _) = parser
        .parse_and_evaluate(expression)
        .unwrap_or_else(|error| panic!("{source} example {expression:?} failed: {error}"));
    let actual = value
        .as_decimal()
        .unwrap_or_else(|| panic!("{source} example {expression:?} was not numeric"))
        .to_f64();

    assert!(
        (actual - expected).abs() < 1e-12,
        "{source} example {expression:?}: expected {expected}, got {actual}"
    );
}

#[test]
fn documented_competitor_examples_remain_compatible() {
    // Each expression appears verbatim in a primary-source page linked by the audit.
    for (source, expression, expected) in [
        ("Numbat", "1920 / 16 * 9", "1080"),
        ("Numbat", "2^32", "4294967296"),
        ("Parsify", "12+5", "17"),
        ("Parsify", "2*(3/4)", "1.5"),
        ("Numi", "8 times 9", "72"),
        ("Numi", "20% of $10", "2 USD"),
        ("Soulver", "30% of 700", "210"),
        ("fend", "1 + 3 * 4", "13"),
        ("fend", "16^2", "256"),
        ("fend", "5!", "120"),
        ("Wolfram|Alpha", "175lb vs 100kg", "175 lb < 100 kg"),
    ] {
        assert_result(source, expression, expected);
    }
}

#[test]
fn documented_word_operator_aliases_are_supported() {
    // Parsify documents plus/minus/times/divided-by. Numi additionally
    // documents subtract/without, multiplied-by/mul, divide, and mod.
    for (expression, expected) in [
        ("2 plus 3", "5"),
        ("2 with 3", "5"),
        ("9 minus 4", "5"),
        ("9 subtract 4", "5"),
        ("9 without 4", "5"),
        ("8 times 9", "72"),
        ("8 multiplied by 9", "72"),
        ("8 mul 9", "72"),
        ("20 divided by 4", "5"),
        ("20 divide by 4", "5"),
        ("25 mod 7", "4"),
        ("25 modulo 7", "4"),
    ] {
        assert_result("word operator", expression, expected);
    }
}

#[test]
fn documented_open_calculator_notation_remains_compatible() {
    // Numbat's syntax overview and the fend manual publish these forms.
    for (source, expression, expected) in [
        ("Numbat", "1.234e3", "1234"),
        ("Numbat", "1920 ÷ 16 × 9", "1080"),
        ("Numbat", "6 · 7", "42"),
        ("Numbat", "6 ⋅ 7", "42"),
        ("Numbat", "2**3", "8"),
        ("Numbat", "2³", "8"),
        ("Numbat", "2⁻³", "0.125"),
        ("Numbat", "mod(17, 4)", "1"),
        ("fend", "1_000_000 / 4", "250000"),
        ("fend", "1.5e-6", "0.0000015"),
        ("fend", "sqrt 16", "4"),
        ("Numi", "6 (3) = 18", "true"),
        ("math.js", "(1+2)(3+4)", "21"),
        ("math.js", "(4-1)2", "6"),
        ("math.js", "sqrt(4)(1+2)", "6"),
        ("Raycast", "square root of 625", "25"),
        ("Raycast", "2 power 10", "1024"),
    ] {
        assert_result(source, expression, expected);
    }

    assert_approx("Numbat", "2 pi", 2.0 * std::f64::consts::PI);
    assert_approx("fend", "2pi", 2.0 * std::f64::consts::PI);
    assert_approx("fend", "sqrt 2", std::f64::consts::SQRT_2);
}

#[test]
fn conventional_scientific_writing_is_supported() {
    for (expression, expected) in [
        ("10 − 3", "7"),
        ("√81", "9"),
        ("∛27", "3"),
        ("6.022e23 / 6.022e23", "1"),
        ("2x + 4 = 10", "x = 3"),
    ] {
        assert_result("conventional notation", expression, expected);
    }

    assert_approx("conventional notation", "2π", 2.0 * std::f64::consts::PI);

    let mut calculator = Calculator::new();
    let polynomial = calculator.calculate_internal("x(x - 3) = 0");
    assert!(
        polynomial.success,
        "implicit polynomial: {:?}",
        polynomial.error
    );
    assert!(polynomial.result.contains("x = 0"), "{}", polynomial.result);
    assert!(polynomial.result.contains("x = 3"), "{}", polynomial.result);
}

#[test]
fn compatibility_notation_does_not_steal_existing_grammar() {
    // Existing adjacent SI suffixes, units, dates, and strict invalid-input
    // handling must keep their established meanings.
    for (expression, expected) in [
        ("2h in minutes", "120 minutes"),
        ("2k USD", "2000 USD"),
        ("15.10.2025 + 1 day", "2025-10-16"),
    ] {
        assert_result("grammar regression", expression, expected);
    }

    let mut calculator = Calculator::new();
    let malformed_separator = calculator.calculate_internal("1__0");
    assert!(!malformed_separator.success);
}
