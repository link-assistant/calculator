//! Regression coverage for localized named constants (issue #267).

use link_calculator::Calculator;
use std::collections::HashMap;

fn calculate(input: &str) -> String {
    let result = Calculator::new().calculate_internal(input);
    assert!(result.success, "{input:?}: {:?}", result.error);
    result.result
}

fn assert_close(input: &str, expected: f64) {
    let actual = calculate(input).parse::<f64>().unwrap();
    assert!((actual - expected).abs() < 1e-12, "{input:?}: {actual}");
}

#[test]
fn golden_ratio_inside_arithmetic() {
    assert_close("0.65 - (1 / Golden ratio)", 0.031_966_011_250_105);
    assert_close("0.65 - (1 / the golden ratio)", 0.031_966_011_250_105);
    assert_close("golden ratio - 1/(golden ratio)", 1.0);
    assert_close("2 GOLDEN\u{202f}RATIO", 3.236_067_977_499_79);
    assert_close("φ² - ϕ", 1.0);
    assert_close("max(1, Golden ratio)", 1.618_033_988_749_895);
    assert_close("золотое сечение**2 - 黄金比例", 1.0);
    assert_close("二乘以黄金比例加黄金比例", 4.854_101_966_249_685);
}

#[test]
fn golden_ratio_in_every_corpus_language() {
    for alias in [
        "Golden ratio",
        "золотое сечение",
        "Goldener Schnitt",
        "nombre d’or",
        "proporción áurea",
        "黄金比例",
        "黄金比",
        "proporção áurea",
        "स्वर्ण अनुपात",
        "النسبة الذهبية",
        "sezione aurea",
        "황금비",
        "ALTIN ORAN",
        "gulden snede",
        "złoty podział",
        "золотий перетин",
        "gyllene snittet",
        "نسبت طلایی",
        "rasio emas",
        "tỷ lệ vàng",
        "อัตราส่วนทองคำ",
        "יחס הזהב",
        "সোনালী অনুপাত",
    ] {
        assert_close(&format!("1 / ({alias})"), 0.618_033_988_749_895);
    }
    assert_close("1 / 黃金比例", 0.618_033_988_749_895);
}

#[test]
fn related_mathematical_constants_and_equations() {
    assert_close("Section dorée + silberner Schnitt", 4.032_247_551_122_99);
    assert_close("τ / nombre pi", 2.0);
    assert_close("the value of Euler’s number", std::f64::consts::E);
    assert_close("constante d’Euler‑Mascheroni", 0.577_215_664_901_532_9);
    assert_eq!(calculate("x + Golden ratio = Golden ratio + 2"), "x = 2");
}

#[test]
fn localized_names_have_canonical_lino_and_math_notation() {
    let (expression, _, _, lino) = Calculator::new()
        .calculate_with_value("2 × золотое сечение")
        .unwrap();
    assert_eq!(lino, "(2 * (golden_ratio))");
    assert!(expression.to_latex().contains("\\varphi"));
}

#[test]
fn bundled_aliases_resolve_to_their_canonical_constants() {
    let mut canonical_results = HashMap::new();
    for line in include_str!("../data/words/named-constants.lino").lines() {
        let Some(entry) = line.strip_prefix("(constant ") else {
            continue;
        };
        let (_, entry) = entry.split_once(' ').unwrap();
        let (name, alias) = entry.strip_suffix("')").unwrap().split_once(" '").unwrap();
        let expected = canonical_results
            .entry(name)
            .or_insert_with(|| calculate(name));
        assert_eq!(&calculate(alias), expected, "{alias:?}");
    }
}

#[test]
fn named_constants_preserve_dimensions() {
    assert_eq!(
        calculate("speed of light in vacuum in km/s"),
        "299792.458 km/s"
    );
    assert_eq!(
        calculate("1 kg * (скорость света в вакууме)^2 in J"),
        "89875517873681764 J"
    );
    assert_eq!(
        calculate("Avogadro's constant * 1 mol"),
        "602214076000000000000000"
    );
}

#[test]
fn aliases_match_whole_names() {
    for invalid in [
        "golden ratios",
        "golden ratio_x",
        "黄金比例表",
        "phi2",
        "Golden ratio(1)",
    ] {
        assert!(
            !Calculator::new().calculate_internal(invalid).success,
            "{invalid:?}"
        );
    }
    assert_close("e + π", std::f64::consts::E + std::f64::consts::PI);
    assert_close("e() + pi()", std::f64::consts::E + std::f64::consts::PI);
    assert_eq!(calculate("5 min to s"), "300 seconds");
}
