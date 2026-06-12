use crate::types::{self, Decimal, Expression};
use crate::Calculator;

impl Calculator {
    /// Substitutes a variable with a numeric value in an expression.
    pub(super) fn substitute_variable(
        expr: &types::Expression,
        var: &str,
        value: f64,
    ) -> types::Expression {
        match expr {
            Expression::Variable(name) if name == var => {
                Expression::number(Decimal::from_f64(value))
            }
            Expression::Variable(_)
            | Expression::Number { .. }
            | Expression::DateTime(_)
            | Expression::Now => expr.clone(),
            Expression::Until(inner) => {
                Expression::Until(Box::new(Self::substitute_variable(inner, var, value)))
            }
            Expression::Binary { left, op, right } => Expression::binary(
                Self::substitute_variable(left, var, value),
                *op,
                Self::substitute_variable(right, var, value),
            ),
            Expression::Negate(inner) => {
                Expression::negate(Self::substitute_variable(inner, var, value))
            }
            Expression::Group(inner) => {
                Expression::group(Self::substitute_variable(inner, var, value))
            }
            Expression::Power { base, exponent } => Expression::power(
                Self::substitute_variable(base, var, value),
                Self::substitute_variable(exponent, var, value),
            ),
            Expression::FunctionCall { name, args } => Expression::function_call(
                name.clone(),
                args.iter()
                    .map(|a| Self::substitute_variable(a, var, value))
                    .collect(),
            ),
            Expression::AtTime { value: v, time } => Expression::at_time(
                Self::substitute_variable(v, var, value),
                Self::substitute_variable(time, var, value),
            ),
            Expression::IndefiniteIntegral {
                integrand,
                variable,
            } => Expression::indefinite_integral(
                Self::substitute_variable(integrand, var, value),
                variable.clone(),
            ),
            Expression::UnitConversion {
                value: v,
                target_unit,
            } => Expression::unit_conversion(
                Self::substitute_variable(v, var, value),
                target_unit.clone(),
            ),
            Expression::Equality { left, right } => Expression::equality(
                Self::substitute_variable(left, var, value),
                Self::substitute_variable(right, var, value),
            ),
            Expression::Comparison { left, op, right } => Expression::comparison(
                Self::substitute_variable(left, var, value),
                *op,
                Self::substitute_variable(right, var, value),
            ),
        }
    }
}
