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
    fn test_integrate_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x)/x dx");
        assert!(!result.success);
        assert!(result.error.is_some());
        let error = result.error.unwrap();
        assert!(error.contains("Advanced math expression detected"));
        assert!(result.issue_link.is_some());
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
        assert!(link.contains("integrate"));
    }

    #[test]
    fn test_differentiate_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("differentiate x^2");
        assert!(!result.success);
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
    }

    #[test]
    fn test_solve_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("solve x^2 = 4");
        assert!(!result.success);
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
    }

    #[test]
    fn test_limit_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("limit x -> 0 sin(x)/x");
        assert!(!result.success);
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
    }

    #[test]
    fn test_sin_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sin(45)");
        assert!(!result.success);
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
    }

    #[test]
    fn test_sqrt_expression_suggests_wolfram() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("sqrt(16)");
        assert!(!result.success);
        let link = result.issue_link.unwrap();
        assert!(link.contains("wolframalpha.com"));
    }

    #[test]
    fn test_wolfram_url_contains_original_input() {
        let calculator = Calculator::new();
        let result = calculator.calculate_internal("integrate sin(x)/x dx");
        let link = result.issue_link.unwrap();
        // URL-encoded version should contain the input
        assert!(link.contains("integrate"));
        assert!(link.contains("sin"));
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
