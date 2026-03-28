//! Tests for timezone conversion support (issue #115).
//!
//! Verifies that:
//! - Named timezones are recognized in time expressions (e.g., "6 PM MSK")
//! - Timezone conversion with "as" works (e.g., "6 PM GMT as MSK")
//! - Common typo "GTM" is handled as "GMT"
//! - Various timezone abbreviations are supported

use link_calculator::Calculator;

mod timezone_parsing {
    use super::*;

    #[test]
    fn test_time_with_gmt() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT");
        assert!(
            result.success,
            "6 PM GMT should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("18:00:00"),
            "6 PM should be 18:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("GMT"),
            "Should display GMT timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_time_with_est() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM EST");
        assert!(
            result.success,
            "6 PM EST should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("18:00:00"),
            "6 PM should be 18:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("EST"),
            "Should display EST timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_time_with_msk() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM MSK");
        assert!(
            result.success,
            "6 PM MSK should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("18:00:00"),
            "6 PM should be 18:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("MSK"),
            "Should display MSK timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_time_with_jst() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("3 AM JST");
        assert!(
            result.success,
            "3 AM JST should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("03:00:00"),
            "3 AM should be 03:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("JST"),
            "Should display JST timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_gtm_typo_treated_as_gmt() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GTM");
        assert!(
            result.success,
            "6 PM GTM should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("18:00:00"),
            "6 PM should be 18:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("GMT"),
            "GTM should be normalized to GMT: {}",
            result.result
        );
    }

    #[test]
    fn test_time_without_timezone() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6:00 PM");
        assert!(result.success, "6:00 PM should succeed: {:?}", result.error);
        assert!(
            result.result.contains("18:00:00"),
            "6 PM should be 18:00:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_am_time_with_timezone() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("9 AM PST");
        assert!(
            result.success,
            "9 AM PST should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("09:00:00"),
            "9 AM should be 09:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("PST"),
            "Should display PST timezone: {}",
            result.result
        );
    }
}

mod timezone_conversion {
    use super::*;

    #[test]
    fn test_gmt_to_msk_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT as MSK");
        assert!(
            result.success,
            "6 PM GMT as MSK should succeed: {:?}",
            result.error
        );
        // 6 PM GMT = 9 PM MSK (UTC+3)
        assert!(
            result.result.contains("21:00:00"),
            "6 PM GMT as MSK should be 21:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("MSK"),
            "Result should show MSK timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_gtm_to_msk_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GTM as MSK");
        assert!(
            result.success,
            "6 PM GTM as MSK should succeed: {:?}",
            result.error
        );
        // GTM → GMT, so 6 PM GMT = 9 PM MSK
        assert!(
            result.result.contains("21:00:00"),
            "6 PM GTM as MSK should be 21:00:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_est_to_pst_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("3 PM EST as PST");
        assert!(
            result.success,
            "3 PM EST as PST should succeed: {:?}",
            result.error
        );
        // 3 PM EST (UTC-5) = 12 PM PST (UTC-8)
        assert!(
            result.result.contains("12:00:00"),
            "3 PM EST as PST should be 12:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("PST"),
            "Result should show PST timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_msk_to_jst_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("12 PM MSK as JST");
        assert!(
            result.success,
            "12 PM MSK as JST should succeed: {:?}",
            result.error
        );
        // 12 PM MSK (UTC+3) = 6 PM JST (UTC+9)
        assert!(
            result.result.contains("18:00:00"),
            "12 PM MSK as JST should be 18:00:00, got: {}",
            result.result
        );
        assert!(
            result.result.contains("JST"),
            "Result should show JST timezone: {}",
            result.result
        );
    }

    #[test]
    fn test_utc_to_est_conversion() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("10 AM UTC as EST");
        assert!(
            result.success,
            "10 AM UTC as EST should succeed: {:?}",
            result.error
        );
        // 10 AM UTC = 5 AM EST (UTC-5)
        assert!(
            result.result.contains("05:00:00"),
            "10 AM UTC as EST should be 05:00:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_timezone_conversion_with_in_keyword() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT in MSK");
        assert!(
            result.success,
            "6 PM GMT in MSK should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("21:00:00"),
            "6 PM GMT in MSK should be 21:00:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_timezone_conversion_with_to_keyword() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT to MSK");
        assert!(
            result.success,
            "6 PM GMT to MSK should succeed: {:?}",
            result.error
        );
        assert!(
            result.result.contains("21:00:00"),
            "6 PM GMT to MSK should be 21:00:00, got: {}",
            result.result
        );
    }

