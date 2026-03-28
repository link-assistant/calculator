//! Tests for issue #45: `DateTime` countdown/elapsed time display.
//!
//! When a standalone `DateTime` is entered, the result should:
//! 1. Have `is_live_time = true` so the frontend auto-refreshes
//! 2. Include "Time since" or "Time until" in the steps
//! 3. Have correct links notation with parentheses

use link_calculator::Calculator;

#[test]
fn standalone_datetime_has_live_time() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("Jan 27, 9:33am UTC");
    assert!(result.success);
    assert_eq!(
        result.is_live_time,
        Some(true),
        "Standalone datetime should have is_live_time=true for countdown display"
    );
}

#[test]
fn standalone_date_has_live_time() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("Jan 27, 2026");
    assert!(result.success);
    assert_eq!(
        result.is_live_time,
        Some(true),
        "Standalone date should have is_live_time=true for countdown display"
    );
}

#[test]
fn datetime_steps_contain_time_since_or_until() {
    let mut calc = Calculator::new();
    // Use a past date to get "Time since"
    let result = calc.calculate_internal("Jan 1, 2020");
    assert!(result.success);
    let has_time_ref = result
        .steps
        .iter()
        .any(|s| s.starts_with("Time since:") || s.starts_with("Time until:"));
    assert!(
        has_time_ref,
        "Steps should contain 'Time since:' or 'Time until:', got: {:?}",
        result.steps
    );
}

#[test]
fn future_datetime_steps_contain_time_until() {
    let mut calc = Calculator::new();
    // Use a far future date
    let result = calc.calculate_internal("Dec 31, 2099");
    assert!(result.success);
    let has_time_until = result.steps.iter().any(|s| s.starts_with("Time until:"));
    assert!(
        has_time_until,
        "Future date steps should contain 'Time until:', got: {:?}",
        result.steps
    );
}

#[test]
fn datetime_lino_has_parentheses() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("Jan 27, 9:33am UTC");
    assert!(result.success);
    assert!(
        result.lino_interpretation.starts_with('(') && result.lino_interpretation.ends_with(')'),
        "Links notation should be wrapped in parentheses, got: {}",
        result.lino_interpretation
    );
}

#[test]
fn plan_standalone_datetime_is_live_time() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("Jan 27, 9:33am UTC");
    assert!(plan.success);
    assert!(
        plan.is_live_time,
        "Plan for standalone datetime should have is_live_time=true"
    );
}

#[test]
fn plan_standalone_date_is_live_time() {
    let calc = Calculator::new();
    let plan = calc.plan_internal("Dec 31, 2026");
    assert!(plan.success);
    assert!(
        plan.is_live_time,
        "Plan for standalone date should have is_live_time=true"
    );
}

#[test]
fn datetime_subtraction_not_live_time() {
    let mut calc = Calculator::new();
    // DateTime subtraction produces a Duration, not a DateTime
    let result = calc.calculate_internal("(Jan 27, 9:33am UTC) - (Jan 25, 12:51pm UTC)");
    assert!(result.success);
    // The result is a duration (not a datetime), so it should NOT be live
    // unless the expression contains a "now" reference
    assert!(
        result.is_live_time.is_none() || result.is_live_time == Some(false),
        "DateTime subtraction result (Duration) should not be live, got: {:?}",
        result.is_live_time
    );
}

#[test]
fn now_expression_still_live() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("now");
    assert!(result.success);
    assert_eq!(
        result.is_live_time,
        Some(true),
        "'now' should still have is_live_time=true"
    );
}
