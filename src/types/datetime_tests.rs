//! Tests for the `DateTime` type.

use super::*;
use chrono::Timelike;

#[test]
fn test_parse_iso_date() {
    let dt = DateTime::parse("2026-01-22").unwrap();
    assert!(dt.has_date);
    assert!(!dt.has_time);
}

#[test]
fn test_parse_us_date() {
    let dt = DateTime::parse("01/22/2026").unwrap();
    assert!(dt.has_date);
}

#[test]
fn test_parse_dot_date_european() {
    // DD.MM.YYYY (German/Russian convention) — issue #166.
    let dt = DateTime::parse("15.10.2025").unwrap();
    assert!(dt.has_date);
    assert_eq!(dt.year(), 2025);
    assert_eq!(dt.to_string(), "2025-10-15");
}

#[test]
fn test_parse_dot_date_iso() {
    // YYYY.MM.DD
    let dt = DateTime::parse("2025.10.15").unwrap();
    assert_eq!(dt.to_string(), "2025-10-15");
}

#[test]
fn test_parse_month_name_date() {
    let dt = DateTime::parse("Jan 22, 2026").unwrap();
    assert!(dt.has_date);
    assert_eq!(dt.year(), 2026);
}

#[test]
fn test_parse_time_12h() {
    let dt = DateTime::parse("8:59am").unwrap();
    assert!(dt.has_time);
}

#[test]
fn test_parse_time_with_utc() {
    let dt = DateTime::parse("8:59am UTC").unwrap();
    assert!(dt.has_time);
    assert!(dt.offset_seconds.is_some());
}

#[test]
fn test_parse_datetime_with_partial_date() {
    let dt = DateTime::parse("Jan 27, 8:59am UTC").unwrap();
    assert!(dt.has_date);
    assert!(dt.has_time);
}

#[test]
fn test_datetime_subtraction() {
    let dt1 = DateTime::parse("Jan 27, 8:59am UTC").unwrap();
    let dt2 = DateTime::parse("Jan 25, 12:51pm UTC").unwrap();
    let diff = dt1.subtract(&dt2);
    // Should be approximately 44 hours and 8 minutes
    let hours = diff.as_secs() / 3600;
    assert!(hours > 40 && hours < 50);
}

#[test]
fn test_today_uses_requested_timezone_date() {
    let now = Utc::now();
    let (offset_seconds, expected_date) = if now.time().hour() < 12 {
        (-12 * 60 * 60, (now - Duration::hours(12)).date_naive())
    } else {
        (14 * 60 * 60, (now + Duration::hours(14)).date_naive())
    };

    let today = DateTime::today(offset_seconds);
    assert_eq!(today.inner.date_naive(), expected_date);
    assert!(today.has_date());
    assert!(!today.has_time());
}

#[test]
fn test_22_jan_2026_format() {
    let dt = DateTime::parse("22 Jan 2026").unwrap();
    assert!(dt.has_date);
    assert_eq!(dt.year(), 2026);
}

/// Issue #212: "08 Aug 2026 22:35" (date followed by a 24-hour time,
/// without a comma) failed to parse.
#[test]
fn test_date_then_time_without_comma() {
    let dt = DateTime::parse("08 Aug 2026 22:35").unwrap();
    assert!(dt.has_date);
    assert!(dt.has_time);
    assert_eq!(
        dt.inner.format("%Y-%m-%d %H:%M").to_string(),
        "2026-08-08 22:35"
    );
}

#[test]
fn test_date_then_time_variants() {
    for input in [
        "2026-08-08 22:35",
        "Aug 08, 2026 22:35",
        "August 8 2026 10:35pm",
        "08.08.2026 22:35",
    ] {
        let dt = DateTime::parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        assert_eq!(
            dt.inner.format("%Y-%m-%d %H:%M").to_string(),
            "2026-08-08 22:35",
            "input: {input}"
        );
    }
}

#[test]
fn test_date_then_time_with_timezone() {
    let dt = DateTime::parse("08 Aug 2026 22:35 UTC").unwrap();
    assert_eq!(dt.offset_seconds, Some(0));
    assert_eq!(
        dt.inner.format("%Y-%m-%d %H:%M").to_string(),
        "2026-08-08 22:35"
    );

    // +3 local time is 3 hours ahead of UTC.
    let dt = DateTime::parse("08 Aug 2026 22:35 MSK").unwrap();
    assert_eq!(dt.offset_seconds, Some(3 * 3600));
    assert_eq!(
        dt.inner.format("%Y-%m-%d %H:%M").to_string(),
        "2026-08-08 19:35"
    );
}
