//! Tests for issue #91: UTC time display format and live time marker

use link_calculator::Calculator;

mod issue_91_format {
    use super::*;

    #[test]
    fn test_utc_time_display_format() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(
            result.success,
            "UTC time should succeed: {:?}",
            result.error
        );
        // Result should contain the label format: 'current UTC time': ...
        assert!(
            result.result.contains("'current UTC time'"),
            "UTC time should use label format, got: {}",
            result.result
        );
    }

    #[test]
    fn test_current_utc_time_display_format() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("current UTC time");
        assert!(
            result.success,
            "current UTC time should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("'current UTC time'"),
            "current UTC time should use label format, got: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_time_contains_offset() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(result.success);
        // Should contain (+00:00) for UTC
        assert!(
            result.result.contains("(+00:00)"),
            "UTC time result should contain (+00:00), got: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_time_is_live_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(result.success);
        assert_eq!(
            result.is_live_time,
            Some(true),
            "UTC time should have is_live_time=true, got: {:?}",
            result.is_live_time
        );
    }

    #[test]
    fn test_current_time_is_live_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("current time");
        assert!(result.success);
        assert_eq!(
            result.is_live_time,
            Some(true),
            "current time should have is_live_time=true"
        );
    }

    #[test]
    fn test_now_is_live_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now");
        assert!(result.success);
        assert_eq!(
            result.is_live_time,
            Some(true),
            "now should have is_live_time=true, got: {:?}",
            result.is_live_time
        );
    }

    #[test]
    fn test_static_datetime_not_live() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("Jan 1, 12:00am UTC");
        assert!(result.success);
        assert!(
            result.is_live_time.is_none() || result.is_live_time == Some(false),
            "Static datetime should NOT have is_live_time=true, got: {:?}",
            result.is_live_time
        );
    }

    #[test]
    fn test_est_time_phrase() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("EST time");
        assert!(
            result.success,
            "EST time should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("'current EST time'"),
            "EST time should use label format with EST, got: {}",
            result.result
        );
    }

    #[test]
    fn test_est_time_contains_offset() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("EST time");
        assert!(result.success);
        // EST is -05:00
        assert!(
            result.result.contains("-05:00"),
            "EST time result should contain -05:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_time_format_has_outer_parentheses() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(result.success);
        // Result should be wrapped in outer parentheses: ('current UTC time': ... UTC (+00:00))
        assert!(
            result.result.starts_with('(') && result.result.ends_with(')'),
            "UTC time result should be wrapped in parentheses, got: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_time_contains_tz_name_in_value() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(result.success);
        // The timezone name "UTC" should appear in the value portion (after the datetime)
        // Expected: ('current UTC time': 2026-03-02 20:40:13 UTC (+00:00))
        assert!(
            result.result.contains("UTC (+00:00)"),
            "UTC time result should contain 'UTC (+00:00)', got: {}",
            result.result
        );
    }

    #[test]
    fn test_est_time_contains_tz_name_in_value() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("EST time");
        assert!(result.success);
        // The timezone name "EST" should appear in the value portion
        // Expected: ('current EST time': 2026-03-02 15:40:13 EST (-05:00))
        assert!(
            result.result.contains("EST (-05:00)"),
            "EST time result should contain 'EST (-05:00)', got: {}",
            result.result
        );
    }

    #[test]
    fn test_now_display_format() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now");
        assert!(result.success);
        // "now" should use the labeled format with UTC
        assert!(
            result.result.contains("'current UTC time'"),
            "now should use 'current UTC time' label format, got: {}",
            result.result
        );
    }
}
