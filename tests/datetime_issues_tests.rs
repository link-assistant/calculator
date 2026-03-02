//! Tests for date/time sub-issues: #34, #23, #68, #45
//!
//! These tests verify the features requested in issue #36.

use link_calculator::Calculator;

mod issue_34_now_keyword {
    use super::*;

    #[test]
    fn test_now_standalone() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now");
        assert!(result.success, "now should succeed: {:?}", result.error);
        // Result should be a datetime string
        assert!(
            result.result.contains('-'),
            "now should return a date: {}",
            result.result
        );
    }

    #[test]
    fn test_now_utc() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now UTC");
        assert!(result.success, "now UTC should succeed: {:?}", result.error);
        assert!(
            result.result.contains('+') || result.result.contains("UTC"),
            "now UTC should include timezone info: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_now() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC now");
        assert!(result.success, "UTC now should succeed: {:?}", result.error);
    }

    #[test]
    fn test_now_in_parentheses() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(now)");
        assert!(result.success, "(now) should succeed: {:?}", result.error);
    }

    #[test]
    fn test_now_in_subtraction() {
        let mut calc = Calculator::new();
        // Use a fixed past date so the result is always a positive duration
        let result = calc.calculate_internal("(now) - (Jan 1, 12:00am UTC)");
        assert!(
            result.success,
            "(now) - (Jan 1) should succeed: {:?}",
            result.error
        );
        // Result should be a duration
        assert!(
            result.result.contains("day") || result.result.contains("hour"),
            "Subtraction with now should produce a duration: {}",
            result.result
        );
    }

    #[test]
    fn test_datetime_minus_now_utc() {
        let mut calc = Calculator::new();
        // The exact expression from issue #34
        let result = calc.calculate_internal("(Jan 27, 8:59am UTC) - (now UTC)");
        assert!(
            result.success,
            "(Jan 27, 8:59am UTC) - (now UTC) should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_now_est() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now EST");
        assert!(result.success, "now EST should succeed: {:?}", result.error);
    }

    #[test]
    fn test_now_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("now");
        assert!(result.success);
        // Lino should wrap now in parentheses
        assert!(
            result.lino_interpretation.contains("(now)"),
            "Lino for 'now' should contain '(now)': {}",
            result.lino_interpretation
        );
    }
}

mod issue_23_until_and_natural_dates {
    use super::*;

