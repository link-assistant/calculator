//! Tests for issue #222: spelled numbers and operator words as a data-driven
//! front end, so consumers such as link-assistant/formal-ai no longer need
//! their own number-word table and fallback evaluator.
//!
//! The vocabulary lives in `data/words/arithmetic-words.lino`. The cases below
//! are the word-operator expressions formal-ai sends to its fallback today
//! (`multilingual_variations.rs`, `issue_962_word_operator_parity.rs`).

use link_calculator::Calculator;

fn assert_calculates(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal(input);
        assert!(
            result.success,
            "{input:?} should calculate, got error: {:?}",
            result.error
        );
        assert_eq!(result.result, *expected, "{input:?}");
    }
}

#[test]
fn test_issue_222_english_words() {
    assert_calculates(&[
        ("two plus two", "4"),
        ("three plus five", "8"),
        ("seven minus four", "3"),
        ("six times seven", "42"),
        ("nine multiplied by nine", "81"),
        ("ten divided by two", "5"),
        ("eight modulo three", "2"),
        ("one plus two plus three", "6"),
        ("2 plus 2", "4"),
        ("Two Plus Two", "4"),
    ]);
}

#[test]
fn test_issue_222_russian_words() {
    assert_calculates(&[
        ("два плюс два", "4"),
        ("три плюс пять", "8"),
        ("семь минус четыре", "3"),
        ("шесть умножить на семь", "42"),
        ("девять умножить на девять", "81"),
        ("десять разделить на два", "5"),
        ("один плюс два плюс три", "6"),
        ("пять минус один", "4"),
        ("2 плюс 2", "4"),
        ("восемь по модулю три", "2"),
    ]);
}

#[test]
fn test_issue_222_hindi_words() {
    assert_calculates(&[
        ("दो जोड़ दो", "4"),
        ("दो जमा दो", "4"),
        ("सात घटा चार", "3"),
        ("छह गुणा सात", "42"),
        ("दस भाग दो", "5"),
        ("दस बटा दो", "5"),
        ("आठ मॉड्यूलो तीन", "2"),
        ("एक जोड़ दो जोड़ तीन", "6"),
        ("2 जोड़ 2", "4"),
        ("4 घटा 2", "2"),
        ("3 गुणा 2", "6"),
        ("6 भाग 2", "3"),
        ("6 बटा 2", "3"),
    ]);
}

#[test]
fn test_issue_222_chinese_words() {
    assert_calculates(&[
        ("二 加上 二", "4"),
        ("二 加 二", "4"),
        ("七 减去 四", "3"),
        ("七 减 四", "3"),
        ("六 乘以 七", "42"),
        ("六 乘 七", "42"),
        ("十 除以 二", "5"),
        ("十 除 二", "5"),
        ("八 取模 三", "2"),
        ("2 加 2", "4"),
        ("4 减 2", "2"),
        ("3 乘 2", "6"),
        ("6 除 2", "3"),
        // Chinese is written without spaces.
        ("二加二", "4"),
        ("二十三加五", "28"),
        ("一百除以四", "25"),
    ]);
}

#[test]
fn test_issue_222_spanish_words() {
    assert_calculates(&[
        ("2 más 2", "4"),
        ("7 menos 4", "3"),
        ("6 por 7", "42"),
        ("10 dividido por 2", "5"),
        ("10 entre 2", "5"),
        ("8 módulo 3", "2"),
        ("dos más dos", "4"),
        ("seis por siete", "42"),
    ]);
}

#[test]
fn test_issue_222_compound_number_words() {
    assert_calculates(&[
        ("twenty one plus one", "22"),
        ("twenty-one times two", "42"),
        ("one hundred twenty three", "123"),
        ("two thousand twenty six minus six", "2020"),
        ("сто двадцать три плюс семь", "130"),
        ("दो सौ बटा दो", "100"),
        ("三百二十 加 一", "321"),
        ("two to the power of ten", "1024"),
        ("два в степени десять", "1024"),
    ]);
}

/// Words that already meant something else keep that meaning.
#[test]
fn test_issue_222_existing_meanings_are_preserved() {
    assert_calculates(&[
        ("mod(17, 5)", "2"),
        ("17 mod 5", "2"),
        ("10 per 4", "2.5"),
        // "минус" next to a currency, "в" as a conversion keyword.
        ("1 USD в RUB", "89.5 RUB"),
        // Spanish "once" (11) is French for ounce.
        ("1 once", "1 oz"),
    ]);

    // All-uppercase currency codes are not number words.
    let mut calculator = Calculator::new();
    let result = calculator.calculate_internal("1 ONE");
    assert!(result.success, "got error: {:?}", result.error);
    assert_eq!(result.result, "1 ONE");
}

/// Number words keep exact big-integer arithmetic.
#[test]
fn test_issue_222_number_words_are_exact() {
    assert_calculates(&[(
        "one billion times one billion times one billion times one billion",
        "1000000000000000000000000000000000000",
    )]);
}

/// Unicode comparison signs behave like their ASCII spellings.
#[test]
fn test_issue_222_unicode_comparison_signs() {
    assert_calculates(&[
        ("1 + 1 ≠ 3", "true"),
        ("1 + 1 ≠ 2", "false"),
        ("2 ≤ 3", "true"),
        ("3 ≤ 3", "true"),
        ("3 ≥ 4", "false"),
        ("two plus two ≥ four", "true"),
    ]);
}
