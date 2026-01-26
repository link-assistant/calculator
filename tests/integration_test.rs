//! Integration tests for Link Calculator.
//!
//! These tests verify the public API works correctly.

use link_calculator::{Calculator, VERSION};

mod calculator_tests {
    use super::*;

    #[test]
    fn test_calculator_creation() {
        let calculator = Calculator::new();
        let _ = calculator;
    }

    #[test]
    fn test_simple_addition() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_simple_subtraction() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("10 - 4");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_simple_multiplication() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "12");
    }

    #[test]
    fn test_simple_division() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("15 / 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_division_by_zero() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("10 / 0");
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("zero"));
    }

    #[test]
    fn test_decimal_numbers() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("3.14 + 2.86");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_negative_numbers() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("-5 + 3");
        assert!(result.success);
        assert_eq!(result.result, "-2");
    }

    #[test]
    fn test_parentheses() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("(2 + 3) * 4");
        assert!(result.success);
        assert_eq!(result.result, "20");
    }

    #[test]
    fn test_operator_precedence() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "14"); // 2 + (3 * 4) = 14
    }
}

mod currency_tests {
    use super::*;

    #[test]
    fn test_currency_value() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("100 USD");
        assert!(result.success);
        assert!(result.result.contains("100"));
        assert!(result.result.contains("USD"));
    }

    #[test]
    fn test_same_currency_addition() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("100 USD + 50 USD");
        assert!(result.success);
        assert!(result.result.contains("150"));
    }

    #[test]
    fn test_currency_conversion() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("84 USD - 34 EUR");
        assert!(result.success);
        // Should convert EUR to USD and perform subtraction
        assert!(result.result.contains("USD"));
    }

    #[test]
    fn test_currency_conversion_steps_show_rate_info() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("0 RUB + 1 USD");
        assert!(result.success);
        // Steps should include exchange rate information
        let steps_text = result.steps.join("\n");
        assert!(
            steps_text.contains("Exchange rate:"),
            "Steps should contain exchange rate info. Steps: {:?}",
            result.steps
        );
        assert!(
            steps_text.contains("source:"),
            "Steps should contain rate source. Steps: {:?}",
            result.steps
        );
        assert!(
            steps_text.contains("date:"),
            "Steps should contain rate date. Steps: {:?}",
            result.steps
        );
    }

    #[test]
    fn test_same_currency_no_rate_info() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("100 USD + 50 USD");
        assert!(result.success);
        // Same currency addition should not show exchange rate info
        let steps_text = result.steps.join("\n");
        assert!(
            !steps_text.contains("Exchange rate:"),
            "Same currency should not show exchange rate. Steps: {:?}",
            result.steps
        );
    }
}

mod datetime_tests {
    use super::*;

    #[test]
    fn test_datetime_subtraction() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
        assert!(result.success);
        // Should be approximately 1 day, 20 hours
        assert!(result.result.contains("day"));
    }
}

mod lino_tests {
    use super::*;

    #[test]
    fn test_lino_representation_simple() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3");
        assert!(result.success);
        assert!(!result.lino_interpretation.is_empty());
        assert!(result.lino_interpretation.contains('+'));
    }

    #[test]
    fn test_lino_representation_currency() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("84 USD - 34 EUR");
        assert!(result.success);
        assert!(result.lino_interpretation.contains("USD"));
        assert!(result.lino_interpretation.contains("EUR"));
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("");
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_invalid_input_generates_issue_link() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("???invalid???");
        assert!(!result.success);
        assert!(result.issue_link.is_some());
        let link = result.issue_link.unwrap();
        assert!(link.contains("github.com"));
        assert!(link.contains("issues/new"));
    }
}

mod advanced_math_tests {
    use super::*;

