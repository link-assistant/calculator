//! Symbolic integral evaluation module.
//!
//! This module handles symbolic integration for indefinite integrals,
//! computing results for known special cases like Si(x), Ci(x), etc.

use crate::error::CalculatorError;
use crate::types::{BinaryOp, Expression, Value};

/// Evaluates an indefinite integral.
///
/// For known special integrals, returns symbolic result.
/// For others, returns an informational message.
pub fn evaluate_indefinite_integral(
    integrand: &Expression,
    variable: &str,
) -> Result<Value, CalculatorError> {
    // Check for known special integrals
    let symbolic_result = try_symbolic_integral(integrand, variable);

    if let Some(result) = symbolic_result {
        // Return a special value that indicates symbolic result
        // For now, we'll create an error with the symbolic result as a message
        // since the Value type doesn't support symbolic results yet
        let latex_result = symbolic_result_to_latex(&result);
        Err(CalculatorError::SymbolicResult {
            expression: format!("∫ {} d{}", integrand, variable),
            result,
            latex_input: format!("\\int {} \\, d{}", integrand.to_latex(), variable),
            latex_result,
        })
    } else {
        // For unknown integrals, provide a helpful message
        Err(CalculatorError::SymbolicResult {
            expression: format!("∫ {} d{}", integrand, variable),
            result: "Cannot compute symbolic result. Use definite integral with bounds: integrate(expr, var, lower, upper)".to_string(),
            latex_input: format!("\\int {} \\, d{}", integrand.to_latex(), variable),
            latex_result: "\\text{Use definite integral with bounds}".to_string(),
        })
    }
}

/// Tries to compute a symbolic integral for known special cases.
pub fn try_symbolic_integral(integrand: &Expression, variable: &str) -> Option<String> {
    // Pattern: sin(x)/x -> Si(x) + C (Sine Integral)
    if let Expression::Binary {
        left,
        op: BinaryOp::Divide,
        right,
    } = integrand
    {
        if let Expression::FunctionCall { name, args } = left.as_ref() {
            if name.to_lowercase() == "sin" && args.len() == 1 {
                if let Expression::Variable(v) = &args[0] {
                    if let Expression::Variable(v2) = right.as_ref() {
                        if v == variable && v2 == variable {
                            return Some(format!("Si({}) + C", variable));
                        }
                    }
                }
            }
        }
    }

    // Pattern: cos(x)/x -> Ci(x) + C (Cosine Integral)
    if let Expression::Binary {
        left,
        op: BinaryOp::Divide,
        right,
    } = integrand
    {
        if let Expression::FunctionCall { name, args } = left.as_ref() {
            if name.to_lowercase() == "cos" && args.len() == 1 {
                if let Expression::Variable(v) = &args[0] {
                    if let Expression::Variable(v2) = right.as_ref() {
                        if v == variable && v2 == variable {
                            return Some(format!("Ci({}) + C", variable));
                        }
                    }
                }
            }
        }
    }

    // Pattern: x^n -> x^(n+1)/(n+1) + C
    if let Expression::Power { base, exponent } = integrand {
        if let Expression::Variable(v) = base.as_ref() {
            if v == variable {
                if let Expression::Number { value, .. } = exponent.as_ref() {
                    let n = value.to_f64();
                    if (n - (-1.0)).abs() > 1e-10 {
                        // Not x^(-1)
                        let new_exp = n + 1.0;
                        return Some(format!("{}^{}/({}) + C", variable, new_exp, new_exp));
                    }
                    // x^(-1) = 1/x -> ln|x| + C
                    return Some(format!("ln|{}| + C", variable));
                }
            }
        }
    }

    // Pattern: just x -> x^2/2 + C
    if let Expression::Variable(v) = integrand {
        if v == variable {
            return Some(format!("{}²/2 + C", variable));
        }
    }

    // Pattern: constant -> constant * x + C
    if let Expression::Number { value, .. } = integrand {
        return Some(format!("{} * {} + C", value, variable));
    }

    // Pattern: sin(x) -> -cos(x) + C
    if let Expression::FunctionCall { name, args } = integrand {
        if args.len() == 1 {
            if let Expression::Variable(v) = &args[0] {
                if v == variable {
                    match name.to_lowercase().as_str() {
                        "sin" => return Some(format!("-cos({}) + C", variable)),
                        "cos" => return Some(format!("sin({}) + C", variable)),
                        "exp" => return Some(format!("exp({}) + C", variable)),
                        _ => {}
                    }
                }
            }
        }
    }

    None
}

/// Converts a symbolic result to LaTeX.
pub fn symbolic_result_to_latex(result: &str) -> String {
    // Basic conversions
    result
        .replace("Si(", "\\text{Si}(")
        .replace("Ci(", "\\text{Ci}(")
        .replace("ln|", "\\ln|")
        .replace("²", "^{2}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Decimal;

    #[test]
    fn test_sin_x_over_x() {
        // sin(x)/x -> Si(x) + C
        let integrand = Expression::binary(
            Expression::function_call("sin", vec![Expression::variable("x")]),
            BinaryOp::Divide,
            Expression::variable("x"),
        );
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("Si(x) + C".to_string()));
    }

    #[test]
    fn test_cos_x_over_x() {
        // cos(x)/x -> Ci(x) + C
        let integrand = Expression::binary(
            Expression::function_call("cos", vec![Expression::variable("x")]),
            BinaryOp::Divide,
            Expression::variable("x"),
        );
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("Ci(x) + C".to_string()));
    }

    #[test]
    fn test_x_squared() {
        // x^2 -> x^3/3 + C
        let integrand = Expression::power(
            Expression::variable("x"),
            Expression::number(Decimal::new(2)),
        );
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("x^3/(3) + C".to_string()));
    }

    #[test]
    fn test_just_x() {
        // x -> x²/2 + C
        let integrand = Expression::variable("x");
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("x²/2 + C".to_string()));
    }

    #[test]
    fn test_constant() {
        // 5 -> 5 * x + C
        let integrand = Expression::number(Decimal::new(5));
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("5 * x + C".to_string()));
    }

    #[test]
    fn test_sin_x() {
        // sin(x) -> -cos(x) + C
        let integrand = Expression::function_call("sin", vec![Expression::variable("x")]);
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("-cos(x) + C".to_string()));
    }

    #[test]
    fn test_cos_x() {
        // cos(x) -> sin(x) + C
        let integrand = Expression::function_call("cos", vec![Expression::variable("x")]);
        let result = try_symbolic_integral(&integrand, "x");
        assert_eq!(result, Some("sin(x) + C".to_string()));
    }

    #[test]
    fn test_symbolic_result_to_latex() {
        assert_eq!(symbolic_result_to_latex("Si(x) + C"), "\\text{Si}(x) + C");
        assert_eq!(symbolic_result_to_latex("ln|x| + C"), "\\ln|x| + C");
        assert_eq!(symbolic_result_to_latex("x²/2 + C"), "x^{2}/2 + C");
    }
}