    #[test]
    fn test_until_datetime() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("until Jan 27, 11:59pm UTC");
        assert!(result.success, "until should succeed: {:?}", result.error);
        // Result should be a duration
        assert!(
            result.result.contains("day")
                || result.result.contains("hour")
                || result.result.contains("minute")
                || result.result.contains("second"),
            "until should produce a duration: {}",
            result.result
        );
    }

    #[test]
    fn test_timezone_est() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("8:59am EST");
        assert!(
            result.success,
            "8:59am EST should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_timezone_pst() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("11:59pm PST");
        assert!(
            result.success,
            "11:59pm PST should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_timezone_cet() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2:30pm CET");
        assert!(
            result.success,
            "2:30pm CET should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_ordinal_date() {
        let mut calc = Calculator::new();
        // "January 26th" should be parsed as "January 26"
        let result = calc.calculate_internal("January 26th, 2026");
        assert!(
            result.success,
            "January 26th, 2026 should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_until_with_ordinal_and_timezone() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("until 11:59pm EST January 26th");
        assert!(
            result.success,
            "until with EST and ordinal should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_day_name_stripping() {
        let mut calc = Calculator::new();
        // "Monday, January 26" should parse after stripping day name
        let result = calc.calculate_internal("Monday, January 26th, 2026");
        assert!(
            result.success,
            "Monday, January 26th, 2026 should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_until_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("until Jan 27, 11:59pm UTC");
        assert!(result.success);
        assert!(
            result.lino_interpretation.contains("until"),
            "Lino should contain 'until': {}",
            result.lino_interpretation
        );
    }

    // Tests for the exact failing input from issue #23
    #[test]
    fn test_until_with_on_and_day_name() {
        let mut calc = Calculator::new();
        // The exact input from the issue report
        let result =
            calc.calculate_internal("until 11:59pm EST on Monday, January 26th");
        assert!(
            result.success,
            "until 11:59pm EST on Monday, January 26th should succeed: {:?}",
            result.error
        );
        // Result should be a duration (positive or negative depending on date)
        assert!(
            result.result.contains("day")
                || result.result.contains("hour")
                || result.result.contains("minute")
                || result.result.contains("second"),
            "until should produce a duration: {}",
            result.result
        );
    }

    #[test]
    fn test_time_with_on_and_day_name() {
        let mut calc = Calculator::new();
        // The second form from the issue: standalone time+date with "on" and day name
        // Previously returned "11" (wrong), should now return a datetime
        let result =
            calc.calculate_internal("11:59pm EST on Monday, January 26th");
        assert!(
            result.success,
            "11:59pm EST on Monday, January 26th should succeed: {:?}",
            result.error
        );
        // Result should be a datetime, not the number 11
        assert_ne!(
            result.result, "11",
            "Result should not be just the number 11"
        );
        assert!(
            result.result.contains('-') || result.result.contains(':'),
            "Result should be a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_time_with_on_preposition() {
        let mut calc = Calculator::new();
        // "on" preposition before date without day name
        let result = calc.calculate_internal("11:59pm EST on January 26");
        assert!(
            result.success,
            "11:59pm EST on January 26 should succeed: {:?}",
            result.error
        );
        assert_ne!(result.result, "11", "Result should not be the number 11");
    }

    #[test]
    fn test_time_with_timezone_and_ordinal_date() {
        let mut calc = Calculator::new();
        // Time with timezone and ordinal date (no "on")
        let result = calc.calculate_internal("11:59pm EST January 26th");
        assert!(
            result.success,
            "11:59pm EST January 26th should succeed: {:?}",
            result.error
        );
        assert_ne!(result.result, "11", "Result should not be the number 11");
    }

    #[test]
    fn test_time_with_on_day_name_lino_is_datetime() {
        let mut calc = Calculator::new();
        let result =
            calc.calculate_internal("11:59pm EST on Monday, January 26th");
        assert!(result.success);
        // Lino should be a datetime wrapped in parens, not just "11"
        assert!(
            result.lino_interpretation.starts_with('(')
                && result.lino_interpretation.ends_with(')'),
            "Lino should be datetime in parens: '{}'",
            result.lino_interpretation
        );
        assert_ne!(
            result.lino_interpretation, "(11)",
            "Lino should not be just '(11)'"
        );
    }
}

mod issue_68_utc_time {
    use super::*;

    #[test]
    fn test_utc_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("UTC time");
        assert!(
            result.success,
            "UTC time should succeed: {:?}",
            result.error
        );
        // Should return current UTC datetime
        assert!(
            result.result.contains('-'),
            "UTC time should return a datetime: {}",
            result.result
        );
    }

    #[test]
    fn test_time_utc() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("time UTC");
        assert!(
            result.success,
            "time UTC should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_current_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("current time");
        assert!(
            result.success,
            "current time should succeed: {:?}",
            result.error
        );
    }

    #[test]
    fn test_current_utc_time() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("current UTC time");
        assert!(
            result.success,
            "current UTC time should succeed: {:?}",
            result.error
        );
    }
}

mod issue_45_datetime_display {
    use super::*;

    #[test]
    fn test_standalone_datetime_shows_elapsed() {
        let mut calc = Calculator::new();
        // Use a past date
        let result = calc.calculate_internal("Jan 1, 12:00am UTC");
        assert!(result.success);
        // Steps should contain "Time since" or "Time until"
        let steps_text = result.steps.join("\n");
        assert!(
            steps_text.contains("Time since") || steps_text.contains("Time until"),
            "Standalone datetime should show elapsed/remaining time in steps: {:?}",
            result.steps
        );
    }

    #[test]
    fn test_standalone_datetime_lino_wrapped() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("Jan 27, 9:33am UTC");
        assert!(result.success);
        // Lino should wrap datetime in parentheses
        let lino = &result.lino_interpretation;
        assert!(
            lino.starts_with('(') && lino.ends_with(')'),
            "DateTime lino should be wrapped in parens: '{lino}'"
        );
    }

    #[test]
    fn test_datetime_subtraction_still_works() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
        assert!(result.success);
        assert!(
            result.result.contains("day"),
            "DateTime subtraction should still work: {}",
            result.result
        );
    }

    #[test]
    fn test_natural_date_shows_elapsed() {
        let mut calc = Calculator::new();
        // Note: ISO format "2026-01-01" is parsed as arithmetic (2026-1-1=2024)
        // by the lexer, so we use natural date format instead.
        let result = calc.calculate_internal("January 1, 2026");
        assert!(result.success);
        let steps_text = result.steps.join("\n");
        assert!(
            steps_text.contains("Time since") || steps_text.contains("Time until"),
            "Natural date should show elapsed/remaining: {:?}",
            result.steps
        );
    }
}
