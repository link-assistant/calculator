//! Regression tests for issue #154: timezone datetimes show conversion context.

use link_calculator::Calculator;

#[test]
fn msk_time_shows_utc_equivalent_in_steps() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("12:30 по МСК");

    assert!(
        result.success,
        "12:30 по МСК should succeed: {:?}",
        result.error
    );
    assert!(
        result.result.contains("12:30:00 MSK"),
        "Result should preserve the requested Moscow time, got: {}",
        result.result
    );
    assert!(
        result
            .steps
            .iter()
            .any(|step| step == "UTC equivalent: 09:30:00 UTC"),
        "Steps should include the UTC equivalent, got: {:?}",
        result.steps
    );

    let datetime_result = result
        .datetime_result
        .as_ref()
        .expect("timezone datetime should include structured conversion metadata");
    assert_eq!(datetime_result.source, "12:30:00 MSK");
    assert_eq!(datetime_result.utc, "09:30:00 UTC");
    assert_eq!(datetime_result.timezone.as_deref(), Some("MSK"));
    assert_eq!(datetime_result.offset_seconds, Some(10_800));
    assert!(!datetime_result.has_date);
    assert!(datetime_result.has_time);
}
