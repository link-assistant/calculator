//! Tests for multilingual date arithmetic expressions (extension of issue #125).
//!
//! Issue #125 fixed Russian date arithmetic: `17 февраля 2027 - 6 месяцев`.
//! This test file verifies that the same capability works for all 7 supported
//! UI languages: English (en), Russian (ru), German (de), French (fr),
//! Chinese Simplified (zh), Hindi (hi), and Arabic (ar).
//!
//! Pattern tested: `<DATE_IN_LANGUAGE> +/- <DURATION_IN_LANGUAGE>`
//!
//! Key mechanisms:
//! 1. `translate_month_names()` in `datetime_parse.rs` converts native month
//!    names to English before parsing.
//! 2. `DurationUnit::parse()` in `unit.rs` recognizes duration unit names in
//!    all supported languages.
//! 3. `looks_like_datetime()` in `datetime_grammar.rs` detects native month
//!    names so the parser tries the datetime path.

use link_calculator::Calculator;

// ═══════════════════════════════════════════════════════════════════════════════
// GERMAN (de) — date arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// German: "17. Februar 2027 - 6 Monate" → 2026-08-17
#[test]
fn test_german_date_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 Februar 2027 - 6 Monate");
    assert!(
        result.success,
        "17 Februar 2027 - 6 Monate should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 Februar 2027 - 6 Monate should be around August 2026, got: {}",
        result.result
    );
}

/// German: "17 Februar 2027 + 3 Monate" → 2027-05-17
#[test]
fn test_german_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 Februar 2027 + 3 Monate");
    assert!(
        result.success,
        "17 Februar 2027 + 3 Monate should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-05"),
        "17 Februar 2027 + 3 Monate should be 2027-05-17, got: {}",
        result.result
    );
}

/// German: "März" is recognized as March
#[test]
fn test_german_maerz_month() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("15 März 2027 - 1 Jahr");
    assert!(
        result.success,
        "15 März 2027 - 1 Jahr should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-03"),
        "15 März 2027 - 1 Jahr should be 2026-03-15, got: {}",
        result.result
    );
}

/// German: duration units — Wochen (weeks), Tage (days)
#[test]
fn test_german_duration_units() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("1 Januar 2027 + 2 Wochen");
    assert!(
        result.success,
        "1 Januar 2027 + 2 Wochen should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-15"),
        "1 Januar 2027 + 2 Wochen should be 2027-01-15, got: {}",
        result.result
    );

    let result2 = calc.calculate_internal("1 Januar 2027 + 7 Tage");
    assert!(
        result2.success,
        "1 Januar 2027 + 7 Tage should succeed, got error: {:?}",
        result2.error
    );
    assert!(
        result2.result.starts_with("2027-01-08"),
        "1 Januar 2027 + 7 Tage should be 2027-01-08, got: {}",
        result2.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// FRENCH (fr) — date arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// French: "17 février 2027 - 6 mois" → 2026-08-17
#[test]
fn test_french_date_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 février 2027 - 6 mois");
    assert!(
        result.success,
        "17 février 2027 - 6 mois should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 février 2027 - 6 mois should be around August 2026, got: {}",
        result.result
    );
}

/// French: accent-free variant "fevrier" also works
#[test]
fn test_french_fevrier_no_accent() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 fevrier 2027 - 6 mois");
    assert!(
        result.success,
        "17 fevrier 2027 - 6 mois should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 fevrier 2027 - 6 mois should be around August 2026, got: {}",
        result.result
    );
}

/// French: "17 février 2027 + 3 mois" → 2027-05-17
#[test]
fn test_french_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 février 2027 + 3 mois");
    assert!(
        result.success,
        "17 février 2027 + 3 mois should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-05"),
        "17 février 2027 + 3 mois should be 2027-05-17, got: {}",
        result.result
    );
}

