//! Tests for the Value type.

use super::*;

#[test]
fn test_number_value() {
    let v = Value::number(Decimal::new(42));
    assert_eq!(v.to_display_string(), "42");
}

#[test]
fn test_currency_value() {
    let v = Value::currency(Decimal::new(100), "USD");
    assert_eq!(v.to_display_string(), "100 USD");
}

#[test]
fn test_number_addition() {
    let a = Value::number(Decimal::new(2));
    let b = Value::number(Decimal::new(3));
    let mut db = CurrencyDatabase::new();
    let result = a.add(&b, &mut db).unwrap();
    assert_eq!(result.to_display_string(), "5");
}

#[test]
fn test_rational_addition() {
    let a = Value::rational(Rational::new(1, 3));
    let b = Value::rational(Rational::new(1, 3));
    let mut db = CurrencyDatabase::new();
    let result = a.add(&b, &mut db).unwrap();
    // 1/3 + 1/3 = 2/3
    assert_eq!(result.to_fraction_string(), Some("2/3".to_string()));
}

#[test]
fn test_currency_addition_same() {
    let a = Value::currency(Decimal::new(100), "USD");
    let b = Value::currency(Decimal::new(50), "USD");
    let mut db = CurrencyDatabase::new();
    let result = a.add(&b, &mut db).unwrap();
    assert_eq!(result.to_display_string(), "150 USD");
}

#[test]
fn test_division_by_zero() {
    let a = Value::number(Decimal::new(10));
    let b = Value::number(Decimal::zero());
    let result = a.divide(&b);
    assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
}

#[test]
fn test_rational_division_by_zero() {
    let a = Value::rational(Rational::from_integer(10));
    let b = Value::rational(Rational::zero());
    let result = a.divide(&b);
    assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
}

#[test]
fn test_one_third_times_three() {
    // This is the key test for issue #21
    let one = Value::rational(Rational::from_integer(1));
    let three = Value::rational(Rational::from_integer(3));

    // 1 / 3
    let one_third = one.divide(&three).unwrap();

    // (1/3) * 3 = 1 (exact!)
    let result = one_third.multiply(&three).unwrap();
    assert_eq!(result.to_display_string(), "1");
}

#[test]
fn test_two_thirds_times_three() {
    let two = Value::rational(Rational::from_integer(2));
    let three = Value::rational(Rational::from_integer(3));

    // 2 / 3
    let two_thirds = two.divide(&three).unwrap();

    // (2/3) * 3 = 2 (exact!)
    let result = two_thirds.multiply(&three).unwrap();
    assert_eq!(result.to_display_string(), "2");
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0), "0 seconds");
    assert_eq!(format_duration(1), "1 second");
    assert_eq!(format_duration(60), "1 minute");
    assert_eq!(format_duration(3661), "1 hour, 1 minute, 1 second");
    assert_eq!(format_duration(86400), "1 day");
}

#[test]
fn test_datetime_subtraction() {
    let dt1 = Value::datetime(DateTime::parse("2026-01-27").unwrap());
    let dt2 = Value::datetime(DateTime::parse("2026-01-25").unwrap());
    let mut db = CurrencyDatabase::new();
    let result = dt1.subtract(&dt2, &mut db).unwrap();
    assert!(matches!(result.kind, ValueKind::Duration { .. }));
}

#[test]
fn test_datetime_plus_duration() {
    let dt = Value::datetime(DateTime::parse("2026-01-25").unwrap());
    let dur = Value::duration(86400); // 1 day in seconds
    let mut db = CurrencyDatabase::new();
    let result = dt.add(&dur, &mut db).unwrap();
    assert!(matches!(result.kind, ValueKind::DateTime(_)));
}

#[test]
fn test_duration_plus_datetime() {
    let dur = Value::duration(86400); // 1 day in seconds (issue #8: duration + datetime)
    let dt = Value::datetime(DateTime::parse("2026-01-25").unwrap());
    let mut db = CurrencyDatabase::new();
    let result = dur.add(&dt, &mut db).unwrap();
    assert!(matches!(result.kind, ValueKind::DateTime(_)));
}

#[test]
fn test_issue_8_expression() {
    // Test the exact expression from issue #8:
    // (Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC) + (Jan 25, 12:51pm UTC)
    let dt1 = Value::datetime(DateTime::parse("Jan 27, 8:59am UTC").unwrap());
    let dt2 = Value::datetime(DateTime::parse("Jan 25, 12:51pm UTC").unwrap());
    let dt3 = Value::datetime(DateTime::parse("Jan 25, 12:51pm UTC").unwrap());
    let mut db = CurrencyDatabase::new();

    // First: dt1 - dt2 = duration
    let duration = dt1.subtract(&dt2, &mut db).unwrap();
    assert!(matches!(duration.kind, ValueKind::Duration { .. }));
    // Second: duration + dt3 = datetime (this was failing before the fix)
    let result = duration.add(&dt3, &mut db).unwrap();
    assert!(matches!(result.kind, ValueKind::DateTime(_)));
}