    #[test]
    fn test_conversion_across_midnight() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("11 PM EST as GMT");
        assert!(
            result.success,
            "11 PM EST as GMT should succeed: {:?}",
            result.error
        );
        // 11 PM EST (UTC-5) = 4 AM GMT next day
        assert!(
            result.result.contains("04:00:00"),
            "11 PM EST as GMT should be 04:00:00, got: {}",
            result.result
        );
    }
}

mod timezone_abbreviations {
    use super::*;

    #[test]
    fn test_european_timezones() {
        let mut calc = Calculator::new();
        for tz in ["CET", "CEST", "EET", "EEST", "WET", "BST"] {
            let expr = format!("12 PM {tz}");
            let result = calc.calculate_internal(&expr);
            assert!(
                result.success,
                "12 PM {tz} should succeed: {:?}",
                result.error
            );
            assert!(
                result.result.contains("12:00:00"),
                "12 PM {tz} should show 12:00:00, got: {}",
                result.result
            );
        }
    }

    #[test]
    fn test_asian_timezones() {
        let mut calc = Calculator::new();
        for tz in ["JST", "KST", "SGT", "HKT", "ICT"] {
            let expr = format!("3 PM {tz}");
            let result = calc.calculate_internal(&expr);
            assert!(
                result.success,
                "3 PM {tz} should succeed: {:?}",
                result.error
            );
            assert!(
                result.result.contains("15:00:00"),
                "3 PM {tz} should show 15:00:00, got: {}",
                result.result
            );
        }
    }

    #[test]
    fn test_australian_timezones() {
        let mut calc = Calculator::new();
        for tz in ["AEST", "AEDT", "AWST"] {
            let expr = format!("8 AM {tz}");
            let result = calc.calculate_internal(&expr);
            assert!(
                result.success,
                "8 AM {tz} should succeed: {:?}",
                result.error
            );
            assert!(
                result.result.contains("08:00:00"),
                "8 AM {tz} should show 08:00:00, got: {}",
                result.result
            );
        }
    }

    #[test]
    fn test_south_american_timezones() {
        let mut calc = Calculator::new();
        for tz in ["ART", "BRT"] {
            let expr = format!("2 PM {tz}");
            let result = calc.calculate_internal(&expr);
            assert!(
                result.success,
                "2 PM {tz} should succeed: {:?}",
                result.error
            );
        }
    }

    #[test]
    fn test_african_timezones() {
        let mut calc = Calculator::new();
        for tz in ["WAT", "CAT", "EAT", "SAST"] {
            let expr = format!("10 AM {tz}");
            let result = calc.calculate_internal(&expr);
            assert!(
                result.success,
                "10 AM {tz} should succeed: {:?}",
                result.error
            );
        }
    }

    #[test]
    fn test_half_hour_offset_ist() {
        let mut calc = Calculator::new();
        // IST is UTC+5:30
        let result = calc.calculate_internal("12 PM IST as UTC");
        assert!(
            result.success,
            "12 PM IST as UTC should succeed: {:?}",
            result.error
        );
        // 12 PM IST (UTC+5:30) = 6:30 AM UTC
        assert!(
            result.result.contains("06:30:00"),
            "12 PM IST as UTC should be 06:30:00, got: {}",
            result.result
        );
    }
}

mod lino_notation {
    use super::*;

    #[test]
    fn test_timezone_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT");
        assert!(result.success);
        assert!(
            result.lino_interpretation.contains("18:00:00"),
            "Lino should contain the time: {}",
            result.lino_interpretation
        );
    }

    #[test]
    fn test_timezone_conversion_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("6 PM GMT as MSK");
        assert!(result.success);
        assert!(
            result.lino_interpretation.contains("as MSK"),
            "Lino should show conversion target: {}",
            result.lino_interpretation
        );
    }
}