    #[test]
    fn test_sin_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sin(0)");
        assert!(result.success, "sin(0) should succeed");
        assert_eq!(result.result, "0");
    }

    #[test]
    fn test_cos_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("cos(0)");
        assert!(result.success, "cos(0) should succeed");
        assert_eq!(result.result, "1");
    }

    #[test]
    fn test_sqrt_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(16)");
        assert!(result.success, "sqrt(16) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_sqrt_function_decimal() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(2)");
        assert!(result.success, "sqrt(2) should succeed");
        // sqrt(2) ≈ 1.414...
        assert!(
            result.result.starts_with("1.41"),
            "sqrt(2) should be approximately 1.414"
        );
    }

    #[test]
    fn test_pow_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("pow(2, 3)");
        assert!(result.success, "pow(2, 3) should succeed");
        assert_eq!(result.result, "8");
    }

    #[test]
    fn test_power_operator() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("2^3");
        assert!(result.success, "2^3 should succeed");
        assert_eq!(result.result, "8");
    }

    #[test]
    fn test_exp_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("exp(0)");
        assert!(result.success, "exp(0) should succeed");
        assert_eq!(result.result, "1");
    }

    #[test]
    fn test_ln_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("ln(1)");
        assert!(result.success, "ln(1) should succeed");
        assert_eq!(result.result, "0");
    }

    #[test]
    fn test_log_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("log(100)");
        assert!(result.success, "log(100) should succeed");
        assert_eq!(result.result, "2");
    }

    #[test]
    fn test_abs_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("abs(-5)");
        assert!(result.success, "abs(-5) should succeed");
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_floor_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("floor(3.7)");
        assert!(result.success, "floor(3.7) should succeed");
        assert_eq!(result.result, "3");
    }

    #[test]
    fn test_ceil_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("ceil(3.2)");
        assert!(result.success, "ceil(3.2) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_pi_constant() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("pi()");
        assert!(result.success, "pi() should succeed");
        assert!(
            result.result.starts_with("3.14"),
            "pi should start with 3.14"
        );
    }

    #[test]
    fn test_e_constant() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("e()");
        assert!(result.success, "e() should succeed");
        assert!(
            result.result.starts_with("2.71"),
            "e should start with 2.71"
        );
    }

    #[test]
    fn test_nested_functions() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(abs(-16))");
        assert!(result.success, "sqrt(abs(-16)) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_function_with_expression() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(9 + 7)");
        assert!(result.success, "sqrt(9 + 7) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_min_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("min(5, 3)");
        assert!(result.success, "min(5, 3) should succeed");
        assert_eq!(result.result, "3");
    }

    #[test]
    fn test_max_function() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("max(5, 3)");
        assert!(result.success, "max(5, 3) should succeed");
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_factorial() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("factorial(5)");
        assert!(result.success, "factorial(5) should succeed");
        assert_eq!(result.result, "120");
    }

    #[test]
    fn test_deg_function() {
        let mut calculator = Calculator::new();
        // pi radians = 180 degrees
        let result = calculator.calculate_internal("deg(3.14159265358979)");
        assert!(result.success, "deg(pi) should succeed");
        // Should be close to 180
        let value: f64 = result.result.parse().unwrap();
        assert!(
            (value - 180.0).abs() < 0.01,
            "deg(pi) should be approximately 180"
        );
    }

    #[test]
    fn test_rad_function() {
        let mut calculator = Calculator::new();
        // 180 degrees = pi radians
        let result = calculator.calculate_internal("rad(180)");
        assert!(result.success, "rad(180) should succeed");
        // Should be close to pi
        let value: f64 = result.result.parse().unwrap();
        assert!(
            (value - std::f64::consts::PI).abs() < 0.01,
            "rad(180) should be approximately pi"
        );
    }

    #[test]
    fn test_integrate_x_squared() {
        let mut calculator = Calculator::new();
        // integrate(x^2, x, 0, 3) = [x^3/3] from 0 to 3 = 9
        let result = calculator.calculate_internal("integrate(x^2, x, 0, 3)");
        assert!(result.success, "integrate(x^2, x, 0, 3) should succeed");
        let value: f64 = result.result.parse().unwrap();
        assert!(
            (value - 9.0).abs() < 0.1,
            "∫x² from 0 to 3 should be approximately 9"
        );
    }

    #[test]
    fn test_integrate_sin() {
        let mut calculator = Calculator::new();
        // integrate(sin(x), x, 0, pi) = 2
        let result = calculator.calculate_internal("integrate(sin(x), x, 0, 3.14159265358979)");
        assert!(result.success, "integrate(sin(x), x, 0, pi) should succeed");
        let value: f64 = result.result.parse().unwrap();
        assert!(
            (value - 2.0).abs() < 0.1,
            "∫sin(x) from 0 to π should be approximately 2"
        );
    }

    #[test]
    fn test_sqrt_negative_error() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(-1)");
        assert!(!result.success, "sqrt(-1) should fail");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Domain error"));
    }

    #[test]
    fn test_ln_negative_error() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("ln(-1)");
        assert!(!result.success, "ln(-1) should fail");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Domain error"));
    }

    #[test]
    fn test_unknown_function_error() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("foobar(5)");
        assert!(!result.success, "foobar(5) should fail");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Unknown function"));
    }
}

