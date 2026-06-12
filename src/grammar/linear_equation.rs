//! Linear-equation helpers for the expression evaluator.

use crate::error::CalculatorError;
use crate::types::{BinaryOp, Expression, Rational, Unit, Value};

#[derive(Debug, Clone)]
struct LinearForm {
    variable: Option<String>,
    coefficient: Rational,
    constant: Rational,
}

#[derive(Debug, Clone)]
pub(super) struct LinearEquationSolution {
    variable: String,
    coefficient: Rational,
    isolated_constant: Rational,
    value: Rational,
    left_form: LinearForm,
    right_form: LinearForm,
}

impl LinearForm {
    fn constant(value: Rational) -> Self {
        Self {
            variable: None,
            coefficient: Rational::zero(),
            constant: value,
        }
    }

    fn variable(name: String) -> Self {
        Self {
            variable: Some(name),
            coefficient: Rational::one(),
            constant: Rational::zero(),
        }
    }

    fn from_expression(expr: &Expression) -> Result<Self, CalculatorError> {
        match expr {
            Expression::Number { value, unit, .. } => {
                if *unit != Unit::None {
                    return Err(Self::unsupported_equation());
                }
                Ok(Self::constant(Rational::from_decimal(*value)))
            }
            Expression::Variable(name) => Ok(Self::variable(name.clone())),
            Expression::Binary { left, op, right } => {
                let left = Self::from_expression(left)?;
                let right = Self::from_expression(right)?;
                match op {
                    BinaryOp::Add => left.add(right),
                    BinaryOp::Subtract => left.subtract(right),
                    BinaryOp::Multiply => left.multiply(right),
                    BinaryOp::Divide => left.divide(right),
                    BinaryOp::Modulo => Err(Self::unsupported_equation()),
                }
            }
            Expression::Negate(inner) => Ok(Self::from_expression(inner)?.negate()),
            Expression::Group(inner) => Self::from_expression(inner),
            Expression::DateTime(_)
            | Expression::Now
            | Expression::Until(_)
            | Expression::AtTime { .. }
            | Expression::FunctionCall { .. }
            | Expression::Power { .. }
            | Expression::IndefiniteIntegral { .. }
            | Expression::UnitConversion { .. }
            | Expression::Equality { .. } => Err(Self::unsupported_equation()),
        }
    }

    fn add(self, other: Self) -> Result<Self, CalculatorError> {
        Ok(Self {
            variable: Self::merge_variable(self.variable, other.variable)?,
            coefficient: self.coefficient + other.coefficient,
            constant: self.constant + other.constant,
        })
    }

    fn subtract(self, other: Self) -> Result<Self, CalculatorError> {
        Ok(Self {
            variable: Self::merge_variable(self.variable, other.variable)?,
            coefficient: self.coefficient - other.coefficient,
            constant: self.constant - other.constant,
        })
    }

    fn multiply(self, other: Self) -> Result<Self, CalculatorError> {
        match (self.variable.is_some(), other.variable.is_some()) {
            (true, true) => Err(CalculatorError::InvalidOperation(
                "equation is not linear".into(),
            )),
            (true, false) => {
                let scale = other.constant;
                Ok(Self {
                    variable: self.variable,
                    coefficient: self.coefficient * scale.clone(),
                    constant: self.constant * scale,
                })
            }
            (false, true) => {
                let scale = self.constant;
                Ok(Self {
                    variable: other.variable,
                    coefficient: other.coefficient * scale.clone(),
                    constant: other.constant * scale,
                })
            }
            (false, false) => Ok(Self::constant(self.constant * other.constant)),
        }
    }

    fn divide(self, other: Self) -> Result<Self, CalculatorError> {
        if other.variable.is_some() {
            return Err(CalculatorError::InvalidOperation(
                "equation has a variable denominator".into(),
            ));
        }
        if other.constant.is_zero() {
            return Err(CalculatorError::DivisionByZero);
        }

        Ok(Self {
            variable: self.variable,
            coefficient: self.coefficient / other.constant.clone(),
            constant: self.constant / other.constant,
        })
    }

    fn negate(self) -> Self {
        Self {
            variable: self.variable,
            coefficient: -self.coefficient,
            constant: -self.constant,
        }
    }

    fn merge_variable(
        left: Option<String>,
        right: Option<String>,
    ) -> Result<Option<String>, CalculatorError> {
        match (left, right) {
            (Some(left), Some(right)) if left == right => Ok(Some(left)),
            (Some(_), Some(_)) => Err(CalculatorError::InvalidOperation(
                "equation has more than one variable".into(),
            )),
            (Some(variable), None) | (None, Some(variable)) => Ok(Some(variable)),
            (None, None) => Ok(None),
        }
    }

    fn unsupported_equation() -> CalculatorError {
        CalculatorError::InvalidOperation(
            "only simple single-variable linear equations with unitless numbers are supported"
                .into(),
        )
    }

    fn format_with_variable(&self, variable: &str) -> String {
        let mut result = String::new();

        if !self.coefficient.is_zero() {
            result.push_str(&Self::format_variable_term(&self.coefficient, variable));
        }

        if self.constant.is_zero() {
            if result.is_empty() {
                return "0".to_string();
            }
            return result;
        }

        if result.is_empty() {
            return self.constant.to_display_string();
        }

        if self.constant.is_negative() {
            result.push_str(" - ");
            result.push_str(&self.constant.abs().to_display_string());
        } else {
            result.push_str(" + ");
            result.push_str(&self.constant.to_display_string());
        }

        result
    }

    fn format_variable_term(coefficient: &Rational, variable: &str) -> String {
        if coefficient == &Rational::one() {
            variable.to_string()
        } else if coefficient == &Rational::from_integer(-1) {
            format!("-{variable}")
        } else {
            format!("{}*{variable}", coefficient.to_display_string())
        }
    }
}

impl LinearEquationSolution {
    pub(super) fn to_value(&self) -> Value {
        Value::equation_solution(self.variable.clone(), self.value.clone())
    }

    pub(super) fn derivation_steps(&self) -> Vec<String> {
        vec![
            format!(
                "Linear form: {} = {}",
                self.left_form.format_with_variable(&self.variable),
                self.right_form.format_with_variable(&self.variable)
            ),
            format!(
                "Isolate variable term: {} = {}",
                LinearForm::format_variable_term(&self.coefficient, &self.variable),
                self.isolated_constant.to_display_string()
            ),
            format!(
                "Solve for {}: {} = {}",
                self.variable,
                self.variable,
                self.value.to_display_string()
            ),
        ]
    }
}

pub(super) fn solve(
    left: &Expression,
    right: &Expression,
) -> Result<LinearEquationSolution, CalculatorError> {
    let left_form = LinearForm::from_expression(left)?;
    let right_form = LinearForm::from_expression(right)?;
    let variable =
        LinearForm::merge_variable(left_form.variable.clone(), right_form.variable.clone())?
            .ok_or_else(LinearForm::unsupported_equation)?;

    let coefficient = left_form.coefficient.clone() - right_form.coefficient.clone();
    if coefficient.is_zero() {
        return Err(CalculatorError::InvalidOperation(
            "linear equation has no unique solution".into(),
        ));
    }

    let isolated_constant = right_form.constant.clone() - left_form.constant.clone();
    let value = isolated_constant.clone() / coefficient.clone();
    Ok(LinearEquationSolution {
        variable,
        coefficient,
        isolated_constant,
        value,
        left_form,
        right_form,
    })
}
