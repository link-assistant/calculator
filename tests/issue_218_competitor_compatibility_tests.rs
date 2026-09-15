//! Compatibility tests for examples published by other calculator projects.
//!
//! The source inventory and unsupported examples live in
//! `docs/case-studies/issue-220/competitor-audit.md`. Keeping the supported,
//! deterministic subset executable prevents compatibility from becoming an
//! unverified feature claim.

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

#[test]
fn documented_competitor_examples_remain_compatible() {
    // Each expression appears verbatim in a primary-source page linked by the audit.
    for (source, expression, expected) in [
        ("Numbat", "1920 / 16 * 9", "1080"),
        ("Numbat", "2^32", "4294967296"),
        ("Parsify", "12+5", "17"),
        ("Parsify", "2 * (3/4)", "1.5"),
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
        ("9 minus 4", "5"),
        ("9 subtract 4", "5"),
        ("9 without 4", "5"),
        ("8 times 9", "72"),
        ("8 multiplied by 9", "72"),
        ("8 mul 9", "72"),
        ("20 divided by 4", "5"),
        ("20 divide by 4", "5"),
        ("25 mod 7", "4"),
    ] {
        assert_result("word operator", expression, expected);
    }
}
