//! Regression tests for issue #217: Unicode spaces used as digit-grouping
//! separators were treated as token boundaries.

use link_calculator::Calculator;

#[test]
fn reported_narrow_no_break_space_expression_calculates() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("15\u{202f}847 / 17\u{202f}256");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.lino_interpretation, "(15847 / 17256)");
    assert_eq!(result.fraction.as_deref(), Some("15847/17256"));
}

#[test]
fn common_space_grouping_characters_are_supported() {
    let calculator = Calculator::new();

    for (name, separator) in [
        ("space", '\u{20}'),
        ("no-break space", '\u{a0}'),
        ("figure space", '\u{2007}'),
        ("thin space", '\u{2009}'),
        ("narrow no-break space", '\u{202f}'),
    ] {
        let expression = format!("1{separator}234{separator}567 + 1");
        let plan = calculator.plan_internal(&expression);

        assert!(plan.success, "{name} grouping failed: {:?}", plan.error);
        assert_eq!(plan.lino_interpretation, "(1234567 + 1)", "{name}");
    }
}

#[test]
fn malformed_space_grouping_is_not_silently_rewritten() {
    let calculator = Calculator::new();

    for expression in ["12 34 + 1", "1234 567 + 1", "1\u{202f}23 + 4"] {
        let plan = calculator.plan_internal(expression);
        assert!(
            !plan.success,
            "malformed grouping {expression:?} must remain invalid"
        );
    }
}
