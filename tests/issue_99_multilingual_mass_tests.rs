//! Tests for issue #99: Multilingual mass unit aliases.
//!
//! The calculator supports 7 UI languages: English (en), Russian (ru),
//! Chinese (zh), Hindi (hi), Arabic (ar), German (de), and French (fr).
//!
//! This test file verifies that all supported languages can express mass
//! unit conversions using their native words for units.
//!
//! The exact failing input from the issue: `19 ton в кг`

use link_calculator::Calculator;

// ═══════════════════════════════════════════════════════════════════════════════
// Issue #99 — exact failing input
// ═══════════════════════════════════════════════════════════════════════════════

/// The exact input from the issue report: `19 ton в кг`
#[test]
fn test_issue_99_exact_input() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("19 ton в кг");
    assert!(
        result.success,
        "19 ton в кг should succeed (19 metric tons to kg), got error: {:?}",
        result.error
    );
    assert_eq!(
        result.result, "19000 kg",
        "19 metric tons = 19000 kg, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// RUSSIAN (ru) — mass unit aliases
// ═══════════════════════════════════════════════════════════════════════════════

/// Russian: "кг" should be recognized as kilogram.
#[test]
fn test_russian_kg_alias() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 г в кг");
    assert!(
        result.success,
        "1000 г в кг should succeed (1000 grams to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// Russian: "г" should be recognized as gram.
#[test]
fn test_russian_gram_alias() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 кг в г");
    assert!(
        result.success,
        "1 кг в г should succeed (1 kg to grams), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// Russian: "мг" should be recognized as milligram.
#[test]
fn test_russian_mg_alias() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 г в мг");
    assert!(
        result.success,
        "1 г в мг should succeed (1 gram to mg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 mg");
}

/// Russian: full word "килограмм" should be recognized.
#[test]
fn test_russian_kilogramm_full_word() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 тонна в килограммах");
    assert!(
        result.success,
        "1 тонна в килограммах should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 kg");
}

/// Russian: "тонна" should be recognized as metric ton.
#[test]
fn test_russian_tonna() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 тонна в кг");
    assert!(
        result.success,
        "1 тонна в кг should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 kg");
}

/// Russian: genitive case "тонн" should be recognized.
#[test]
fn test_russian_tonn_genitive() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("5 тонн в кг");
    assert!(
        result.success,
        "5 тонн в кг should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "5000 kg");
}

/// Russian: "унция" (ounce) should be recognized.
#[test]
fn test_russian_untsia_ounce() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 кг в унциях");
    assert!(
        result.success,
        "1 кг в унциях should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("oz"),
        "Result should be in oz, got: {}",
        result.result
    );
}

/// Russian: "граммов" (genitive plural) should be recognized.
#[test]
fn test_russian_grammov_genitive() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 кг в граммов");
    assert!(
        result.success,
        "1 кг в граммов should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHINESE SIMPLIFIED (zh) — mass unit aliases
// ═══════════════════════════════════════════════════════════════════════════════

/// Chinese: "公斤" (gōng jīn, common for kg) should be recognized.
#[test]
fn test_chinese_gongjin_kg() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 克 换成 公斤");
    assert!(
        result.success,
        "1000 克 换成 公斤 should succeed (1000 g to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// Chinese: "千克" (qiān kè, formal for kg) should be recognized.
#[test]
fn test_chinese_qianke_kg() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 千克 换成 克");
    assert!(
        result.success,
        "1 千克 换成 克 should succeed (1 kg to grams), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// Chinese: "克" (kè, gram) should be recognized.
#[test]
fn test_chinese_ke_gram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 公斤 换成 克");
    assert!(
        result.success,
        "1 公斤 换成 克 should succeed (1 kg to g), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// Chinese: "吨" (dūn, metric ton) should be recognized.
#[test]
fn test_chinese_dun_ton() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 吨 换成 公斤");
    assert!(
        result.success,
        "1 吨 换成 公斤 should succeed (1 ton to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 kg");
}

/// Chinese: "磅" (bàng, pound) should be recognized.
#[test]
fn test_chinese_bang_pound() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 磅 换成 克");
    assert!(
        result.success,
        "1 磅 换成 克 should succeed (1 lb to g), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains('g'),
        "Result should be in grams, got: {}",
        result.result
    );
}

/// Chinese: "盎司" (àng sī, ounce) should be recognized.
#[test]
fn test_chinese_angsi_ounce() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 公斤 换成 盎司");
    assert!(
        result.success,
        "1 公斤 换成 盎司 should succeed (1 kg to oz), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("oz"),
        "Result should be in oz, got: {}",
        result.result
    );
}

/// Chinese: "毫克" (háo kè, milligram) should be recognized.
#[test]
fn test_chinese_haoke_milligram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 克 换成 毫克");
    assert!(
        result.success,
        "1 克 换成 毫克 should succeed (1 g to mg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 mg");
}

// ═══════════════════════════════════════════════════════════════════════════════
// HINDI (hi) — mass unit aliases
// ═══════════════════════════════════════════════════════════════════════════════

