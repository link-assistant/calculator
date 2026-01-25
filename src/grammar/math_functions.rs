//! Mathematical functions implementation.
//!
//! This module provides implementations for various mathematical functions
//! including trigonometry, logarithms, and numerical integration.

use crate::error::CalculatorError;
use crate::types::Decimal;

/// The number of subdivisions for numerical integration (Simpson's rule).
const INTEGRATION_SUBDIVISIONS: usize = 1000;

/// Evaluates a mathematical function with the given arguments.
///
/// # Supported Functions
///
/// ## Trigonometric (arguments in radians)
/// - `sin(x)` - Sine
/// - `cos(x)` - Cosine
/// - `tan(x)` - Tangent
/// - `asin(x)` - Arc sine
/// - `acos(x)` - Arc cosine
/// - `atan(x)` - Arc tangent
/// - `sinh(x)` - Hyperbolic sine
/// - `cosh(x)` - Hyperbolic cosine
/// - `tanh(x)` - Hyperbolic tangent
///
/// ## Exponential and Logarithmic
/// - `exp(x)` - e^x
/// - `ln(x)` - Natural logarithm
/// - `log(x)` - Base-10 logarithm
/// - `log2(x)` - Base-2 logarithm
/// - `pow(base, exp)` - Power function
///
/// ## Other
/// - `sqrt(x)` - Square root
/// - `abs(x)` - Absolute value
/// - `floor(x)` - Floor
/// - `ceil(x)` - Ceiling
/// - `round(x)` - Round to nearest
/// - `factorial(n)` - Factorial (n must be non-negative integer)
///
/// ## Constants
/// - `pi()` - π ≈ 3.14159...
/// - `e()` - Euler's number ≈ 2.71828...
///
pub fn evaluate_function(name: &str, args: &[Decimal]) -> Result<Decimal, CalculatorError> {
    let name_lower = name.to_lowercase();

    match name_lower.as_str() {
        // Constants
        "pi" => {
            check_arg_count(&name_lower, args, 0)?;
            Ok(Decimal::from_f64(std::f64::consts::PI))
        }
        "e" => {
            check_arg_count(&name_lower, args, 0)?;
            Ok(Decimal::from_f64(std::f64::consts::E))
        }

        // Trigonometric functions
        "sin" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.sin()))
        }
        "cos" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.cos()))
        }
        "tan" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            let result = x.tan();
            if result.is_infinite() || result.is_nan() {
                return Err(CalculatorError::domain("tan is undefined at this value"));
            }
            Ok(Decimal::from_f64(result))
        }
        "asin" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if !(-1.0..=1.0).contains(&x) {
                return Err(CalculatorError::domain("asin argument must be in [-1, 1]"));
            }
            Ok(Decimal::from_f64(x.asin()))
        }
        "acos" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if !(-1.0..=1.0).contains(&x) {
                return Err(CalculatorError::domain("acos argument must be in [-1, 1]"));
            }
            Ok(Decimal::from_f64(x.acos()))
        }
        "atan" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.atan()))
        }
        "atan2" => {
            check_arg_count(&name_lower, args, 2)?;
            let y = args[0].to_f64();
            let x = args[1].to_f64();
            Ok(Decimal::from_f64(y.atan2(x)))
        }

        // Hyperbolic functions
        "sinh" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.sinh()))
        }
        "cosh" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.cosh()))
        }
        "tanh" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.tanh()))
        }

        // Exponential and logarithmic
        "exp" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            let result = x.exp();
            if result.is_infinite() {
                return Err(CalculatorError::Overflow);
            }
            Ok(Decimal::from_f64(result))
        }
        "ln" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if x <= 0.0 {
                return Err(CalculatorError::domain("ln argument must be positive"));
            }
            Ok(Decimal::from_f64(x.ln()))
        }
        "log" => {
            // log(x) is log base 10, log(x, base) is log base `base`
            if args.is_empty() || args.len() > 2 {
                return Err(CalculatorError::invalid_args(
                    &name_lower,
                    "expected 1 or 2 arguments",
                ));
            }
            let x = args[0].to_f64();
            if x <= 0.0 {
                return Err(CalculatorError::domain("log argument must be positive"));
            }
            if args.len() == 2 {
                let base = args[1].to_f64();
                #[allow(clippy::float_cmp)]
                if base <= 0.0 || base == 1.0 {
                    return Err(CalculatorError::domain(
                        "log base must be positive and not 1",
                    ));
                }
                Ok(Decimal::from_f64(x.log(base)))
            } else {
                Ok(Decimal::from_f64(x.log10()))
            }
        }
        "log2" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if x <= 0.0 {
                return Err(CalculatorError::domain("log2 argument must be positive"));
            }
            Ok(Decimal::from_f64(x.log2()))
        }
        "log10" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if x <= 0.0 {
                return Err(CalculatorError::domain("log10 argument must be positive"));
            }
            Ok(Decimal::from_f64(x.log10()))
        }
        "pow" => {
            check_arg_count(&name_lower, args, 2)?;
            let base = args[0].to_f64();
            let exp = args[1].to_f64();
            let result = base.powf(exp);
            if result.is_nan() {
                return Err(CalculatorError::domain(
                    "pow result is undefined (e.g., negative base with fractional exponent)",
                ));
            }
            if result.is_infinite() {
                return Err(CalculatorError::Overflow);
            }
            Ok(Decimal::from_f64(result))
        }

        // Other mathematical functions
        "sqrt" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            if x < 0.0 {
                return Err(CalculatorError::domain(
                    "sqrt argument must be non-negative",
                ));
            }
            Ok(Decimal::from_f64(x.sqrt()))
        }
        "cbrt" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.cbrt()))
        }
        "abs" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.abs()))
        }
        "floor" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.floor()))
        }
        "ceil" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.ceil()))
        }
        "round" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.round()))
        }
        "trunc" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.trunc()))
        }
        "sign" | "signum" => {
            check_arg_count(&name_lower, args, 1)?;
            let x = args[0].to_f64();
            Ok(Decimal::from_f64(x.signum()))
        }
        "min" => {
            if args.len() < 2 {
                return Err(CalculatorError::invalid_args(
                    &name_lower,
                    "expected at least 2 arguments",
                ));
            }
            let min = args
                .iter()
                .map(Decimal::to_f64)
                .fold(f64::INFINITY, f64::min);
            Ok(Decimal::from_f64(min))
        }
        "max" => {
            if args.len() < 2 {
                return Err(CalculatorError::invalid_args(
                    &name_lower,
                    "expected at least 2 arguments",
                ));
            }
            let max = args
                .iter()
                .map(Decimal::to_f64)
                .fold(f64::NEG_INFINITY, f64::max);
            Ok(Decimal::from_f64(max))
        }
        "factorial" => {
            check_arg_count(&name_lower, args, 1)?;
            let n = args[0].to_f64();
            #[allow(clippy::float_cmp)]
            if n < 0.0 || n != n.floor() {
                return Err(CalculatorError::domain(
                    "factorial argument must be a non-negative integer",
                ));
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n_int = n as u64;
            if n_int > 170 {
                return Err(CalculatorError::Overflow);
            }
            let result = factorial(n_int);
            Ok(Decimal::from_f64(result))
        }

        // Conversion functions
        "deg" | "degrees" => {
            check_arg_count(&name_lower, args, 1)?;
            let radians = args[0].to_f64();
            Ok(Decimal::from_f64(radians.to_degrees()))
        }
        "rad" | "radians" => {
            check_arg_count(&name_lower, args, 1)?;
            let degrees = args[0].to_f64();
            Ok(Decimal::from_f64(degrees.to_radians()))
        }

        _ => Err(CalculatorError::unknown_function(name)),
    }
}

