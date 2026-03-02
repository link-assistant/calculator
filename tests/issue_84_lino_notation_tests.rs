//! Tests for issue #84: Proper links notation for function expressions
//! and alternative interpretation support.

use link_calculator::Calculator;

mod lino_notation_tests {
    use super::*;

    #[test]
    fn test_integrate_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("integrate(x^2, x, 0, 3)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(integrate ((x ^ 2) x 0 3))");
    }

    #[test]
    fn test_sin_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("sin(0)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(sin (0))");
    }

    #[test]
    fn test_cos_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("cos(0)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(cos (0))");
    }

    #[test]
    fn test_sqrt_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("sqrt(16)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(sqrt (16))");
    }

    #[test]
    fn test_pow_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("pow(2, 3)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(pow (2 3))");
    }

    #[test]
    fn test_abs_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("abs(-5)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(abs ((-5)))");
    }

    #[test]
    fn test_nested_functions_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("sqrt(abs(-16))");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(sqrt ((abs ((-16)))))");
    }

    #[test]
    fn test_function_with_expression_arg_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("sqrt(9 + 7)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(sqrt ((9 + 7)))");
    }

    #[test]
    fn test_min_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("min(5, 3)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(min (5 3))");
    }

    #[test]
    fn test_max_function_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("max(5, 3)");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(max (5 3))");
    }

    #[test]
    fn test_power_operator_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2^3");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(2 ^ 3)");
    }

    #[test]
    fn test_pi_constant_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("pi()");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(pi)");
    }

    #[test]
    fn test_e_constant_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("e()");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(e)");
    }

    #[test]
    fn test_binary_expression_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2 + 3");
        assert!(result.success);
        assert_eq!(result.lino_interpretation, "(2 + 3)");
    }

    #[test]
    fn test_indefinite_integral_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("integrate x^2 dx");
        assert!(result.success);
        assert_eq!(
            result.lino_interpretation,
            "(integrate ((x ^ 2) * (differential of (x))))"
        );
    }

    #[test]
    fn test_indefinite_integral_sin_over_x_lino() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("integrate sin(x)/x dx");
        assert!(result.success);
        assert_eq!(
            result.lino_interpretation,
            "(integrate (((sin (x)) / x) * (differential of (x))))"
        );
    }

    #[test]
    fn test_indefinite_integral_cos_x_lino() {
        // Issue #89: integrate cos(x) dx should produce explicit lino notation
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("integrate cos(x) dx");
        assert!(result.success);
        assert_eq!(
            result.lino_interpretation,
            "(integrate ((cos (x)) * (differential of (x))))"
        );
    }

    #[test]
    fn test_all_expressions_have_outer_parens() {
        let mut calc = Calculator::new();

        let expressions = vec![
            "sin(0)",
            "2 + 3",
            "2^3",
            "sqrt(16)",
            "integrate(x^2, x, 0, 3)",
            "pi()",
            "abs(-5)",
            "integrate x^2 dx",
            "integrate cos(x) dx",
        ];

        for expr in expressions {
            let result = calc.calculate_internal(expr);
            assert!(result.success, "Expression '{expr}' failed to evaluate");
            let lino = &result.lino_interpretation;
            assert!(
                lino.starts_with('(') && lino.ends_with(')'),
                "Expression '{expr}' lino '{lino}' should be wrapped in outer ()"
            );
        }
    }
}

mod alternative_interpretation_tests {
    use super::*;

    #[test]
    fn test_simple_expression_no_alternatives() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2 + 3");
        assert!(result.success);
        // Simple addition has no alternative interpretations
        assert!(result.alternative_lino.is_none());
    }

    #[test]
    fn test_mixed_precedence_has_alternatives() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2 + 3 * 4");
        assert!(result.success);
        assert_eq!(result.result, "14");

        // Should have alternative interpretations
        let alts = result.alternative_lino.expect("should have alternatives");
        assert!(alts.len() >= 2, "should have at least 2 interpretations");
        // First should be the default (standard precedence)
        assert_eq!(alts[0], "(2 + (3 * 4))");
        // Second should be the alternative grouping
        assert_eq!(alts[1], "((2 + 3) * 4)");
    }

    #[test]
    fn test_function_call_has_alternatives() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("sin(0)");
        assert!(result.success);

        let alts = result.alternative_lino.expect("should have alternatives");
        assert!(alts.len() >= 2);
        // First is default links notation
        assert_eq!(alts[0], "(sin (0))");
        // Second is the mathematical expression notation
        assert!(alts[1].contains("expression"));
    }

    #[test]
    fn test_integrate_function_has_alternatives() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("integrate(x^2, x, 0, 3)");
        assert!(result.success);

        let alts = result.alternative_lino.expect("should have alternatives");
        assert!(alts.len() >= 2);
        // First is default links notation
        assert_eq!(alts[0], "(integrate ((x ^ 2) x 0 3))");
    }

    #[test]
    fn test_power_expression_no_alternatives() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2^3");
        assert!(result.success);
        // Simple power has no alternative interpretations
        assert!(result.alternative_lino.is_none());
    }

    #[test]
    fn test_reverse_mixed_precedence() {
        let mut calc = Calculator::new();
        let result = calc.calculate_internal("2 * 3 + 4");
        assert!(result.success);
        assert_eq!(result.result, "10");

        let alts = result.alternative_lino.expect("should have alternatives");
        assert!(alts.len() >= 2);
        assert_eq!(alts[0], "((2 * 3) + 4)");
        assert_eq!(alts[1], "(2 * (3 + 4))");
    }
}