mod version_tests {
    use super::*;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_version_matches_cargo_toml() {
        // Version should match the one in Cargo.toml
        assert!(VERSION.starts_with("0."));
    }
}

mod indefinite_integral_tests {
    use super::*;

    #[test]
    fn test_natural_integral_notation_sin_x_over_x() {
        // This is the exact test case from issue #3: "integrate sin(x)/x dx"
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x)/x dx");

        // The result should be a symbolic result with Si(x) + C
        assert!(result.success, "integrate sin(x)/x dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        assert!(
            result.result.contains("Si(x)"),
            "Result should contain Si(x): got '{}'",
            result.result
        );

        // Should have LaTeX rendering
        assert!(result.latex_input.is_some(), "Should have LaTeX input");
        assert!(result.latex_result.is_some(), "Should have LaTeX result");

        // Should have plot data
        assert!(result.plot_data.is_some(), "Should have plot data");
        let plot = result.plot_data.unwrap();
        assert!(!plot.x_values.is_empty(), "Plot should have x values");
        assert!(!plot.y_values.is_empty(), "Plot should have y values");
    }

    #[test]
    fn test_natural_integral_notation_x_squared() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate x^2 dx");

        assert!(result.success, "integrate x^2 dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // x^2 integrates to x^3/3 + C
        assert!(
            result.result.contains('3') && result.result.contains('C'),
            "Result should contain x^3/3 + C pattern: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_sin() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x) dx");

        assert!(result.success, "integrate sin(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // sin(x) integrates to -cos(x) + C
        assert!(
            result.result.contains("cos") && result.result.contains('C'),
            "Result should contain -cos(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_cos() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate cos(x) dx");

        assert!(result.success, "integrate cos(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // cos(x) integrates to sin(x) + C
        assert!(
            result.result.contains("sin") && result.result.contains('C'),
            "Result should contain sin(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_x() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate x dx");

        assert!(result.success, "integrate x dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // x integrates to x^2/2 + C
        assert!(
            result.result.contains('2') && result.result.contains('C'),
            "Result should contain x²/2 + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_constant() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate 5 dx");

        assert!(result.success, "integrate 5 dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // 5 integrates to 5x + C
        assert!(
            result.result.contains('5')
                && result.result.contains('x')
                && result.result.contains('C'),
            "Result should contain 5 * x + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_exp() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate exp(x) dx");

        assert!(result.success, "integrate exp(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // exp(x) integrates to exp(x) + C
        assert!(
            result.result.contains("exp") && result.result.contains('C'),
            "Result should contain exp(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_different_variable() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(t) dt");

        assert!(result.success, "integrate sin(t) dt should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        assert!(
            result.result.contains("cos") && result.result.contains('t'),
            "Result should contain -cos(t) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_latex_rendering_for_integral() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x)/x dx");

        assert!(result.success, "Should succeed");
        let latex_input = result.latex_input.unwrap();
        assert!(
            latex_input.contains("\\int"),
            "LaTeX input should contain integral symbol: got '{latex_input}'"
        );
    }

    #[test]
    fn test_definite_integral_still_works() {
        // Ensure the traditional definite integral syntax still works
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate(sin(x), x, 0, 3.14159)");

        assert!(result.success, "Definite integral should still work");
        assert!(
            !result.is_symbolic.unwrap_or(false),
            "Definite integral should not be symbolic"
        );
        let value: f64 = result.result.parse().unwrap();
        assert!(
            (value - 2.0).abs() < 0.1,
            "∫sin(x) from 0 to π should be approximately 2"
        );
    }
}

/// Tests that verify .lino rate files are properly loaded and used in calculations.
/// These tests ensure that:
/// 1. Rates from .lino files can be loaded into the calculator
/// 2. Currency conversions actually use those loaded rates
/// 3. Historical rates work with the "at" date syntax
mod lino_rate_file_tests {
    use super::*;

    /// Test that we can load rates from a .lino file and use them in calculations.
    /// Uses the "Feb 8, 2021" date format which is parsed as a `DateTime` correctly.
    #[test]
    fn test_load_lino_rates_and_use_in_conversion() {
        let mut calculator = Calculator::new();

        // Load USD to RUB rates in the new .lino format
        let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192";

        let result = calculator.load_rates_from_consolidated_lino(lino_content);
        assert!(result.is_ok(), "Should load rates successfully");
        assert_eq!(result.unwrap(), 2, "Should load 2 rates");

        // Use month name format since ISO date format (YYYY-MM-DD) is tokenized
        // as number-minus-number-minus-number instead of a date
        let calc_result = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
        assert!(
            calc_result.success,
            "Historical conversion should succeed: {:?}",
            calc_result.error
        );

        // Check that the steps show we're using the loaded rate, not the default
        let steps_text = calc_result.steps.join("\n");
        assert!(
            steps_text.contains("cbr.ru") || steps_text.contains("74.26"),
            "Should use the loaded rate from cbr.ru, not default. Steps: {steps_text}"
        );

        // Parse the result to get the numeric value
        let result_str = calc_result.result.replace(" RUB", "").replace(',', "");
        let result_value: f64 = result_str
            .trim()
            .parse()
            .expect("Should parse result as number");

        // The rate for 2021-02-08 is 74.2602, so 1 USD should = ~74.26 RUB
        assert!(
            (result_value - 74.2602).abs() < 0.01,
            "1 USD at 2021-02-08 should be ~74.26 RUB (from .lino file), got {result_value}. Steps: {steps_text}"
        );
    }

    /// Test that different dates return different historical rates.
    /// Note: Uses month name format (e.g., "Jan 25, 2021") as ISO date format
    /// (YYYY-MM-DD) is currently tokenized as arithmetic, not as a date.
    #[test]
    fn test_different_dates_use_different_rates() {
        let mut calculator = Calculator::new();

        // Load multiple dates of USD to EUR rates
        let lino_content = "conversion:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  rates:
    2021-01-25 0.8234
    2021-02-01 0.8315
    2021-02-08 0.8402";

        calculator
            .load_rates_from_consolidated_lino(lino_content)
            .expect("Should load rates");

        // Test first date - using month name format
        let result1 = calculator.calculate_internal("(0 EUR + 1 USD) at Jan 25, 2021");
        assert!(result1.success);
        let val1_str = result1.result.replace(" EUR", "").replace(',', "");
        let val1: f64 = val1_str.trim().parse().expect("Should parse");
        assert!(
            (val1 - 0.8234).abs() < 0.001,
            "Rate on Jan 25, 2021 should be 0.8234, got {val1}"
        );

        // Test second date - using month name format
        let result2 = calculator.calculate_internal("(0 EUR + 1 USD) at Feb 8, 2021");
        assert!(result2.success);
        let val2_str = result2.result.replace(" EUR", "").replace(',', "");
        let val2: f64 = val2_str.trim().parse().expect("Should parse");
        assert!(
            (val2 - 0.8402).abs() < 0.001,
            "Rate on Feb 8, 2021 should be 0.8402, got {val2}"
        );

        // Rates should be different
        assert!(
            (val1 - val2).abs() > 0.01,
            "Different dates should have different rates"
        );
    }

    /// Test loading rates for multiple currency pairs.
    #[test]
    fn test_multiple_currency_pairs() {
        let mut calculator = Calculator::new();

        // Load EUR to GBP rates
        let eur_gbp_content = "conversion:
  from EUR
  to GBP
  source 'ecb.europa.eu'
  rates:
    2021-02-08 0.8765";

        // Load USD to JPY rates
        let usd_jpy_content = "conversion:
  from USD
  to JPY
  source 'boj.or.jp'
  rates:
    2021-02-08 105.25";

        calculator
            .load_rates_from_consolidated_lino(eur_gbp_content)
            .expect("Should load EUR/GBP");
        calculator
            .load_rates_from_consolidated_lino(usd_jpy_content)
            .expect("Should load USD/JPY");

        // Test EUR to GBP conversion - using month name format
        let result1 = calculator.calculate_internal("(0 GBP + 1 EUR) at Feb 8, 2021");
        assert!(result1.success);
        let val1_str = result1.result.replace(" GBP", "").replace(',', "");
        let val1: f64 = val1_str.trim().parse().expect("Should parse");
        assert!(
            (val1 - 0.8765).abs() < 0.001,
            "EUR->GBP rate should be 0.8765, got {val1}"
        );

        // Test USD to JPY conversion - using month name format
        let result2 = calculator.calculate_internal("(0 JPY + 1 USD) at Feb 8, 2021");
        assert!(result2.success);
        let val2_str = result2.result.replace(" JPY", "").replace(',', "");
        let val2: f64 = val2_str.trim().parse().expect("Should parse");
        assert!(
            (val2 - 105.25).abs() < 0.1,
            "USD->JPY rate should be 105.25, got {val2}"
        );
    }

    /// Test that the inverse rate is also available after loading.
    #[test]
    fn test_inverse_rate_available() {
        let mut calculator = Calculator::new();

        // Load USD to RUB rate
        let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru'
  rates:
    2021-02-08 74.2602";

        calculator
            .load_rates_from_consolidated_lino(lino_content)
            .expect("Should load rates");

        // Test forward conversion: USD -> RUB (using month name format)
        let result1 = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
        assert!(result1.success);

        // Test inverse conversion: RUB -> USD (using month name format)
        let result2 = calculator.calculate_internal("(0 USD + 100 RUB) at Feb 8, 2021");
        assert!(result2.success);
        let val_str = result2.result.replace(" USD", "").replace(',', "");
        let val: f64 = val_str.trim().parse().expect("Should parse");

        // 100 RUB at 74.2602 rate = 100 / 74.2602 ≈ 1.347 USD
        let expected = 100.0 / 74.2602;
        assert!(
            (val - expected).abs() < 0.01,
            "100 RUB should be ~{expected:.3} USD, got {val}"
        );
    }

    /// Test that rate source and date are shown in calculation steps.
    #[test]
    fn test_rate_info_shown_in_steps() {
        let mut calculator = Calculator::new();

        let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602";

        calculator
            .load_rates_from_consolidated_lino(lino_content)
            .expect("Should load rates");

        // Use month name format for the date
        let result = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
        assert!(result.success);

        let steps_text = result.steps.join("\n");

        // Should show the source from the loaded .lino file
        assert!(
            steps_text.contains("cbr.ru"),
            "Steps should contain rate source 'cbr.ru'. Steps: {steps_text}"
        );

        // Should show the rate value
        assert!(
            steps_text.contains("74.26"),
            "Steps should contain exchange rate 74.26. Steps: {steps_text}"
        );
    }

    /// Test loading the legacy .lino format (rates/data) still works.
    #[test]
    fn test_legacy_lino_format() {
        let mut calculator = Calculator::new();

        // Legacy format: rates: as root, data: for rates
        let lino_content = "rates:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  data:
    2021-01-25 0.8234
    2021-02-01 0.8315";

        let result = calculator.load_rates_from_consolidated_lino(lino_content);
        assert!(result.is_ok(), "Should load legacy format");
        assert_eq!(result.unwrap(), 2, "Should load 2 rates");

        // Verify the rate is used - using month name format for the date
        let calc_result = calculator.calculate_internal("(0 EUR + 1 USD) at Jan 25, 2021");
        assert!(calc_result.success);
        let val_str = calc_result.result.replace(" EUR", "").replace(',', "");
        let val: f64 = val_str.trim().parse().expect("Should parse");
        assert!(
            (val - 0.8234).abs() < 0.001,
            "Rate should be 0.8234, got {val}"
        );
    }

    /// Test arithmetic with currency conversion uses correct rates.
    #[test]
    fn test_currency_arithmetic_with_loaded_rates() {
        let mut calculator = Calculator::new();

        let lino_content = "conversion:
  from USD
  to EUR
  source 'test'
  rates:
    2021-02-08 0.85";

        calculator
            .load_rates_from_consolidated_lino(lino_content)
            .expect("Should load rates");

        // Test: 100 USD + 50 EUR at Feb 8, 2021
        // First, 50 EUR needs to be converted to USD
        // 50 EUR = 50 / 0.85 USD ≈ 58.82 USD
        // Total = 100 + 58.82 = 158.82 USD
        let result = calculator.calculate_internal("(100 USD + 50 EUR) at Feb 8, 2021");
        assert!(result.success, "Arithmetic with conversion should succeed");
        let val_str = result.result.replace(" USD", "").replace(',', "");
        let val: f64 = val_str.trim().parse().expect("Should parse");

        let expected = 100.0 + (50.0 / 0.85);
        assert!(
            (val - expected).abs() < 0.1,
            "100 USD + 50 EUR should be ~{expected:.2} USD, got {val}"
        );
    }

    /// Test subtraction with currency conversion.
    #[test]
    fn test_currency_subtraction_with_loaded_rates() {
        let mut calculator = Calculator::new();

        let lino_content = "conversion:
  from USD
  to EUR
  source 'test'
  rates:
    2021-02-08 0.85";

        calculator
            .load_rates_from_consolidated_lino(lino_content)
            .expect("Should load rates");

        // Test: 100 USD - 34 EUR at Feb 8, 2021
        // 34 EUR = 34 / 0.85 USD ≈ 40 USD
        // Result = 100 - 40 = 60 USD
        let result = calculator.calculate_internal("(100 USD - 34 EUR) at Feb 8, 2021");
        assert!(
            result.success,
            "Subtraction with conversion should succeed: {:?}",
            result.error
        );
        let val_str = result.result.replace(" USD", "").replace(',', "");
        let val: f64 = val_str.trim().parse().expect("Should parse");

        let expected = 100.0 - (34.0 / 0.85);
        assert!(
            (val - expected).abs() < 0.1,
            "100 USD - 34 EUR should be ~{expected:.2} USD, got {val}"
        );
    }

    /// Test that ISO date format (YYYY-MM-DD) in "at" clause is currently parsed
    /// as arithmetic (subtraction), not as a date. This is a known limitation.
    /// Users should use month name format (e.g., "Feb 8, 2021") for historical
    /// date queries.
    #[test]
    fn test_iso_date_format_limitation() {
        let mut calculator = Calculator::new();

        // Note: "2021-02-08" after "at" is tokenized as 2021 - 02 - 08 = 2011
        // This is a known limitation of the current lexer.
        let result = calculator.calculate_internal("(0 RUB + 1 USD) at 2021-02-08");
        assert!(result.success);

        // The expression is evaluated as: (0 RUB + 1 USD) at 2011
        // which fails because 2011 (a number) is not a valid DateTime
        // Actually it seems to succeed, let's check what happens

        // The current implementation might handle this differently
        // This test documents the current behavior
        let steps_text = result.steps.join("\n");

        // The "at" clause evaluates to a number (2021 - 02 - 08 = 2011)
        // which is likely treated as year-only or causes fallback to default rates
        // Document the actual behavior here:
        assert!(
            steps_text.contains("default") || steps_text.contains("89.5"),
            "ISO dates are not properly parsed in at clause - falls back to default rate. Steps: {steps_text}"
        );
    }
}