/// Returns true if the given name is a known math function.
#[must_use]
pub fn is_math_function(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    matches!(
        name_lower.as_str(),
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "atan2"
            | "sinh"
            | "cosh"
            | "tanh"
            | "exp"
            | "ln"
            | "log"
            | "log2"
            | "log10"
            | "pow"
            | "sqrt"
            | "cbrt"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "trunc"
            | "sign"
            | "signum"
            | "min"
            | "max"
            | "integrate"
            | "factorial"
            | "pi"
            | "e"
            | "deg"
            | "degrees"
            | "rad"
            | "radians"
    )
}

/// Checks that the function received the expected number of arguments.
fn check_arg_count(
    func_name: &str,
    args: &[Decimal],
    expected: usize,
) -> Result<(), CalculatorError> {
    if args.len() != expected {
        return Err(CalculatorError::invalid_args(
            func_name,
            format!("expected {} argument(s), got {}", expected, args.len()),
        ));
    }
    Ok(())
}

/// Computes factorial of n.
fn factorial(n: u64) -> f64 {
    if n <= 1 {
        1.0
    } else {
        (2..=n).fold(1.0, |acc, x| acc * (x as f64))
    }
}

/// Performs numerical integration using Simpson's rule.
///
/// Integrates the function `f` from `a` to `b` using Simpson's rule
/// with `INTEGRATION_SUBDIVISIONS` intervals.
#[allow(clippy::many_single_char_names)]
pub fn integrate<F>(f: F, a: f64, b: f64) -> f64
where
    F: Fn(f64) -> f64,
{
    let n = INTEGRATION_SUBDIVISIONS;
    let h = (b - a) / (n as f64);

    let mut sum = f(a) + f(b);

    for i in 1..n {
        let x = (i as f64).mul_add(h, a);
        if i % 2 == 0 {
            sum += 2.0 * f(x);
        } else {
            sum += 4.0 * f(x);
        }
    }

    sum * h / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_sin() {
        let result = evaluate_function("sin", &[Decimal::new(0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 0.0, 1e-10));

        let result =
            evaluate_function("sin", &[Decimal::from_f64(std::f64::consts::PI / 2.0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));
    }

    #[test]
    fn test_cos() {
        let result = evaluate_function("cos", &[Decimal::new(0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));

        let result = evaluate_function("cos", &[Decimal::from_f64(std::f64::consts::PI)]).unwrap();
        assert!(approx_eq(result.to_f64(), -1.0, 1e-10));
    }

    #[test]
    fn test_sqrt() {
        let result = evaluate_function("sqrt", &[Decimal::new(16)]).unwrap();
        assert!(approx_eq(result.to_f64(), 4.0, 1e-10));

        let result = evaluate_function("sqrt", &[Decimal::new(2)]).unwrap();
        assert!(approx_eq(result.to_f64(), std::f64::consts::SQRT_2, 1e-10));
    }

    #[test]
    fn test_sqrt_negative() {
        let result = evaluate_function("sqrt", &[Decimal::from_f64(-1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ln() {
        let result = evaluate_function("ln", &[Decimal::from_f64(std::f64::consts::E)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));
    }

    #[test]
    fn test_exp() {
        let result = evaluate_function("exp", &[Decimal::new(0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));

        let result = evaluate_function("exp", &[Decimal::new(1)]).unwrap();
        assert!(approx_eq(result.to_f64(), std::f64::consts::E, 1e-10));
    }

    #[test]
    fn test_pow() {
        let result = evaluate_function("pow", &[Decimal::new(2), Decimal::new(3)]).unwrap();
        assert!(approx_eq(result.to_f64(), 8.0, 1e-10));
    }

    #[test]
    fn test_factorial() {
        let result = evaluate_function("factorial", &[Decimal::new(5)]).unwrap();
        assert!(approx_eq(result.to_f64(), 120.0, 1e-10));

        let result = evaluate_function("factorial", &[Decimal::new(0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));
    }

    #[test]
    fn test_pi() {
        let result = evaluate_function("pi", &[]).unwrap();
        assert!(approx_eq(result.to_f64(), std::f64::consts::PI, 1e-10));
    }

    #[test]
    fn test_e() {
        let result = evaluate_function("e", &[]).unwrap();
        assert!(approx_eq(result.to_f64(), std::f64::consts::E, 1e-10));
    }

    #[test]
    fn test_abs() {
        let result = evaluate_function("abs", &[Decimal::from_f64(-5.0)]).unwrap();
        assert!(approx_eq(result.to_f64(), 5.0, 1e-10));
    }

    #[test]
    fn test_min_max() {
        let result =
            evaluate_function("min", &[Decimal::new(3), Decimal::new(1), Decimal::new(2)]).unwrap();
        assert!(approx_eq(result.to_f64(), 1.0, 1e-10));

        let result =
            evaluate_function("max", &[Decimal::new(3), Decimal::new(1), Decimal::new(2)]).unwrap();
        assert!(approx_eq(result.to_f64(), 3.0, 1e-10));
    }

    #[test]
    fn test_integrate_constant() {
        // Integral of 1 from 0 to 1 should be 1
        let result = integrate(|_| 1.0, 0.0, 1.0);
        assert!(approx_eq(result, 1.0, 1e-6));
    }

    #[test]
    fn test_integrate_linear() {
        // Integral of x from 0 to 1 should be 0.5
        let result = integrate(|x| x, 0.0, 1.0);
        assert!(approx_eq(result, 0.5, 1e-6));
    }

    #[test]
    fn test_integrate_quadratic() {
        // Integral of x^2 from 0 to 1 should be 1/3
        let result = integrate(|x| x * x, 0.0, 1.0);
        assert!(approx_eq(result, 1.0 / 3.0, 1e-6));
    }

    #[test]
    fn test_integrate_sin() {
        // Integral of sin(x) from 0 to pi should be 2
        let result = integrate(f64::sin, 0.0, std::f64::consts::PI);
        assert!(approx_eq(result, 2.0, 1e-6));
    }

    #[test]
    fn test_is_math_function() {
        assert!(is_math_function("sin"));
        assert!(is_math_function("SIN"));
        assert!(is_math_function("sqrt"));
        assert!(is_math_function("pi"));
        assert!(!is_math_function("foo"));
        assert!(!is_math_function("USD"));
    }
}
