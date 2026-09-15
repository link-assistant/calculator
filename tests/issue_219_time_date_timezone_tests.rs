//! Regression tests for issue #219: a timezone at the end of a
//! `time date timezone` expression was left as trailing input.

use link_calculator::Calculator;

#[test]
fn reported_time_date_timezone_expression_calculates() {
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("00:35 16 september 2026 IST");

    assert!(result.success, "calculation failed: {:?}", result.error);
    assert_eq!(result.result, "2026-09-16 00:35:00 IST");
    assert_eq!(
        result.lino_interpretation,
        "(2026-09-16 00:35:00 IST)"
    );

    let datetime = result.datetime_result.expect("missing datetime metadata");
    assert_eq!(datetime.utc, "2026-09-15 19:05:00 UTC");
    assert_eq!(datetime.timezone.as_deref(), Some("IST"));
    assert_eq!(datetime.offset_seconds, Some(19_800));
}

#[test]
fn timezone_is_correct_for_every_supported_word_order() {
    let mut calculator = Calculator::new();

    for expression in [
        "00:35 16 september 2026 IST",
        "00:35 IST 16 september 2026",
        "16 september 2026 00:35 IST",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
        assert_eq!(result.result, "2026-09-16 00:35:00 IST", "{expression}");
        assert_eq!(
            result.datetime_result.expect("missing metadata").utc,
            "2026-09-15 19:05:00 UTC",
            "{expression}"
        );
    }
}

#[test]
fn trailing_timezone_supports_12_hour_and_numeric_dates() {
    let mut calculator = Calculator::new();

    for expression in [
        "12:35am 16 September 2026 IST",
        "00:35 2026-09-16 IST",
        "00:35 16/09/2026 IST",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(
            result.success,
            "{expression:?} should succeed: {:?}",
            result.error
        );
        assert_eq!(result.result, "2026-09-16 00:35:00 IST", "{expression}");
    }
}
