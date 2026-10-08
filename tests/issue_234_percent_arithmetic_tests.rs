//! Percentage arithmetic and natural-language regressions for issue #234.
use link_calculator::grammar::ExpressionParser;
use link_calculator::Calculator;

fn assert_result(expression: &str, expected: &str) {
    let result = Calculator::new().calculate_internal(expression);
    assert!(result.success, "{expression:?}: {:?}", result.error);
    assert_eq!(result.result, expected, "{expression:?}");
}

#[test]
fn percentage_increases_and_decreases_compound() {
    for (expression, expected) in [
        ("200 + 10%", "220"),
        ("200 - 10%", "180"),
        ("6 - 50%", "3"),
        ("100 + 10% + 10%", "121"),
        ("100 - 10% - 10%", "81"),
        ("100 + (10 + 10)%", "120"),
        ("0.1 + 5%", "0.105"),
        ("$200 + 10%", "220 USD"),
        ("10 meters + 20%", "12 meters"),
        ("1000 + (-12%)", "880"),
        ("200 + (10% * 2)", "200.2"),
    ] {
        assert_result(expression, expected);
    }
}

#[test]
fn percentages_retain_their_display_and_exact_fraction() {
    for (expression, expected) in [
        ("5%", "5%"),
        ("10% + 20%", "30%"),
        ("90% - 40%", "50%"),
        ("5% + 1", "105%"),
        ("30% + 0.4", "70%"),
        ("100% - 1/2", "50%"),
        ("100% + 2 + 30%", "330%"),
        ("-15%", "-15%"),
        ("5% * 5%", "0.25%"),
        ("50% * 30", "15"),
        ("100 / 10%", "1000"),
        ("10% / 2%", "5"),
    ] {
        assert_result(expression, expected);
    }
    let (value, _, _) = ExpressionParser::new()
        .parse_and_evaluate("(1/3)%")
        .unwrap();
    assert_eq!(value.to_rational().unwrap().to_fraction_string(), "1/300");
}

#[test]
fn percentage_questions_and_conversions() {
    for (expression, expected) in [
        ("10% on 200", "220"),
        ("10% off 200", "180"),
        ("20 is 10% of what", "200"),
        ("180 is 10% off what", "200"),
        ("220 is 10% on what", "200"),
        ("50 to 75 is what %", "50%"),
        ("40 to 90 as %", "125%"),
        ("180 is what % off 200", "10%"),
        ("180 is what % on 150", "20%"),
        ("20 is what % of 200", "10%"),
        ("20 as a % of 200", "10%"),
        ("20/200 as %", "10%"),
        ("20/200 %", "10%"),
        ("0.35 as %", "35%"),
        ("2/5 as percent", "40%"),
        ("3/20 is what %", "15%"),
        ("3 in 20 is what %", "15%"),
        ("5% to %", "5%"),
        ("20% as dec", "0.2"),
        ("50% as fraction", "1/2"),
        ("5% is 1 in what", "1/20"),
        ("40% is 1 in what", "1/2.5"),
        ("2/3 of 600", "400"),
        ("50 is 1/5 of what", "250"),
        ("20/5 as multiplier", "4x"),
        ("50 as x of 5", "10x"),
        ("2 as multiplier of 1", "2x"),
        ("2 as multiplier on 1", "1x"),
        ("1 as x off 2", "0.5x"),
        ("50 to 75 is what x", "1.5x"),
        ("20 to 40 as x", "2x"),
    ] {
        assert_result(expression, expected);
    }
}

#[test]
fn multilingual_percentage_phrases_are_data_driven() {
    for (expression, expected) in [
        ("ten percent of fifty", "5"),
        ("twenty-five percent of two hundred", "50"),
        ("80 increased by 25 percent", "100"),
        ("an increase of 25% from 80", "100"),
        ("700 увеличить на 30 процентов", "910"),
        ("тридцать процентов от 700", "210"),
        ("700的百分之三十", "210"),
        ("百分之三十", "30%"),
        ("700の30パーセント", "210"),
        ("30パーセント", "30%"),
        ("what is 28% of 40", "11.2"),
        ("6 is what percent of 24", "25%"),
        ("45 + 15% =", "51.75"),
        ("120% 2", "2.4"),
        ("56.7% of 1,234 participants", "699.678 participants"),
    ] {
        assert_result(expression, expected);
    }
}

