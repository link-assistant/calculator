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

    /// Test for issue #30: Links notation should use exactly 2 outer parentheses,
    /// not 3. Input: (datetime) - (datetime) should produce ((datetime) - (datetime))
    #[test]
    fn test_datetime_subtraction_lino_notation_issue_30() {
        let mut calculator = Calculator::new();
        let result = calculator.calculate_internal("(Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)");
        assert!(result.success);

        // The lino notation should be ((Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC))
        // NOT (((Jan 27, 8:59am UTC) - (Jan 26, 10:20am UTC)))
        let lino = &result.lino_interpretation;

        // Count leading and trailing parentheses
        let leading_parens = lino.chars().take_while(|&c| c == '(').count();
        let trailing_parens = lino.chars().rev().take_while(|&c| c == ')').count();

        assert_eq!(
            leading_parens, 2,
            "Should have exactly 2 leading parentheses in lino notation, but got {leading_parens}: '{lino}'"
        );
        assert_eq!(
            trailing_parens, 2,
            "Should have exactly 2 trailing parentheses in lino notation, but got {trailing_parens}: '{lino}'"
        );

        // Also verify the overall structure
        assert!(
            lino.starts_with("((") && lino.ends_with("))"),
            "Lino notation should start with '((' and end with '))': '{lino}'"
        );
        assert!(
            !lino.starts_with("((("),
            "Lino notation should NOT start with '(((': '{lino}'"
        );
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

/// Tests for the `update_rates_from_api` method (Issue #18 fix)
mod api_rate_update_tests {
    use super::*;

    #[test]
    fn test_update_rates_from_api_returns_count() {
        let mut calc = Calculator::new();

        // Create a JSON string simulating API response
        let rates_json = r#"{"eur": 0.92, "gbp": 0.79, "rub": 75.5}"#;

        let count = calc.update_rates_from_api("USD", "2026-01-26", rates_json);

        // Should have updated 3 rates
        assert_eq!(count, 3);
    }

    #[test]
    fn test_update_rates_from_api_invalid_json() {
        let mut calc = Calculator::new();

        // Invalid JSON should return 0
        let count = calc.update_rates_from_api("USD", "2026-01-26", "invalid json");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_rates_from_api_empty() {
        let mut calc = Calculator::new();

        // Empty rates object should return 0
        let count = calc.update_rates_from_api("USD", "2026-01-26", "{}");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_update_rates_from_api_skips_same_currency() {
        let mut calc = Calculator::new();

        // USD to USD should be skipped
        let rates_json = r#"{"usd": 1.0, "eur": 0.92}"#;
        let count = calc.update_rates_from_api("USD", "2026-01-26", rates_json);

        // Should only update EUR, not USD (1 rate)
        assert_eq!(count, 1);
    }

    #[test]
    fn test_calculation_uses_updated_api_rate() {
        let mut calc = Calculator::new();

        // Update with a specific rate for testing (100 RUB per USD)
        let rates_json = r#"{"rub": 100.0}"#;
        calc.update_rates_from_api("USD", "2026-01-26", rates_json);

        // Calculate 1 USD in RUB - should use the API rate
        let result = calc.calculate_internal("0 RUB + 1 USD");
        assert!(result.success);
        assert_eq!(result.result, "100 RUB");
    }

    #[test]
    fn test_calculation_steps_show_api_rate_source() {
        let mut calc = Calculator::new();

        // Update with a specific rate for testing
        let rates_json = r#"{"rub": 100.0}"#;
        calc.update_rates_from_api("USD", "2026-01-26", rates_json);

        // Calculate 1 USD in RUB
        let result = calc.calculate_internal("0 RUB + 1 USD");
        assert!(result.success);

        // Verify the steps show the correct API source and date
        let steps_str = result.steps.join("\n");
        assert!(
            steps_str.contains("fawazahmed0/currency-api"),
            "Steps should show API source"
        );
        assert!(
            steps_str.contains("2026-01-26"),
            "Steps should show rate date"
        );
    }

    #[test]
    fn test_api_rate_overrides_hardcoded_rate() {
        let mut calc = Calculator::new();

        // First calculation uses hardcoded rate (89.5)
        let result1 = calc.calculate_internal("0 RUB + 1 USD");
        assert!(result1.success);
        // The hardcoded rate is 89.5, so 1 USD = 89.5 RUB
        assert_eq!(result1.result, "89.5 RUB");

        // Update with API rate (75.5)
        let rates_json = r#"{"rub": 75.5}"#;
        calc.update_rates_from_api("USD", "2026-01-26", rates_json);

        // Second calculation should use the API rate
        let result2 = calc.calculate_internal("0 RUB + 1 USD");
        assert!(result2.success);
        assert_eq!(result2.result, "75.5 RUB");
    }
}
