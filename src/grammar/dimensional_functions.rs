//! Powers, roots and functions of dimensional values (issue #235).
//!
//! `(2 m)^2` is `4 m^2`, `sqrt(4 m^2)` is `2 m` and `sqrt(2 m)` fails with a
//! non-integer dimension. Rounding keeps the unit, trigonometric functions
//! accept angles (`sin(30°)`), and other functions need plain numbers.

use crate::error::CalculatorError;
use crate::types::{normalize_affine, Dimension, Expression, Rational, Unit, Value};

use super::math_functions::evaluate_function_values;

/// Reads a temperature result in kelvin unless it was converted explicitly:
/// `20 °C` is `293.15 K` as in Numbat, while `25 °C in °F` stays `77 °F`.
/// Inside expressions `°C` keeps its offset, so `25 °C * 2` is rejected.
pub fn kelvin_result(expr: &Expression, value: Value) -> Value {
    let mut inner = expr;
    while let Expression::Group(group) = inner {
        inner = group;
    }
    if matches!(inner, Expression::UnitConversion { .. }) {
        return value;
    }
    let Some(amount) = value.to_rational() else {
        return value;
    };
    let (amount, unit) = normalize_affine(amount, &value.unit);
    if unit == value.unit {
        return value;
    }
    Value::rational_with_unit(amount, unit)
}

/// Raises a dimensional value to a rational power; `None` when the base has
/// no dimensional unit.
pub fn dimensional_power(base: &Value, exponent: &Value) -> Option<Result<Value, CalculatorError>> {
    base.physical_base()?;
    if exponent.unit != Unit::None {
        return Some(Err(CalculatorError::domain(
            "an exponent must be a plain number",
        )));
    }
    let Some(exponent) = exponent.to_rational() else {
        return Some(Err(CalculatorError::domain(
            "an exponent must be a plain number",
        )));
    };
    let power = i32::try_from(exponent.numer())
        .ok()
        .filter(|power| power.abs() <= 1000);
    let root = i32::try_from(exponent.denom())
        .ok()
        .filter(|root| *root <= 1000);
    let (Some(power), Some(root)) = (power, root) else {
        return Some(Err(CalculatorError::domain(format!(
            "power {exponent} of a quantity with units is not supported"
        ))));
    };
    if root == 1 {
        return base.physical_power(power);
    }
    Some(
        base.physical_root(root)?
            .and_then(|rooted| rooted.physical_power(power).unwrap_or(Ok(rooted))),
    )
}

/// Evaluates a one-argument function of a dimensional value; `None` when the
/// argument is a plain number or the function has no dimensional meaning.
pub fn dimensional_function(name: &str, args: &[Value]) -> Option<Result<Value, CalculatorError>> {
    let [arg] = args else {
        return None;
    };
    let (base, dimension) = arg.physical_base()?;
    let lower = name.to_lowercase();
    match lower.as_str() {
        "sqrt" => arg.physical_root(2),
        "cbrt" => arg.physical_root(3),
        "abs" | "floor" | "ceil" | "round" | "trunc" => {
            let amount = arg.to_rational()?;
            Some(
                evaluate_function_values(name, &[Value::rational(amount)]).and_then(|value| {
                    let amount = value.to_rational().ok_or_else(|| {
                        CalculatorError::invalid_args(name, "expected numeric result")
                    })?;
                    Ok(Value::rational_with_unit(amount, arg.unit.clone()))
                }),
            )
        }
        "sin" | "cos" | "tan" | "sec" | "csc" | "cot"
            if Dimension::base("angle") == Some(dimension) =>
        {
            Some(trigonometric(name, &lower, &base, arg))
        }
        _ if matches!(arg.unit, Unit::Physical(_)) => Some(Err(CalculatorError::invalid_args(
            name,
            format!("expected a plain number, got {}", arg.to_display_string()),
        ))),
        _ => None,
    }
}

/// Evaluates a trigonometric function of an angle in radians, giving exact
/// values at multiples of a right angle (`sin(180°)` is `0`).
fn trigonometric(
    name: &str,
    lower: &str,
    radians: &Rational,
    arg: &Value,
) -> Result<Value, CalculatorError> {
    let value = evaluate_function_values(name, &[Value::rational(radians.clone())])?;
    let quarter_turns = radians.to_f64() / std::f64::consts::FRAC_PI_2;
    let on_axis = (quarter_turns - quarter_turns.round()).abs() < 1e-9;
    if !on_axis {
        return Ok(value);
    }
    #[allow(clippy::cast_possible_truncation)]
    let quadrant = (quarter_turns.round() as i64).rem_euclid(4);
    let (sin, cos) = [(0, 1), (1, 0), (0, -1), (-1, 0)][usize::try_from(quadrant).unwrap_or(0)];
    let exact = match lower {
        "sin" => Some((sin, 1)),
        "cos" => Some((cos, 1)),
        "tan" => Some((sin, cos)),
        "cot" => Some((cos, sin)),
        "sec" => Some((1, cos)),
        "csc" => Some((1, sin)),
        _ => None,
    };
    match exact {
        Some((_, 0)) => Err(CalculatorError::domain(format!(
            "{name} is undefined at {}",
            arg.to_display_string()
        ))),
        Some((numer, denom)) => Ok(Value::rational(Rational::new(numer, denom))),
        None => Ok(value),
    }
}