/// Hindi: "किलोग्राम" (kilōgrām) should be recognized as kilogram.
#[test]
fn test_hindi_kilogram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 ग्राम में किलोग्राम");
    assert!(
        result.success,
        "1000 ग्राम में किलोग्राम should succeed (1000 g to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// Hindi: "ग्राम" (grām) should be recognized as gram.
#[test]
fn test_hindi_gram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 किलोग्राम में ग्राम");
    assert!(
        result.success,
        "1 किलोग्राम में ग्राम should succeed (1 kg to g), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// Hindi: "टन" (ṭan) should be recognized as metric ton.
#[test]
fn test_hindi_ton() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 टन में किलोग्राम");
    assert!(
        result.success,
        "1 टन में किलोग्राम should succeed (1 ton to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 kg");
}

/// Hindi: "किलो" (kilo, short form) should be recognized as kilogram.
#[test]
fn test_hindi_kilo_short() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("2 किलो में ग्राम");
    assert!(
        result.success,
        "2 किलो में ग्राम should succeed (2 kg to g), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "2000 g");
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARABIC (ar) — mass unit aliases
// ═══════════════════════════════════════════════════════════════════════════════

/// Arabic: "كيلوغرام" (kīlūġrām) should be recognized as kilogram.
#[test]
fn test_arabic_kilogram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 غرام إلى كيلوغرام");
    assert!(
        result.success,
        "1000 غرام إلى كيلوغرام should succeed (1000 g to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// Arabic: "غرام" (ġrām) should be recognized as gram.
#[test]
fn test_arabic_gram() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 كيلوغرام إلى غرام");
    assert!(
        result.success,
        "1 كيلوغرام إلى غرام should succeed (1 kg to g), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// Arabic: "طن" (ṭun) should be recognized as metric ton.
#[test]
fn test_arabic_ton() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 طن إلى كيلوغرام");
    assert!(
        result.success,
        "1 طن إلى كيلوغرام should succeed (1 ton to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 kg");
}

/// Arabic: "رطل" (raṭl, pound) should be recognized — distinct from "جنيه" (GBP).
#[test]
fn test_arabic_ratl_pound() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 رطل إلى غرام");
    assert!(
        result.success,
        "1 رطل إلى غرام should succeed (1 lb to g), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains('g'),
        "Result should be in grams, got: {}",
        result.result
    );
}

/// Arabic: "كغ" (abbreviation for kg) should be recognized.
#[test]
fn test_arabic_kg_abbreviation() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 غرام إلى كغ");
    assert!(
        result.success,
        "1000 غرام إلى كغ should succeed (1000 g to kg), got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

// ═══════════════════════════════════════════════════════════════════════════════
// GERMAN (de) — mass unit aliases (Latin-script full words)
// ═══════════════════════════════════════════════════════════════════════════════

/// German: "Kilogramm" should be recognized.
#[test]
fn test_german_kilogramm() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 Gramm in Kilogramm");
    assert!(
        result.success,
        "1000 Gramm in Kilogramm should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// German: "Gramm" should be recognized.
#[test]
fn test_german_gramm() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 Kilogramm in Gramm");
    assert!(
        result.success,
        "1 Kilogramm in Gramm should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// German: "Unze" should be recognized as ounce.
#[test]
fn test_german_unze() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 kg in Unzen");
    assert!(
        result.success,
        "1 kg in Unzen should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("oz"),
        "Result should be in oz, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// FRENCH (fr) — mass unit aliases (Latin-script full words)
// ═══════════════════════════════════════════════════════════════════════════════

/// French: "kilogramme" should be recognized.
#[test]
fn test_french_kilogramme() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 grammes en kilogrammes");
    assert!(
        result.success,
        "1000 grammes en kilogrammes should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1 kg");
}

/// French: "gramme" should be recognized.
#[test]
fn test_french_gramme() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 kilogramme en grammes");
    assert!(
        result.success,
        "1 kilogramme en grammes should succeed, got error: {:?}",
        result.error
    );
    assert_eq!(result.result, "1000 g");
}

/// French: "once" (ounce) should be recognized.
#[test]
fn test_french_once_ounce() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1 kg en onces");
    assert!(
        result.success,
        "1 kg en onces should succeed, got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("oz"),
        "Result should be in oz, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Parse-level unit tests for MassUnit::parse()
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod parse_mass_unit_tests {
    use link_calculator::types::MassUnit;

    fn parse(s: &str) -> Option<MassUnit> {
        MassUnit::parse(s)
    }

    // Russian
    #[test]
    fn test_parse_russian_kg() {
        assert_eq!(parse("кг"), Some(MassUnit::Kilogram));
    }

    #[test]
    fn test_parse_russian_gram() {
        assert_eq!(parse("г"), Some(MassUnit::Gram));
        assert_eq!(parse("гр"), Some(MassUnit::Gram));
        assert_eq!(parse("грамм"), Some(MassUnit::Gram));
        assert_eq!(parse("граммов"), Some(MassUnit::Gram));
    }

    #[test]
    fn test_parse_russian_kilogramm() {
        assert_eq!(parse("килограмм"), Some(MassUnit::Kilogram));
        assert_eq!(parse("килограммов"), Some(MassUnit::Kilogram));
        assert_eq!(parse("килограммах"), Some(MassUnit::Kilogram));
    }

    #[test]
    fn test_parse_russian_mg() {
        assert_eq!(parse("мг"), Some(MassUnit::Milligram));
        assert_eq!(parse("миллиграмм"), Some(MassUnit::Milligram));
    }

    #[test]
    fn test_parse_russian_tonna() {
        assert_eq!(parse("тонна"), Some(MassUnit::MetricTon));
        assert_eq!(parse("тонны"), Some(MassUnit::MetricTon));
        assert_eq!(parse("тонн"), Some(MassUnit::MetricTon));
    }

    #[test]
    fn test_parse_russian_untsia() {
        assert_eq!(parse("унция"), Some(MassUnit::Ounce));
        assert_eq!(parse("унций"), Some(MassUnit::Ounce));
        assert_eq!(parse("унциях"), Some(MassUnit::Ounce));
    }

    // Chinese
    #[test]
    fn test_parse_chinese_kg() {
        assert_eq!(parse("千克"), Some(MassUnit::Kilogram));
        assert_eq!(parse("公斤"), Some(MassUnit::Kilogram));
    }

    #[test]
    fn test_parse_chinese_gram() {
        assert_eq!(parse("克"), Some(MassUnit::Gram));
    }

    #[test]
    fn test_parse_chinese_mg() {
        assert_eq!(parse("毫克"), Some(MassUnit::Milligram));
    }

    #[test]
    fn test_parse_chinese_ton() {
        assert_eq!(parse("吨"), Some(MassUnit::MetricTon));
    }

    #[test]
    fn test_parse_chinese_pound() {
        assert_eq!(parse("磅"), Some(MassUnit::Pound));
    }

    #[test]
    fn test_parse_chinese_ounce() {
        assert_eq!(parse("盎司"), Some(MassUnit::Ounce));
    }

    // Hindi
    #[test]
    fn test_parse_hindi_kilogram() {
        assert_eq!(parse("किलोग्राम"), Some(MassUnit::Kilogram));
        assert_eq!(parse("किलो"), Some(MassUnit::Kilogram));
    }

    #[test]
    fn test_parse_hindi_gram() {
        assert_eq!(parse("ग्राम"), Some(MassUnit::Gram));
    }

    #[test]
    fn test_parse_hindi_ton() {
        assert_eq!(parse("टन"), Some(MassUnit::MetricTon));
    }

    // Arabic
    #[test]
    fn test_parse_arabic_kilogram() {
        assert_eq!(parse("كيلوغرام"), Some(MassUnit::Kilogram));
        assert_eq!(parse("كيلوجرام"), Some(MassUnit::Kilogram));
        assert_eq!(parse("كغ"), Some(MassUnit::Kilogram));
        assert_eq!(parse("كيلو"), Some(MassUnit::Kilogram));
    }

    #[test]
    fn test_parse_arabic_gram() {
        assert_eq!(parse("غرام"), Some(MassUnit::Gram));
        assert_eq!(parse("جرام"), Some(MassUnit::Gram));
    }

    #[test]
    fn test_parse_arabic_ton() {
        assert_eq!(parse("طن"), Some(MassUnit::MetricTon));
        assert_eq!(parse("أطنان"), Some(MassUnit::MetricTon));
    }

    #[test]
    fn test_parse_arabic_pound() {
        assert_eq!(parse("رطل"), Some(MassUnit::Pound));
        assert_eq!(parse("أرطال"), Some(MassUnit::Pound));
    }

    #[test]
    fn test_parse_arabic_ounce() {
        assert_eq!(parse("أونصة"), Some(MassUnit::Ounce));
        assert_eq!(parse("أوقية"), Some(MassUnit::Ounce));
    }

    // German
    #[test]
    fn test_parse_german_units() {
        assert_eq!(parse("kilogramm"), Some(MassUnit::Kilogram));
        assert_eq!(parse("gramm"), Some(MassUnit::Gram));
        assert_eq!(parse("milligramm"), Some(MassUnit::Milligram));
        assert_eq!(parse("kilo"), Some(MassUnit::Kilogram));
        assert_eq!(parse("tonnen"), Some(MassUnit::MetricTon));
        assert_eq!(parse("unze"), Some(MassUnit::Ounce));
        assert_eq!(parse("unzen"), Some(MassUnit::Ounce));
    }

    // French
    #[test]
    fn test_parse_french_units() {
        assert_eq!(parse("kilogramme"), Some(MassUnit::Kilogram));
        assert_eq!(parse("kilogrammes"), Some(MassUnit::Kilogram));
        assert_eq!(parse("gramme"), Some(MassUnit::Gram));
        assert_eq!(parse("grammes"), Some(MassUnit::Gram));
        assert_eq!(parse("milligramme"), Some(MassUnit::Milligram));
        assert_eq!(parse("milligrammes"), Some(MassUnit::Milligram));
        assert_eq!(parse("once"), Some(MassUnit::Ounce));
        assert_eq!(parse("onces"), Some(MassUnit::Ounce));
    }
}
