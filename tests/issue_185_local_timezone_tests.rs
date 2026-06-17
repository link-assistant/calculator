//! Tests for issue #185: `now` and bare times should be interpreted in the
//! user's local timezone (when known) instead of always defaulting to UTC.
//!
//! The user reported that `now-12:30` produced `0 seconds`. Two bugs were
//! involved:
//!   1. `now` and the bare time `12:30` were both treated as UTC, so the
//!      calculation never matched the user's local wall clock.
//!   2. `DateTime - DateTime` collapsed any negative difference to `0 seconds`
//!      because it went through `std::time::Duration` (which cannot be negative).
//!
//! See <https://github.com/link-assistant/calculator/issues/185>.

use link_calculator::Calculator;

/// India Standard Time, UTC+5:30, in minutes east of UTC.
const IST_MINUTES: i32 = 330;

#[test]
fn bare_time_uses_local_timezone_when_set() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    let result = calc.calculate_internal("12:30");
    assert!(result.success, "12:30 should succeed: {:?}", result.error);
    // The wall-clock reading is preserved, but now displayed in local time.
    assert!(
        result.result.contains("12:30:00"),
        "expected wall clock 12:30:00, got: {}",
        result.result
    );
    assert!(
        result.result.contains("+05:30"),
        "bare time should display the local offset, got: {}",
        result.result
    );
    assert!(
        !result.result.contains("UTC"),
        "bare time must NOT be labelled UTC when a local offset is set: {}",
        result.result
    );
}

#[test]
fn explicit_utc_is_still_honored() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    let result = calc.calculate_internal("12:30 UTC");
    assert!(result.success, "12:30 UTC should succeed: {:?}", result.error);
    assert!(
        result.result.contains("UTC"),
        "explicit UTC must remain UTC regardless of the local offset: {}",
        result.result
    );
}

#[test]
fn now_uses_local_timezone_when_set() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    let result = calc.calculate_internal("now");
    assert!(result.success, "now should succeed: {:?}", result.error);
    assert!(
        result.result.contains("current local time"),
        "now should be labelled as local time: {}",
        result.result
    );
    assert!(
        result.result.contains("+05:30"),
        "now should display the local offset: {}",
        result.result
    );
}

#[test]
fn now_defaults_to_utc_without_offset() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("now");
    assert!(result.success);
    assert!(
        result.result.contains("current UTC time"),
        "without a configured offset, now stays UTC: {}",
        result.result
    );
}

#[test]
fn time_difference_uses_wall_clock_in_local_timezone() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    // Both operands are bare times re-anchored to the same local timezone, so the
    // result is the wall-clock difference: 14:44 - 12:30 = 2h 14m.
    // Spaced form exercises the dedicated datetime-subtraction grammar path.
    let result = calc.calculate_internal("(14:44) - (12:30)");
    assert!(result.success, "subtraction should succeed: {:?}", result.error);
    assert!(
        result.result.contains("2 hours") && result.result.contains("14 minutes"),
        "expected 2 hours, 14 minutes, got: {}",
        result.result
    );
}

#[test]
fn unspaced_time_difference_uses_wall_clock() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    // Unspaced form goes through the general expression evaluator (Binary op).
    let result = calc.calculate_internal("14:44-12:30");
    assert!(result.success, "subtraction should succeed: {:?}", result.error);
    assert!(
        result.result.contains("2 hours") && result.result.contains("14 minutes"),
        "expected 2 hours, 14 minutes, got: {}",
        result.result
    );
}

#[test]
fn negative_time_difference_is_not_collapsed_to_zero() {
    // Reproduces the core of the reported "0 seconds" bug without timezones:
    // an earlier-minus-later subtraction must yield a negative duration.
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("12:00-14:00");
    assert!(result.success, "subtraction should succeed: {:?}", result.error);
    assert!(
        !result.result.contains("0 seconds"),
        "negative difference must not collapse to 0 seconds: {}",
        result.result
    );
    assert!(
        result.result.contains("2 hours") && result.result.trim_start().starts_with('-'),
        "expected a negative 2 hours, got: {}",
        result.result
    );
}

#[test]
fn issue_185_now_minus_time_succeeds() {
    // The exact expression from the issue. With a local offset configured it must
    // succeed and produce a duration rather than the bogus "0 seconds".
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);

    let result = calc.calculate_internal("now-12:30");
    assert!(result.success, "now-12:30 should succeed: {:?}", result.error);
    assert!(
        result.result.contains("second")
            || result.result.contains("minute")
            || result.result.contains("hour"),
        "now-12:30 should produce a duration: {}",
        result.result
    );
}

#[test]
fn clear_timezone_offset_restores_utc() {
    let mut calc = Calculator::new();
    calc.set_timezone_offset(IST_MINUTES);
    calc.clear_timezone_offset();

    let result = calc.calculate_internal("12:30");
    assert!(result.success);
    assert!(
        result.result.contains("UTC"),
        "after clearing the offset, bare times revert to UTC: {}",
        result.result
    );
}
