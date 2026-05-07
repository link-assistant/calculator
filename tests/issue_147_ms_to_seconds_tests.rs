//! Tests for issue #147: Unrecognized input: `300000 ms in seconds`.
//!
//! Root cause 1: `parse_unit_for_conversion` in `token_parser.rs` never tried
//! `DurationUnit::parse()`, so time unit names like "seconds", "ms", "minutes"
//! were not recognized as valid conversion targets after "as"/"in"/"to".
//!
//! Root cause 2: `convert_to_unit_at_date` in `value/mod.rs` had no
//! Duration→Duration match arm, so even if the parse succeeded the conversion
//! would fall through to the catch-all error.
//!
//! Fix: Added `DurationUnit::parse()` check in `parse_unit_for_conversion` and
//! a `(Unit::Duration(from), Unit::Duration(to))` arm in `convert_to_unit_at_date`.

use link_calculator::Calculator;

/// The exact expression from the issue report must succeed and return 300 seconds.
#[test]
fn test_300000_ms_in_seconds() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("300000 ms in seconds");
    assert!(
        result.success,
        "300000 ms in seconds should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("300"),
        "300000 ms = 300 seconds, got: {}",
        result.result
    );
}

/// "in" keyword variant using short unit names.
#[test]
fn test_ms_to_s_short() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 ms in s");
    assert!(
        result.success,
        "1000 ms in s should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains('1'),
        "1000 ms = 1 second, got: {}",
        result.result
    );
}

/// "as" keyword variant.
#[test]
fn test_ms_as_seconds() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5000 ms as seconds");
    assert!(
        result.success,
        "5000 ms as seconds should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains('5'),
        "5000 ms = 5 seconds, got: {}",
        result.result
    );
}

/// "to" keyword variant.
#[test]
fn test_ms_to_seconds_keyword() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2000 ms to seconds");
    assert!(
        result.success,
        "2000 ms to seconds should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains('2'),
        "2000 ms = 2 seconds, got: {}",
        result.result
    );
}

/// Minutes to seconds conversion.
#[test]
fn test_minutes_to_seconds() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5 minutes in seconds");
    assert!(
        result.success,
        "5 minutes in seconds should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("300"),
        "5 minutes = 300 seconds, got: {}",
        result.result
    );
}

/// Hours to minutes conversion.
#[test]
fn test_hours_to_minutes() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2 hours in minutes");
    assert!(
        result.success,
        "2 hours in minutes should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("120"),
        "2 hours = 120 minutes, got: {}",
        result.result
    );
}

/// Seconds to milliseconds (reverse direction).
#[test]
fn test_seconds_to_ms() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 second in ms");
    assert!(
        result.success,
        "1 second in ms should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("1000"),
        "1 second = 1000 ms, got: {}",
        result.result
    );
}

/// Hours to seconds conversion.
#[test]
fn test_hours_to_seconds() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 hour in seconds");
    assert!(
        result.success,
        "1 hour in seconds should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("3600"),
        "1 hour = 3600 seconds, got: {}",
        result.result
    );
}