/// French: duration units — semaines (weeks), jours (days), ans (years)
#[test]
fn test_french_duration_units() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("1 janvier 2027 + 2 semaines");
    assert!(
        result.success,
        "1 janvier 2027 + 2 semaines should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-15"),
        "1 janvier 2027 + 2 semaines should be 2027-01-15, got: {}",
        result.result
    );

    let result2 = calc.calculate_internal("1 janvier 2027 - 1 an");
    assert!(
        result2.success,
        "1 janvier 2027 - 1 an should succeed, got error: {:?}",
        result2.error
    );
    assert!(
        result2.result.starts_with("2026-01"),
        "1 janvier 2027 - 1 an should be 2026-01-01, got: {}",
        result2.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHINESE SIMPLIFIED (zh) — date arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// Chinese: "2027年二月17日 - 6个月" — Chinese dates typically use ISO or numeric
/// but month words like 二月 (February) should be recognized.
#[test]
fn test_chinese_month_name_recognized() {
    let mut calc = Calculator::new();
    // Test that 二月 (February) is recognized as a datetime component
    let result = calc.calculate_internal("17 二月 2027 - 6 个月");
    assert!(
        result.success,
        "17 二月 2027 - 6 个月 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 二月 2027 - 6 个月 should be around August 2026, got: {}",
        result.result
    );
}

/// Chinese: "17 一月 2027 + 3 月" → 2027-04-17
#[test]
fn test_chinese_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 一月 2027 + 3 月");
    assert!(
        result.success,
        "17 一月 2027 + 3 月 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-04"),
        "17 一月 2027 + 3 月 should be around April 2027, got: {}",
        result.result
    );
}

/// Chinese: duration unit 天 (days)
#[test]
fn test_chinese_duration_days() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 一月 2027 + 7 天");
    assert!(
        result.success,
        "1 一月 2027 + 7 天 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-08"),
        "1 一月 2027 + 7 天 should be 2027-01-08, got: {}",
        result.result
    );
}

/// Chinese: duration unit 周 (weeks)
#[test]
fn test_chinese_duration_weeks() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 一月 2027 + 2 周");
    assert!(
        result.success,
        "1 一月 2027 + 2 周 should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-15"),
        "1 一月 2027 + 2 周 should be 2027-01-15, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// HINDI (hi) — date arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// Hindi: "17 फरवरी 2027 - 6 महीने" → 2026-08-17
#[test]
fn test_hindi_date_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 फरवरी 2027 - 6 महीने");
    assert!(
        result.success,
        "17 फरवरी 2027 - 6 महीने should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 फरवरी 2027 - 6 महीने should be around August 2026, got: {}",
        result.result
    );
}

/// Hindi: "17 फरवरी 2027 + 3 महीने" → 2027-05-17
#[test]
fn test_hindi_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 फरवरी 2027 + 3 महीने");
    assert!(
        result.success,
        "17 फरवरी 2027 + 3 महीने should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-05"),
        "17 फरवरी 2027 + 3 महीने should be 2027-05-17, got: {}",
        result.result
    );
}

/// Hindi: duration units — दिन (days), सप्ताह (weeks)
#[test]
fn test_hindi_duration_units() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("1 जनवरी 2027 + 7 दिन");
    assert!(
        result.success,
        "1 जनवरी 2027 + 7 दिन should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-08"),
        "1 जनवरी 2027 + 7 दिन should be 2027-01-08, got: {}",
        result.result
    );

    let result2 = calc.calculate_internal("1 जनवरी 2027 + 2 सप्ताह");
    assert!(
        result2.success,
        "1 जनवरी 2027 + 2 सप्ताह should succeed, got error: {:?}",
        result2.error
    );
    assert!(
        result2.result.starts_with("2027-01-15"),
        "1 जनवरी 2027 + 2 सप्ताह should be 2027-01-15, got: {}",
        result2.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARABIC (ar) — date arithmetic
// ═══════════════════════════════════════════════════════════════════════════════

/// Arabic: "17 فبراير 2027 - 6 أشهر" → 2026-08-17
#[test]
fn test_arabic_date_minus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 فبراير 2027 - 6 أشهر");
    assert!(
        result.success,
        "17 فبراير 2027 - 6 أشهر should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2026-08"),
        "17 فبراير 2027 - 6 أشهر should be around August 2026, got: {}",
        result.result
    );
}

