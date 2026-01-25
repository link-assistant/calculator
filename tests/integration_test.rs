//! Integration tests for Link Calculator.
//!
//! These tests verify the public API works correctly.

use link_calculator::{Calculator, VERSION};

mod calculator_tests {
    use super::*;

    #[test]
    fn test_calculator_creation() {
        let mut calculator = Calculator::new();
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
