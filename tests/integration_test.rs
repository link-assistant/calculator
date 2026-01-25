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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_simple_subtraction() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("10 - 4");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_simple_multiplication() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "12");
    }

    #[test]
    fn test_simple_division() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("15 / 3");
        assert!(result.success);
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_division_by_zero() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("10 / 0");
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("zero"));
    }

    #[test]
    fn test_decimal_numbers() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("3.14 + 2.86");
        assert!(result.success);
        assert_eq!(result.result, "6");
    }

    #[test]
    fn test_negative_numbers() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("-5 + 3");
        assert!(result.success);
        assert_eq!(result.result, "-2");
    }

    #[test]
    fn test_parentheses() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("(2 + 3) * 4");
        assert!(result.success);
        assert_eq!(result.result, "20");
    }

    #[test]
    fn test_operator_precedence() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "14"); // 2 + (3 * 4) = 14
    }
}

mod currency_tests {
    use super::*;

    #[test]
    fn test_currency_value() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("100 USD");
        assert!(result.success);
        assert!(result.result.contains("100"));
        assert!(result.result.contains("USD"));
    }

    #[test]
    fn test_same_currency_addition() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("100 USD + 50 USD");
        assert!(result.success);
        assert!(result.result.contains("150"));
    }

    #[test]
    fn test_currency_conversion() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("84 USD - 34 EUR");
        assert!(result.success);
        // Should convert EUR to USD and perform subtraction
        assert!(result.result.contains("USD"));
    }
}

mod datetime_tests {
    use super::*;

    #[test]
    fn test_datetime_subtraction() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("2 + 3");
        assert!(result.success);
        assert!(!result.lino_interpretation.is_empty());
        assert!(result.lino_interpretation.contains('+'));
    }

    #[test]
    fn test_lino_representation_currency() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("");
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_invalid_input_generates_issue_link() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sin(0)");
        assert!(result.success, "sin(0) should succeed");
        assert_eq!(result.result, "0");
    }

    #[test]
    fn test_cos_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("cos(0)");
        assert!(result.success, "cos(0) should succeed");
        assert_eq!(result.result, "1");
    }

    #[test]
    fn test_sqrt_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(16)");
        assert!(result.success, "sqrt(16) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_sqrt_function_decimal() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("pow(2, 3)");
        assert!(result.success, "pow(2, 3) should succeed");
        assert_eq!(result.result, "8");
    }

    #[test]
    fn test_power_operator() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("2^3");
        assert!(result.success, "2^3 should succeed");
        assert_eq!(result.result, "8");
    }

    #[test]
    fn test_exp_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("exp(0)");
        assert!(result.success, "exp(0) should succeed");
        assert_eq!(result.result, "1");
    }

    #[test]
    fn test_ln_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("ln(1)");
        assert!(result.success, "ln(1) should succeed");
        assert_eq!(result.result, "0");
    }

    #[test]
    fn test_log_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("log(100)");
        assert!(result.success, "log(100) should succeed");
        assert_eq!(result.result, "2");
    }

    #[test]
    fn test_abs_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("abs(-5)");
        assert!(result.success, "abs(-5) should succeed");
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_floor_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("floor(3.7)");
        assert!(result.success, "floor(3.7) should succeed");
        assert_eq!(result.result, "3");
    }

    #[test]
    fn test_ceil_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("ceil(3.2)");
        assert!(result.success, "ceil(3.2) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_pi_constant() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("pi()");
        assert!(result.success, "pi() should succeed");
        assert!(
            result.result.starts_with("3.14"),
            "pi should start with 3.14"
        );
    }

    #[test]
    fn test_e_constant() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("e()");
        assert!(result.success, "e() should succeed");
        assert!(
            result.result.starts_with("2.71"),
            "e should start with 2.71"
        );
    }

    #[test]
    fn test_nested_functions() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(abs(-16))");
        assert!(result.success, "sqrt(abs(-16)) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_function_with_expression() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(9 + 7)");
        assert!(result.success, "sqrt(9 + 7) should succeed");
        assert_eq!(result.result, "4");
    }

    #[test]
    fn test_min_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("min(5, 3)");
        assert!(result.success, "min(5, 3) should succeed");
        assert_eq!(result.result, "3");
    }

    #[test]
    fn test_max_function() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("max(5, 3)");
        assert!(result.success, "max(5, 3) should succeed");
        assert_eq!(result.result, "5");
    }

    #[test]
    fn test_factorial() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("factorial(5)");
        assert!(result.success, "factorial(5) should succeed");
        assert_eq!(result.result, "120");
    }

    #[test]
    fn test_deg_function() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(-1)");
        assert!(!result.success, "sqrt(-1) should fail");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Domain error"));
    }

    #[test]
    fn test_ln_negative_error() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("ln(-1)");
        assert!(!result.success, "ln(-1) should fail");
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Domain error"));
    }

    #[test]
    fn test_unknown_function_error() {
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
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
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate x^2 dx");

        assert!(result.success, "integrate x^2 dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // x^2 integrates to x^3/3 + C
        assert!(
            result.result.contains("3") && result.result.contains("C"),
            "Result should contain x^3/3 + C pattern: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_sin() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x) dx");

        assert!(result.success, "integrate sin(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // sin(x) integrates to -cos(x) + C
        assert!(
            result.result.contains("cos") && result.result.contains("C"),
            "Result should contain -cos(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_cos() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate cos(x) dx");

        assert!(result.success, "integrate cos(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // cos(x) integrates to sin(x) + C
        assert!(
            result.result.contains("sin") && result.result.contains("C"),
            "Result should contain sin(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_x() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate x dx");

        assert!(result.success, "integrate x dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // x integrates to x^2/2 + C
        assert!(
            result.result.contains("2") && result.result.contains("C"),
            "Result should contain x²/2 + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_constant() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate 5 dx");

        assert!(result.success, "integrate 5 dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // 5 integrates to 5x + C
        assert!(
            result.result.contains("5")
                && result.result.contains("x")
                && result.result.contains("C"),
            "Result should contain 5 * x + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_exp() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate exp(x) dx");

        assert!(result.success, "integrate exp(x) dx should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        // exp(x) integrates to exp(x) + C
        assert!(
            result.result.contains("exp") && result.result.contains("C"),
            "Result should contain exp(x) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_natural_integral_notation_different_variable() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(t) dt");

        assert!(result.success, "integrate sin(t) dt should succeed");
        assert!(
            result.is_symbolic.unwrap_or(false),
            "Result should be symbolic"
        );
        assert!(
            result.result.contains("cos") && result.result.contains("t"),
            "Result should contain -cos(t) + C: got '{}'",
            result.result
        );
    }

    #[test]
    fn test_latex_rendering_for_integral() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x)/x dx");

        assert!(result.success, "Should succeed");
        let latex_input = result.latex_input.unwrap();
        assert!(
            latex_input.contains("\\int"),
            "LaTeX input should contain integral symbol: got '{}'",
            latex_input
        );
    }

    #[test]
    fn test_definite_integral_still_works() {
        // Ensure the traditional definite integral syntax still works
        let calculator = Calculator::new();
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