/// Arabic: "17 فبراير 2027 + 3 أشهر" → 2027-05-17
#[test]
fn test_arabic_date_plus_months() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("17 فبراير 2027 + 3 أشهر");
    assert!(
        result.success,
        "17 فبراير 2027 + 3 أشهر should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-05"),
        "17 فبراير 2027 + 3 أشهر should be 2027-05-17, got: {}",
        result.result
    );
}

/// Arabic: duration units — أيام (days), أسابيع (weeks)
#[test]
fn test_arabic_duration_units() {
    let mut calc = Calculator::new();

    let result = calc.calculate_internal("1 يناير 2027 + 7 أيام");
    assert!(
        result.success,
        "1 يناير 2027 + 7 أيام should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.starts_with("2027-01-08"),
        "1 يناير 2027 + 7 أيام should be 2027-01-08, got: {}",
        result.result
    );

    let result2 = calc.calculate_internal("1 يناير 2027 + 2 أسابيع");
    assert!(
        result2.success,
        "1 يناير 2027 + 2 أسابيع should succeed, got error: {:?}",
        result2.error
    );
    assert!(
        result2.result.starts_with("2027-01-15"),
        "1 يناير 2027 + 2 أسابيع should be 2027-01-15, got: {}",
        result2.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit-level: DurationUnit::parse() for all new languages
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod duration_unit_parse_tests {
    use link_calculator::types::DurationUnit;

    fn parse(s: &str) -> Option<DurationUnit> {
        DurationUnit::parse(s)
    }

    // German
    #[test]
    fn test_german_duration_units() {
        assert_eq!(parse("Millisekunde"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("millisekunden"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("Sekunde"), Some(DurationUnit::Seconds));
        assert_eq!(parse("sekunden"), Some(DurationUnit::Seconds));
        assert_eq!(parse("Minuten"), Some(DurationUnit::Minutes));
        assert_eq!(parse("Stunde"), Some(DurationUnit::Hours));
        assert_eq!(parse("stunden"), Some(DurationUnit::Hours));
        assert_eq!(parse("Tag"), Some(DurationUnit::Days));
        assert_eq!(parse("Tage"), Some(DurationUnit::Days));
        assert_eq!(parse("Woche"), Some(DurationUnit::Weeks));
        assert_eq!(parse("wochen"), Some(DurationUnit::Weeks));
        assert_eq!(parse("Monat"), Some(DurationUnit::Months));
        assert_eq!(parse("Monate"), Some(DurationUnit::Months));
        assert_eq!(parse("Jahr"), Some(DurationUnit::Years));
        assert_eq!(parse("jahre"), Some(DurationUnit::Years));
    }

    // French
    #[test]
    fn test_french_duration_units() {
        assert_eq!(parse("milliseconde"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("millisecondes"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("seconde"), Some(DurationUnit::Seconds));
        assert_eq!(parse("secondes"), Some(DurationUnit::Seconds));
        assert_eq!(parse("heure"), Some(DurationUnit::Hours));
        assert_eq!(parse("heures"), Some(DurationUnit::Hours));
        assert_eq!(parse("jour"), Some(DurationUnit::Days));
        assert_eq!(parse("jours"), Some(DurationUnit::Days));
        assert_eq!(parse("semaine"), Some(DurationUnit::Weeks));
        assert_eq!(parse("semaines"), Some(DurationUnit::Weeks));
        assert_eq!(parse("mois"), Some(DurationUnit::Months));
        assert_eq!(parse("an"), Some(DurationUnit::Years));
        assert_eq!(parse("ans"), Some(DurationUnit::Years));
        assert_eq!(parse("année"), Some(DurationUnit::Years));
        assert_eq!(parse("années"), Some(DurationUnit::Years));
        assert_eq!(parse("annee"), Some(DurationUnit::Years));
    }

    // Chinese
    #[test]
    fn test_chinese_duration_units() {
        assert_eq!(parse("毫秒"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("秒"), Some(DurationUnit::Seconds));
        assert_eq!(parse("分钟"), Some(DurationUnit::Minutes));
        assert_eq!(parse("分"), Some(DurationUnit::Minutes));
        assert_eq!(parse("小时"), Some(DurationUnit::Hours));
        assert_eq!(parse("天"), Some(DurationUnit::Days));
        assert_eq!(parse("日"), Some(DurationUnit::Days));
        assert_eq!(parse("周"), Some(DurationUnit::Weeks));
        assert_eq!(parse("星期"), Some(DurationUnit::Weeks));
        assert_eq!(parse("月"), Some(DurationUnit::Months));
        assert_eq!(parse("个月"), Some(DurationUnit::Months));
        assert_eq!(parse("年"), Some(DurationUnit::Years));
    }

    // Hindi
    #[test]
    fn test_hindi_duration_units() {
        assert_eq!(parse("मिलीसेकंड"), Some(DurationUnit::Milliseconds));
        assert_eq!(parse("सेकंड"), Some(DurationUnit::Seconds));
        assert_eq!(parse("मिनट"), Some(DurationUnit::Minutes));
        assert_eq!(parse("घंटा"), Some(DurationUnit::Hours));
        assert_eq!(parse("घंटे"), Some(DurationUnit::Hours));
        assert_eq!(parse("दिन"), Some(DurationUnit::Days));
        assert_eq!(parse("सप्ताह"), Some(DurationUnit::Weeks));
        assert_eq!(parse("हफ्ता"), Some(DurationUnit::Weeks));
        assert_eq!(parse("महीना"), Some(DurationUnit::Months));
        assert_eq!(parse("महीने"), Some(DurationUnit::Months));
        assert_eq!(parse("माह"), Some(DurationUnit::Months));
        assert_eq!(parse("साल"), Some(DurationUnit::Years));
        assert_eq!(parse("वर्ष"), Some(DurationUnit::Years));
    }

    // Arabic
    #[test]
    fn test_arabic_duration_units() {
        assert_eq!(parse("ثانية"), Some(DurationUnit::Seconds));
        assert_eq!(parse("ثواني"), Some(DurationUnit::Seconds));
        assert_eq!(parse("دقيقة"), Some(DurationUnit::Minutes));
        assert_eq!(parse("دقائق"), Some(DurationUnit::Minutes));
        assert_eq!(parse("ساعة"), Some(DurationUnit::Hours));
        assert_eq!(parse("ساعات"), Some(DurationUnit::Hours));
        assert_eq!(parse("يوم"), Some(DurationUnit::Days));
        assert_eq!(parse("أيام"), Some(DurationUnit::Days));
        assert_eq!(parse("ايام"), Some(DurationUnit::Days));
        assert_eq!(parse("أسبوع"), Some(DurationUnit::Weeks));
        assert_eq!(parse("أسابيع"), Some(DurationUnit::Weeks));
        assert_eq!(parse("شهر"), Some(DurationUnit::Months));
        assert_eq!(parse("أشهر"), Some(DurationUnit::Months));
        assert_eq!(parse("شهور"), Some(DurationUnit::Months));
        assert_eq!(parse("سنة"), Some(DurationUnit::Years));
        assert_eq!(parse("سنوات"), Some(DurationUnit::Years));
        assert_eq!(parse("عام"), Some(DurationUnit::Years));
        assert_eq!(parse("أعوام"), Some(DurationUnit::Years));
    }
}
