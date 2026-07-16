//! Regression tests for issue #199: `today` should resolve to the current
//! calendar date and participate in date arithmetic.
//!
//! See <https://github.com/link-assistant/calculator/issues/199>.

use chrono::{Duration, Utc};
use link_calculator::Calculator;

#[test]
fn today_minus_fixed_date_returns_elapsed_days() {
    let before = Utc::now().date_naive();

    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("today - 17.01.2023");

    let after = Utc::now().date_naive();
    assert!(
        result.success,
        "today - 17.01.2023 should succeed: {:?}",
        result.error
    );

    let expected_before = before
        .signed_duration_since(chrono::NaiveDate::from_ymd_opt(2023, 1, 17).unwrap())
        .num_days();
    let expected_after = after
        .signed_duration_since(chrono::NaiveDate::from_ymd_opt(2023, 1, 17).unwrap())
        .num_days();
    assert!(
        result.result == format!("{expected_before} days")
            || result.result == format!("{expected_after} days"),
        "expected elapsed whole days, got: {}",
        result.result
    );
}

#[test]
fn today_uses_the_configured_local_calendar_date() {
    let now = Utc::now();
    let utc_date = now.date_naive();
    let (offset_minutes, expected_date) = (-12 * 60..=14 * 60)
        .step_by(30)
        .find_map(|offset_minutes| {
            let local_date = (now + Duration::minutes(i64::from(offset_minutes))).date_naive();
            (local_date != utc_date).then_some((offset_minutes, local_date))
        })
        .expect("at least one supported offset must cross a UTC date boundary");

    let mut calculator = Calculator::new();
    calculator.set_timezone_offset(offset_minutes);
    let result = calculator.calculate_internal("today");

    assert!(result.success, "today should succeed: {:?}", result.error);
    assert_eq!(result.result, expected_date.format("%Y-%m-%d").to_string());
}