#[test]
fn percent_of_modulo_and_numeric_functions_remain_compatible() {
    for (expression, expected) in [
        ("8% of $50", "4 USD"),
        ("38% от 100к", "38000"),
        ("2 + 5% of 200", "12"),
        ("(2 + 5)% of 200", "14"),
        ("5%4", "1"),
        ("8 % 3", "2"),
        ("50%(13)", "11"),
        ("8 mod 3", "2"),
        ("5 modulo 4", "1"),
        // Existing custom-unit fallbacks must survive without percent vocabulary.
        ("3的平方加4的平方", "3 的平方加4的平方"),
        ("十の三乗", "10 の三乗"),
        ("0off", "0 OFF"),
        ("50% / 100", "0.005"),
        ("1% to unitless", "0.01"),
        ("50%%", "0.5%"),
        ("% 1", "1%"),
        ("200% ** 2", "4"),
        ("10 - x% = 8", "x = 20"),
    ] {
        assert_result(expression, expected);
    }
}

#[test]
fn invalid_percentage_queries_report_errors() {
    for expression in [
        "20 is 0% of what",
        "180 is 100% off what",
        "20 as % of 0",
        "5% + $1",
        "1 USD%",
        "10% of",
        "50 to 75 is what % junk",
        "3 permutations of 10",
        "20% is 0, what is 750",
        "if 20 is 0%, what is 60%",
        "if 20 is 30, what is 60",
    ] {
        let result = Calculator::new().calculate_internal(expression);
        assert!(!result.success, "{expression:?}: {}", result.result);
    }
}

#[test]
fn percentages_preserve_compound_units_from_number_separator_grammar() {
    for (expression, expected) in [
        ("20% of 50 km/h", "10 km/h"),
        ("50 km/h + 20%", "60 km/h"),
        ("50 km/h - 20%", "40 km/h"),
        ("50 km/h * 20%", "10 km/h"),
    ] {
        assert_result(expression, expected);
    }
    for expression in ["20% * (10 km/h * 3 m/s)", "20% of (10 km/h)^2"] {
        assert!(!Calculator::new().calculate_internal(expression).success);
    }
}

#[test]
fn percentages_survive_serialization_and_lino_interpretation() {
    use link_calculator::types::{Rational, Value, ValueKind};
    let percent = Value::percent(Rational::new(1, 10));
    assert!(percent.is_number());
    assert_eq!(percent.to_fraction_string().as_deref(), Some("1/10"));
    assert_eq!(percent.type_name(), "percent");
    let serialized = serde_json::to_string(&percent).unwrap();
    let decoded: Value = serde_json::from_str(&serialized).unwrap();
    assert!(matches!(decoded.kind, ValueKind::Percent(_)));
    assert_eq!(decoded, Value::rational(Rational::new(1, 10)));
    for expression in [
        "200 + 10%",
        "10% + 20%",
        "50%",
        "20/200 as %",
        "5% * 5%",
        "200 + Percent(10)",
        "AS_PERCENT(0.1)",
        "20% is 500, what is 750",
        "if 20 is 30%, what is 60%",
    ] {
        let result = Calculator::new().calculate_internal(expression);
        let restored = Calculator::new().calculate_internal(&result.lino_interpretation);
        assert!(
            restored.success,
            "{}: {:?}",
            result.lino_interpretation, restored.error
        );
        assert_eq!(restored.result, result.result, "{expression}");
    }
}

#[test]
fn percentage_constructors_follow_case_insensitive_function_conventions() {
    for (expression, expected) in [
        ("200 + Percent(10)", "220"),
        ("AS_PERCENT(0.1)", "10%"),
        ("As_Fraction(0.5)", "1/2"),
        ("As_Reciprocal(0.5)", "1/2"),
        ("As_Multiplier(2)", "2x"),
        ("10 - Percent(x) = 8", "x = 20"),
    ] {
        assert_result(expression, expected);
    }
}

#[test]
fn percentage_proportion_questions_from_expanded_documentation() {
    for (expression, expected) in [
        ("20% is 500, what is 750", "30%"),
        ("if 20 is 30%, what is 60%", "40"),
    ] {
        assert_result(expression, expected);
    }
}
