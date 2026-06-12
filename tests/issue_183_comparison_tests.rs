//! Regression tests for issue #183 and its comparison syntax sub-issues.
//!
//! The calculator should recognize explicit ordering operators and natural
//! comparison forms instead of treating them as unrecognized input.

use link_calculator::Calculator;

fn calculate(input: &str) -> link_calculator::CalculationResult {
    Calculator::new().calculate_internal(input)
}

#[test]
fn issue_181_less_than_operator_compares_values() {
    let result = calculate("1781293631682 < 1781299013388");

    assert!(
        result.success,
        "less-than comparison should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "true");
    assert_eq!(
        result.lino_interpretation,
        "(1781293631682 < 1781299013388)"
    );
}

#[test]
fn issue_180_greater_than_operator_compares_values() {
    let result = calculate("1781293631682 > 1781299013388");

    assert!(
        result.success,
        "greater-than comparison should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "false");
    assert_eq!(
        result.lino_interpretation,
        "(1781293631682 > 1781299013388)"
    );
}

#[test]
fn issue_179_compare_keyword_reports_ordering() {
    let result = calculate("compare 1781293631682 and 1781299013388");

    assert!(
        result.success,
        "compare keyword should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "1781293631682 < 1781299013388",
        "generic comparison should report the relation, got steps: {:?}",
        result.steps
    );
    assert_eq!(
        result.lino_interpretation,
        "(compare 1781293631682 1781299013388)"
    );
}

#[test]
fn issue_178_vs_keyword_reports_ordering() {
    let result = calculate("1781293631682 vs 1781299013388");

    assert!(
        result.success,
        "vs comparison should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1781293631682 < 1781299013388");
    assert_eq!(
        result.lino_interpretation,
        "(compare 1781293631682 1781299013388)"
    );
}

#[test]
fn comparison_operators_cover_common_assertions() {
    for (input, expected) in [
        ("2 <= 2", "true"),
        ("2 >= 3", "false"),
        ("2 != 3", "true"),
        ("2 == 2", "true"),
        ("2 kg == 2000 g", "true"),
        ("2 kg != 2000 g", "false"),
    ] {
        let result = calculate(input);
        assert!(
            result.success,
            "{input} should succeed, got error: {:?}",
            result.error
        );
        assert_eq!(result.result, expected, "{input}");
    }
}

#[test]
fn comparisons_work_after_arithmetic_precedence() {
    let result = calculate("1 + 2 * 3 > 6");

    assert!(
        result.success,
        "arithmetic comparison should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "true");
    assert_eq!(result.lino_interpretation, "((1 + (2 * 3)) > 6)");
}

#[test]
fn comparisons_normalize_compatible_units() {
    let greater = calculate("2 kg > 1500 g");
    assert!(
        greater.success,
        "mass comparison should succeed, got error: {:?}",
        greater.error
    );
    assert_eq!(greater.result, "true");

    let generic = calculate("compare 3 days and 72 hours");
    assert!(
        generic.success,
        "duration comparison should succeed, got error: {:?}",
        generic.error
    );
    assert_eq!(generic.result, "3 days = 72 hours");
}
