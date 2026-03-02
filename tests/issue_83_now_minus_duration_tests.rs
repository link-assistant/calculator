//! Tests for issue #83: "now - 10 days" and similar expressions.
//!
//! These tests verify that datetime arithmetic with number-unit duration expressions works.

use link_calculator::Calculator;

mod issue_83_now_minus_duration {
    use super::*;

    #[test]
    fn test_now_minus_days() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 10 days");
        assert!(
            result.success,
            "now - 10 days should succeed: {:?}",
            result.error
        );
        // Result should be a datetime (10 days before now)
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "now - 10 days should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_now_plus_days() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 10 days");
        assert!(
            result.success,
            "now + 10 days should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "now + 10 days should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_now_minus_hours() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 2 hours");
        assert!(
            result.success,
            "now - 2 hours should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "now - 2 hours should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_now_plus_hours() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 2 hours");
        assert!(
            result.success,
            "now + 2 hours should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_minutes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 30 minutes");
        assert!(
            result.success,
            "now - 30 minutes should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_plus_minutes() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 30 minutes");
        assert!(
            result.success,
            "now + 30 minutes should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_weeks() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 2 weeks");
        assert!(
            result.success,
            "now - 2 weeks should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_plus_weeks() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 2 weeks");
        assert!(
            result.success,
            "now + 2 weeks should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_months() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 3 months");
        assert!(
            result.success,
            "now - 3 months should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_years() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 1 year");
        assert!(
            result.success,
            "now - 1 year should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_seconds() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 60 seconds");
        assert!(
            result.success,
            "now - 60 seconds should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_fixed_date_minus_days() {
        let mut calc = Calculator::new();
        // Use parentheses to ensure the date is parsed as a datetime, not arithmetic
        let result = calc.calculate_internal("(Jan 27, 8:00am UTC) - 10 days");
        assert!(
            result.success,
            "(Jan 27, 8:00am UTC) - 10 days should succeed: {:?}",
            result.error
        );
        // Result should be a datetime (10 days before Jan 27)
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "(Jan 27, 8:00am UTC) - 10 days should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_fixed_date_plus_days() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(Jan 1, 12:00am UTC) + 30 days");
        assert!(
            result.success,
            "(Jan 1, 12:00am UTC) + 30 days should succeed: {:?}",
            result.error
        );
        // Result should be a datetime (30 days after Jan 1)
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "(Jan 1, 12:00am UTC) + 30 days should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_duration_unit_abbreviation_day() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 1 day");
        assert!(
            result.success,
            "now - 1 day should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_duration_unit_abbreviation_hour() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 1 hour");
        assert!(
            result.success,
            "now + 1 hour should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_duration_unit_abbreviation_minute() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 1 minute");
        assert!(
            result.success,
            "now + 1 minute should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_duration_unit_abbreviation_second() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 1 second");
        assert!(
            result.success,
            "now - 1 second should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_duration_unit_abbreviation_week() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now + 1 week");
        assert!(
            result.success,
            "now + 1 week should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_minus_10_days_exact_issue() {
        // This is the exact failing input from the issue
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now - 10 days");
        assert!(
            result.success,
            "Exact issue input 'now - 10 days' should succeed: {:?}",
            result.error
        );
        // Should return a datetime, not an error
        assert!(
            result.error.is_none(),
            "Should not have an error: {:?}",
            result.error
        );
    }
}
