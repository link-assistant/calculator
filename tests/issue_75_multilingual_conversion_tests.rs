//! Tests for issue #75: Multilingual currency conversion support.
//!
//! The calculator supports 7 UI languages: English (en), Russian (ru),
//! Chinese (zh), Hindi (hi), Arabic (ar), German (de), and French (fr).
//!
//! This test file verifies that all supported languages can express currency
//! conversion using their native words for currencies and conversion keywords.
//!
//! Pattern tested: `NUMBER CURRENCY_NAME KEYWORD TARGET_CURRENCY`
//!
//! Language-specific conversion keywords added to the lexer:
//! - Russian: "в" → TokenKind::In  (already implemented)
//! - French:  "en" → TokenKind::In
//! - Chinese: "换成"/"兑换成"/"转换为"/"兑成"/"转为" → TokenKind::To
//! - Hindi:   "में" → TokenKind::In  (postposition)
//! - Arabic:  "إلى" → TokenKind::To
//! - German:  "in" (identical to English, no change needed)

use link_calculator::Calculator;

// ═══════════════════════════════════════════════════════════════════════════════
// GERMAN (de) — Conversion keyword "in" is identical to English
// ═══════════════════════════════════════════════════════════════════════════════

/// German: "1000 Dollar in Euro" (USD to EUR using German currency names)
#[test]
fn test_german_dollar_in_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 Dollar in Euro");
    assert!(
        result.success,
        "1000 Dollar in Euro should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// German: "Rubel" should map to RUB
#[test]
fn test_german_rubel_to_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 Rubel in Euro");
    assert!(
        result.success,
        "1000 Rubel in Euro should succeed (RUB to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// German: "Pfund" should map to GBP
#[test]
fn test_german_pfund_in_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 Pfund in Dollar");
    assert!(
        result.success,
        "100 Pfund in Dollar should succeed (GBP to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// German: "Franken" should map to CHF (German spelling, not "Franc")
#[test]
fn test_german_franken_in_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 Franken in Euro");
    assert!(
        result.success,
        "100 Franken in Euro should succeed (CHF to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// German: "Rupie" should map to INR
#[test]
fn test_german_rupie_in_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 Rupie in Dollar");
    assert!(
        result.success,
        "1000 Rupie in Dollar should succeed (INR to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// German: "Rupien" (plural) should map to INR
#[test]
fn test_german_rupien_plural_in_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 Rupien in Euro");
    assert!(
        result.success,
        "1000 Rupien in Euro should succeed (INR to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// FRENCH (fr) — Conversion keyword "en" added to lexer
// ═══════════════════════════════════════════════════════════════════════════════

/// French: "1000 dollars en euros" (USD to EUR, the canonical French form)
#[test]
fn test_french_dollars_en_euros() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 dollars en euros");
    assert!(
        result.success,
        "1000 dollars en euros should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// French preposition "en" should work as the conversion keyword with ISO codes.
#[test]
fn test_french_en_keyword_with_iso_codes() {
    let mut calc = Calculator::new();
    let result_french = calc.calculate_internal("1000 USD en EUR");
    let result_english = calc.calculate_internal("1000 USD in EUR");
    assert!(
        result_french.success,
        "1000 USD en EUR should succeed, got error: {:?}",
        result_french.error
    );
    assert_eq!(
        result_french.result, result_english.result,
        "French 'en' and English 'in' should produce the same result"
    );
}

/// French: "livre" (short form) should map to GBP
/// Note: "livres sterling" (2 tokens) requires parser multi-word support.
/// The single-token "livre" / "livres" forms work.
#[test]
fn test_french_livre_en_euros() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 livres en euros");
    assert!(
        result.success,
        "100 livres en euros should succeed (GBP to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// French: "roupie" (French spelling of rupee) should map to INR
#[test]
fn test_french_roupie_en_euros() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 roupies en euros");
    assert!(
        result.success,
        "1000 roupies en euros should succeed (INR to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// French: "rouble" (short form) should map to RUB
/// Note: "roubles russes" (2 tokens) requires parser multi-word support.
/// The single-token "rouble" / "roubles" forms work.
#[test]
fn test_french_rouble_en_dollars() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 roubles en dollars");
    assert!(
        result.success,
        "1000 roubles en dollars should succeed (RUB to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// French: "franc" (short form) should map to CHF
/// Note: "francs suisses" (2 tokens) requires parser multi-word support.
/// The single-token "franc" / "francs" forms work.
#[test]
fn test_french_franc_en_euros() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 francs en euros");
    assert!(
        result.success,
        "100 francs en euros should succeed (CHF to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// French: "yuans" (plural) should map to CNY
#[test]
fn test_french_yuans_en_euros() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 yuans en euros");
    assert!(
        result.success,
        "1000 yuans en euros should succeed (CNY to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHINESE SIMPLIFIED (zh) — Conversion keywords "换成"/"兑换成" etc. added to lexer
// ═══════════════════════════════════════════════════════════════════════════════

/// Chinese: "1000 美元 换成 欧元" (1000 USD converted to EUR, space-separated for parser)
/// Note: Chinese calculator input uses spaces between tokens since Chinese has no
/// word boundary markers. The conversion keyword "换成" must be space-separated.
#[test]
fn test_chinese_usd_to_eur_huan_cheng() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 美元 换成 欧元");
    assert!(
        result.success,
        "1000 美元 换成 欧元 should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Chinese: "转换为" (zhuǎn huàn wèi) conversion keyword should work
#[test]
fn test_chinese_usd_to_eur_zhuan_huan_wei() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 美元 转换为 欧元");
    assert!(
        result.success,
        "1000 美元 转换为 欧元 should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Chinese: "人民币" (rén mín bì = Renminbi/CNY) should be recognized
#[test]
fn test_chinese_renminbi_to_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 人民币 换成 美元");
    assert!(
        result.success,
        "1000 人民币 换成 美元 should succeed (CNY to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Chinese: "美金" (měi jīn, colloquial for USD) should be recognized
#[test]
fn test_chinese_mei_jin_to_eur() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 美金 换成 欧元");
    assert!(
        result.success,
        "1000 美金 换成 欧元 should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Chinese: "英镑" (yīng bàng = GBP) should be recognized
#[test]
fn test_chinese_gbp_to_cny() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 英镑 换成 人民币");
    assert!(
        result.success,
        "100 英镑 换成 人民币 should succeed (GBP to CNY), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("CNY"),
        "Result should be in CNY, got: {}",
        result.result
    );
}

/// Chinese: "日元" (rì yuán = JPY) should be recognized
#[test]
fn test_chinese_jpy_to_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 日元 换成 美元");
    assert!(
        result.success,
        "10000 日元 换成 美元 should succeed (JPY to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Chinese: "卢布" (lú bù = RUB) should be recognized
#[test]
fn test_chinese_rub_to_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 卢布 换成 美元");
    assert!(
        result.success,
        "1000 卢布 换成 美元 should succeed (RUB to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Chinese: "印度卢比" (yìn dù lú bǐ = INR) full form should be recognized
/// Note: "印度卢比" (4 chars) arrives as a single identifier token since no spaces.
#[test]
fn test_chinese_inr_to_usd() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 印度卢比 换成 美元");
    assert!(
        result.success,
        "1000 印度卢比 换成 美元 should succeed (INR to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Chinese: "兑换成" (duì huàn chéng = exchange into) keyword should work
#[test]
fn test_chinese_dui_huan_cheng_keyword() {
    let mut calc = Calculator::new();
    let result_dui_huan = calc.calculate_internal("1000 美元 兑换成 欧元");
    let result_huan_cheng = calc.calculate_internal("1000 美元 换成 欧元");
    assert!(
        result_dui_huan.success,
        "1000 美元 兑换成 欧元 should succeed, got error: {:?}",
        result_dui_huan.error
    );
    assert_eq!(
        result_dui_huan.result, result_huan_cheng.result,
        "兑换成 and 换成 should produce the same result"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// HINDI (hi) — Conversion postposition "में" added to lexer
// ═══════════════════════════════════════════════════════════════════════════════

/// Hindi: "1000 डॉलर में यूरो" (1000 dollars in euros)
#[test]
fn test_hindi_dollar_mein_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 डॉलर में यूरो");
    assert!(
        result.success,
        "1000 डॉलर में यूरो should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Hindi: "में" postposition should work with ISO codes
#[test]
fn test_hindi_mein_keyword_with_iso_codes() {
    let mut calc = Calculator::new();
    let result_hindi = calc.calculate_internal("1000 USD में EUR");
    let result_english = calc.calculate_internal("1000 USD in EUR");
    assert!(
        result_hindi.success,
        "1000 USD में EUR should succeed, got error: {:?}",
        result_hindi.error
    );
    assert_eq!(
        result_hindi.result, result_english.result,
        "Hindi 'में' and English 'in' should produce the same result"
    );
}

/// Hindi: "रुपये" (plural direct of rupee) should map to INR
#[test]
fn test_hindi_rupaye_mein_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 रुपये में डॉलर");
    assert!(
        result.success,
        "1000 रुपये में डॉलर should succeed (INR to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Hindi: "पाउंड" (pound = GBP) should be recognized
#[test]
fn test_hindi_pound_mein_rupaye() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 पाउंड में रुपये");
    assert!(
        result.success,
        "100 पाउंड में रुपये should succeed (GBP to INR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("INR"),
        "Result should be in INR, got: {}",
        result.result
    );
}

/// Hindi: "येन" (yen = JPY) should be recognized
#[test]
fn test_hindi_yen_mein_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 येन में डॉलर");
    assert!(
        result.success,
        "10000 येन में डॉलर should succeed (JPY to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Hindi: "रूबल" (rubal = RUB) should be recognized
#[test]
fn test_hindi_rubal_mein_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 रूबल में डॉलर");
    assert!(
        result.success,
        "1000 रूबल में डॉलर should succeed (RUB to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARABIC (ar) — Conversion keyword "إلى" added to lexer
// ═══════════════════════════════════════════════════════════════════════════════

/// Arabic: "1000 دولار إلى يورو" (1000 dollars to euros)
#[test]
fn test_arabic_dollar_ila_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 دولار إلى يورو");
    assert!(
        result.success,
        "1000 دولار إلى يورو should succeed (USD to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Arabic: "إلى" keyword should work with ISO codes
#[test]
fn test_arabic_ila_keyword_with_iso_codes() {
    let mut calc = Calculator::new();
    let result_arabic = calc.calculate_internal("1000 USD إلى EUR");
    let result_english = calc.calculate_internal("1000 USD to EUR");
    assert!(
        result_arabic.success,
        "1000 USD إلى EUR should succeed, got error: {:?}",
        result_arabic.error
    );
    assert_eq!(
        result_arabic.result, result_english.result,
        "Arabic 'إلى' and English 'to' should produce the same result"
    );
}

/// Arabic: "روبل" (rubl = RUB) should be recognized
#[test]
fn test_arabic_rubl_ila_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 روبل إلى دولار");
    assert!(
        result.success,
        "1000 روبل إلى دولار should succeed (RUB to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Arabic: "جنيه" (junayh = GBP) should be recognized
#[test]
fn test_arabic_junayh_ila_euro() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 جنيه إلى يورو");
    assert!(
        result.success,
        "100 جنيه إلى يورو should succeed (GBP to EUR), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("EUR"),
        "Result should be in EUR, got: {}",
        result.result
    );
}

/// Arabic: "روبية" (rūbiyya = INR) should be recognized
#[test]
fn test_arabic_rupiya_ila_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 روبية إلى دولار");
    assert!(
        result.success,
        "1000 روبية إلى دولار should succeed (INR to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Arabic: "فرنك" (frank = CHF) should be recognized
#[test]
fn test_arabic_frank_ila_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("100 فرنك إلى دولار");
    assert!(
        result.success,
        "100 فرنك إلى دولار should succeed (CHF to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Arabic: "يوان" (yuan = CNY) should be recognized
#[test]
fn test_arabic_yuan_ila_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("1000 يوان إلى دولار");
    assert!(
        result.success,
        "1000 يوان إلى دولار should succeed (CNY to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

/// Arabic: "ين" (yen = JPY) should be recognized
#[test]
fn test_arabic_yen_ila_dollar() {
    let mut calc = Calculator::new();
    let result = calc.calculate_internal("10000 ين إلى دولار");
    assert!(
        result.success,
        "10000 ين إلى دولار should succeed (JPY to USD), got error: {:?}",
        result.error
    );
    assert!(
        result.result.contains("USD"),
        "Result should be in USD, got: {}",
        result.result
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Parse-level unit tests for CurrencyDatabase::parse_currency()
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod parse_currency_tests {
    use link_calculator::types::CurrencyDatabase;

    fn parse(s: &str) -> Option<String> {
        CurrencyDatabase::parse_currency(s)
    }

    // German
    #[test]
    fn test_parse_german_us_dollar() {
        assert_eq!(parse("us-dollar"), Some("USD".to_string()));
    }

    #[test]
    fn test_parse_german_pfund() {
        assert_eq!(parse("Pfund"), Some("GBP".to_string()));
        assert_eq!(parse("pfund sterling"), Some("GBP".to_string()));
        assert_eq!(parse("britisches pfund"), Some("GBP".to_string()));
    }

    #[test]
    fn test_parse_german_franken() {
        assert_eq!(parse("Franken"), Some("CHF".to_string()));
        assert_eq!(parse("schweizer franken"), Some("CHF".to_string()));
    }

    #[test]
    fn test_parse_german_rubel() {
        assert_eq!(parse("Rubel"), Some("RUB".to_string()));
        assert_eq!(parse("Rubeln"), Some("RUB".to_string()));
        assert_eq!(parse("russischer rubel"), Some("RUB".to_string()));
    }

    #[test]
    fn test_parse_german_rupie() {
        assert_eq!(parse("Rupie"), Some("INR".to_string()));
        assert_eq!(parse("Rupien"), Some("INR".to_string()));
        assert_eq!(parse("indische rupie"), Some("INR".to_string()));
    }

    // French
    #[test]
    fn test_parse_french_dollar_americain() {
        assert_eq!(parse("dollar américain"), Some("USD".to_string()));
        assert_eq!(parse("dollars américains"), Some("USD".to_string()));
        assert_eq!(parse("dollar americain"), Some("USD".to_string()));
    }

    #[test]
    fn test_parse_french_livre_sterling() {
        assert_eq!(parse("livre sterling"), Some("GBP".to_string()));
        assert_eq!(parse("livres sterling"), Some("GBP".to_string()));
        assert_eq!(parse("livre"), Some("GBP".to_string()));
        assert_eq!(parse("livres"), Some("GBP".to_string()));
    }

    #[test]
    fn test_parse_french_roupie() {
        assert_eq!(parse("roupie"), Some("INR".to_string()));
        assert_eq!(parse("roupies"), Some("INR".to_string()));
        assert_eq!(parse("roupie indienne"), Some("INR".to_string()));
    }

    #[test]
    fn test_parse_french_rouble_russe() {
        assert_eq!(parse("rouble russe"), Some("RUB".to_string()));
        assert_eq!(parse("roubles russes"), Some("RUB".to_string()));
    }

    #[test]
    fn test_parse_french_franc_suisse() {
        assert_eq!(parse("franc suisse"), Some("CHF".to_string()));
        assert_eq!(parse("francs suisses"), Some("CHF".to_string()));
    }

    #[test]
    fn test_parse_french_yuans() {
        assert_eq!(parse("yuans"), Some("CNY".to_string()));
        assert_eq!(parse("yuan chinois"), Some("CNY".to_string()));
    }

    // Chinese
    #[test]
    fn test_parse_chinese_usd() {
        assert_eq!(parse("美元"), Some("USD".to_string()));
        assert_eq!(parse("美金"), Some("USD".to_string()));
    }

    #[test]
    fn test_parse_chinese_eur() {
        assert_eq!(parse("欧元"), Some("EUR".to_string()));
    }

    #[test]
    fn test_parse_chinese_gbp() {
        assert_eq!(parse("英镑"), Some("GBP".to_string()));
    }

    #[test]
    fn test_parse_chinese_jpy() {
        assert_eq!(parse("日元"), Some("JPY".to_string()));
        assert_eq!(parse("日圆"), Some("JPY".to_string()));
    }

    #[test]
    fn test_parse_chinese_chf() {
        assert_eq!(parse("瑞士法郎"), Some("CHF".to_string()));
        assert_eq!(parse("法郎"), Some("CHF".to_string()));
    }

    #[test]
    fn test_parse_chinese_cny() {
        assert_eq!(parse("人民币"), Some("CNY".to_string()));
        assert_eq!(parse("元"), Some("CNY".to_string()));
        assert_eq!(parse("块"), Some("CNY".to_string()));
    }

    #[test]
    fn test_parse_chinese_rub() {
        assert_eq!(parse("卢布"), Some("RUB".to_string()));
    }

    #[test]
    fn test_parse_chinese_inr() {
        assert_eq!(parse("卢比"), Some("INR".to_string()));
        assert_eq!(parse("印度卢比"), Some("INR".to_string()));
    }

    // Hindi
    #[test]
    fn test_parse_hindi_usd() {
        assert_eq!(parse("डॉलर"), Some("USD".to_string()));
        assert_eq!(parse("अमेरिकी डॉलर"), Some("USD".to_string()));
    }

    #[test]
    fn test_parse_hindi_eur() {
        assert_eq!(parse("यूरो"), Some("EUR".to_string()));
    }

    #[test]
    fn test_parse_hindi_gbp() {
        assert_eq!(parse("पाउंड"), Some("GBP".to_string()));
        assert_eq!(parse("ब्रिटिश पाउंड"), Some("GBP".to_string()));
        assert_eq!(parse("पाउंड स्टर्लिंग"), Some("GBP".to_string()));
    }

    #[test]
    fn test_parse_hindi_jpy() {
        assert_eq!(parse("येन"), Some("JPY".to_string()));
        assert_eq!(parse("जापानी येन"), Some("JPY".to_string()));
    }

    #[test]
    fn test_parse_hindi_chf() {
        assert_eq!(parse("फ्रैंक"), Some("CHF".to_string()));
        assert_eq!(parse("स्विस फ्रैंक"), Some("CHF".to_string()));
    }

    #[test]
    fn test_parse_hindi_cny() {
        assert_eq!(parse("युआन"), Some("CNY".to_string()));
        assert_eq!(parse("चीनी युआन"), Some("CNY".to_string()));
        assert_eq!(parse("रेनमिनबी"), Some("CNY".to_string()));
    }

    #[test]
    fn test_parse_hindi_rub() {
        assert_eq!(parse("रूबल"), Some("RUB".to_string()));
        assert_eq!(parse("रूसी रूबल"), Some("RUB".to_string()));
        assert_eq!(parse("रशियन रूबल"), Some("RUB".to_string()));
    }

    #[test]
    fn test_parse_hindi_inr() {
        assert_eq!(parse("रुपया"), Some("INR".to_string()));
        assert_eq!(parse("रुपये"), Some("INR".to_string()));
        assert_eq!(parse("रुपयों"), Some("INR".to_string()));
        assert_eq!(parse("भारतीय रुपया"), Some("INR".to_string()));
    }

    // Arabic
    #[test]
    fn test_parse_arabic_usd() {
        assert_eq!(parse("دولار"), Some("USD".to_string()));
        assert_eq!(parse("دولارات"), Some("USD".to_string()));
        assert_eq!(parse("دولار أمريكي"), Some("USD".to_string()));
    }

    #[test]
    fn test_parse_arabic_eur() {
        assert_eq!(parse("يورو"), Some("EUR".to_string()));
    }

    #[test]
    fn test_parse_arabic_gbp() {
        assert_eq!(parse("جنيه إسترليني"), Some("GBP".to_string()));
        assert_eq!(parse("جنيهات إسترلينية"), Some("GBP".to_string()));
        assert_eq!(parse("جنيه"), Some("GBP".to_string()));
        assert_eq!(parse("جنيهات"), Some("GBP".to_string()));
    }

    #[test]
    fn test_parse_arabic_jpy() {
        assert_eq!(parse("ين ياباني"), Some("JPY".to_string()));
        assert_eq!(parse("ين"), Some("JPY".to_string()));
    }

    #[test]
    fn test_parse_arabic_chf() {
        assert_eq!(parse("فرنك سويسري"), Some("CHF".to_string()));
        assert_eq!(parse("فرنكات سويسرية"), Some("CHF".to_string()));
        assert_eq!(parse("فرنك"), Some("CHF".to_string()));
    }

    #[test]
    fn test_parse_arabic_cny() {
        assert_eq!(parse("يوان صيني"), Some("CNY".to_string()));
        assert_eq!(parse("يوان"), Some("CNY".to_string()));
        assert_eq!(parse("رنمينبي"), Some("CNY".to_string()));
    }

    #[test]
    fn test_parse_arabic_rub() {
        assert_eq!(parse("روبل روسي"), Some("RUB".to_string()));
        assert_eq!(parse("روبلات روسية"), Some("RUB".to_string()));
        assert_eq!(parse("روبل"), Some("RUB".to_string()));
    }

    #[test]
    fn test_parse_arabic_inr() {
        assert_eq!(parse("روبية هندية"), Some("INR".to_string()));
        assert_eq!(parse("روبيات هندية"), Some("INR".to_string()));
        assert_eq!(parse("روبية"), Some("INR".to_string()));
        assert_eq!(parse("روبيات"), Some("INR".to_string()));
    }
}
