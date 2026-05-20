//! Regression tests for issue #162: adjacent SI suffixes before units.
//!
//! The original failure parsed `19к рублей в долларах` as `19 к` and then
//! rejected the trailing `рублей`. The suffix should scale the number first,
//! so the expression is equivalent to `19000 рублей в долларах`.

use link_calculator::grammar::ExpressionParser;
use link_calculator::types::{Decimal, Expression, Unit};
use link_calculator::Calculator;

fn assert_currency_conversion(input: &str, expected_value: &str, source: &str, target: &str) {
    let parser = ExpressionParser::new();
    let expr = parser
        .parse(input)
        .unwrap_or_else(|err| panic!("{input:?} should parse, got {err}"));

    let Expression::UnitConversion { value, target_unit } = expr else {
        panic!("{input:?} should parse as a unit conversion");
    };
    assert_eq!(target_unit, Unit::currency(target));

    let Expression::Number { value, unit, .. } = *value else {
        panic!("{input:?} source should be a number with a unit");
    };
    assert_eq!(value, expected_value.parse::<Decimal>().unwrap());
    assert_eq!(unit, Unit::currency(source));
}

#[test]
fn issue_162_cyrillic_k_scales_before_russian_currency_unit() {
    assert_currency_conversion("19к рублей в долларах", "19000", "RUB", "USD");
}

#[test]
fn issue_162_exact_input_evaluates_as_19000_rub_to_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("19к рублей в долларах");

    assert!(
        result.success,
        "19к рублей в долларах should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.lino_interpretation, "((19000 RUB) as USD)");
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got {}",
        result.result
    );
}

#[test]
fn issue_162_latin_k_scales_before_currency_units_across_languages() {
    for input in [
        "19k rubles in dollars",
        "19k рублей в долларах",
        "19k roubles en dollars",
        "19k 卢布 换成 美元",
        "19k रूबल में डॉलर",
        "19k روبل إلى دولار",
    ] {
        assert_currency_conversion(input, "19000", "RUB", "USD");
    }
}

#[test]
fn issue_162_decimal_and_larger_si_suffixes_scale_values() {
    for (input, expected_value) in [
        ("1.5k USD in EUR", "1500"),
        ("2M USD in EUR", "2000000"),
        ("3G USD in EUR", "3000000000"),
    ] {
        assert_currency_conversion(input, expected_value, "USD", "EUR");
    }
}

#[test]
fn issue_162_si_suffixes_follow_case_sensitive_meanings() {
    for (input, expected_value) in [
        ("500m USD in EUR", "0.5"),
        ("7u USD in EUR", "0.000007"),
        ("4da USD in EUR", "40"),
        ("5h USD in EUR", "500"),
    ] {
        assert_currency_conversion(input, expected_value, "USD", "EUR");
    }
}

#[test]
fn issue_162_adjacent_duration_abbreviations_still_parse_as_units() {
    let mut calc = Calculator::new();

    let hours = calc.calculate_internal("2h in minutes");
    assert!(
        hours.success,
        "2h in minutes should still parse as hours, got error: {:?}",
        hours.error
    );
    assert_eq!(hours.result, "120 minutes");

    let days = calc.calculate_internal("3d in hours");
    assert!(
        days.success,
        "3d in hours should still parse as days, got error: {:?}",
        days.error
    );
    assert_eq!(days.result, "72 hours");
}
