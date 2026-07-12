//! Linear-equation helpers for the expression evaluator.

use crate::error::CalculatorError;
use crate::types::{BinaryOp, Expression, Rational, Unit, Value};

#[derive(Debug, Clone)]
struct LinearTerm {
    variable: String,
    coefficient: Rational,
}

#[derive(Debug, Clone)]
struct LinearForm {
    terms: Vec<LinearTerm>,
    constant: Rational,
}

#[derive(Debug, Clone)]
struct VariableMoveDetail {
    variable: String,
    left_coefficient: Rational,
    right_coefficient: Rational,
    moved_coefficient: Rational,
}

#[derive(Debug, Clone)]
pub(super) struct LinearEquationSolution {
    variable: String,
    coefficient: Rational,
    isolated_expression: LinearForm,
    value_expression: LinearForm,
    left_form: LinearForm,
    right_form: LinearForm,
    variable_move_details: Vec<VariableMoveDetail>,
}

impl LinearForm {
    fn constant(value: Rational) -> Self {
        Self {
            terms: Vec::new(),
            constant: value,
        }
    }

    fn variable(name: String) -> Self {
        Self {
            terms: vec![LinearTerm {
                variable: name,
                coefficient: Rational::one(),
            }],
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
                    BinaryOp::Add => Ok(left.add(right)),
                    BinaryOp::Subtract => Ok(left.subtract(right)),
                    BinaryOp::Multiply => left.multiply(right),
                    BinaryOp::Divide => left.divide(&right),
                    BinaryOp::Modulo => Err(Self::unsupported_equation()),
                }
            }
            Expression::Negate(inner) => Ok(Self::from_expression(inner)?.negate()),
            Expression::Group(inner) => Self::from_expression(inner),
            Expression::DateTime(_)
            | Expression::Now
            | Expression::Today
            | Expression::Until(_)
            | Expression::AtTime { .. }
            | Expression::FunctionCall { .. }
            | Expression::Power { .. }
            | Expression::IndefiniteIntegral { .. }
            | Expression::UnitConversion { .. }
            | Expression::Equality { .. }
            | Expression::Comparison { .. } => Err(Self::unsupported_equation()),
        }
    }

    fn add(mut self, other: Self) -> Self {
        for term in other.terms {
            self.add_term(term.variable, term.coefficient);
        }
        self.constant = self.constant + other.constant;
        self
    }

    fn subtract(mut self, other: Self) -> Self {
        for term in other.terms {
            self.add_term(term.variable, -term.coefficient);
        }
        self.constant = self.constant - other.constant;
        self
    }

    fn multiply(self, other: Self) -> Result<Self, CalculatorError> {
        match (self.has_variable_terms(), other.has_variable_terms()) {
            (true, true) => Err(CalculatorError::InvalidOperation(
                "equation is not linear".into(),
            )),
            (true, false) => Ok(self.scale(&other.constant)),
            (false, true) => Ok(other.scale(&self.constant)),
            (false, false) => Ok(Self::constant(self.constant * other.constant)),
        }
    }

    fn divide(self, other: &Self) -> Result<Self, CalculatorError> {
        if other.has_variable_terms() {
            return Err(CalculatorError::InvalidOperation(
                "equation has a variable denominator".into(),
            ));
        }
        self.divide_by_rational(&other.constant)
    }

    fn negate(self) -> Self {
        self.scale(&Rational::from_integer(-1))
    }

    fn scale(mut self, scale: &Rational) -> Self {
        self.constant = self.constant * scale.clone();
        for term in &mut self.terms {
            term.coefficient = term.coefficient.clone() * scale.clone();
        }
        self.remove_zero_terms();
        self
    }

    fn divide_by_rational(mut self, denominator: &Rational) -> Result<Self, CalculatorError> {
        if denominator.is_zero() {
            return Err(CalculatorError::DivisionByZero);
        }
        self.constant = self.constant / denominator.clone();
        for term in &mut self.terms {
            term.coefficient = term.coefficient.clone() / denominator.clone();
        }
        self.remove_zero_terms();
        Ok(self)
    }

    fn add_term(&mut self, variable: String, coefficient: Rational) {
        if coefficient.is_zero() {
            return;
        }

        if let Some(index) = self.terms.iter().position(|term| term.variable == variable) {
            let new_coefficient = self.terms[index].coefficient.clone() + coefficient;
            if new_coefficient.is_zero() {
                self.terms.remove(index);
            } else {
                self.terms[index].coefficient = new_coefficient;
            }
            return;
        }

        self.terms.push(LinearTerm {
            variable,
            coefficient,
        });
    }

    fn remove_zero_terms(&mut self) {
        self.terms.retain(|term| !term.coefficient.is_zero());
    }

    fn has_variable_terms(&self) -> bool {
        !self.terms.is_empty()
    }

    fn coefficient_of(&self, variable: &str) -> Rational {
        self.terms
            .iter()
            .find(|term| term.variable == variable)
            .map_or_else(Rational::zero, |term| term.coefficient.clone())
    }

    fn push_variable_order(&self, variables: &mut Vec<String>) {
        for term in &self.terms {
            if !variables.iter().any(|variable| variable == &term.variable) {
                variables.push(term.variable.clone());
            }
        }
    }

    fn unsupported_equation() -> CalculatorError {
        CalculatorError::InvalidOperation(
            "only linear equations with unitless numbers are supported".into(),
        )
    }

    fn format_terms_first(&self) -> String {
        self.format(false)
    }

    fn format_solution_expression(&self) -> String {
        self.format(true)
    }

    fn format(&self, constant_first: bool) -> String {
        let mut parts = Vec::new();

        if constant_first {
            Self::push_constant_part(&mut parts, &self.constant);
        }

        for term in &self.terms {
            Self::push_variable_part(&mut parts, &term.coefficient, &term.variable);
        }

        if !constant_first {
            Self::push_constant_part(&mut parts, &self.constant);
        }

        Self::join_signed_parts(parts)
    }

    fn push_constant_part(parts: &mut Vec<(bool, String)>, value: &Rational) {
        if value.is_zero() {
            return;
        }
        parts.push((value.is_negative(), value.abs().to_display_string()));
    }

    fn push_variable_part(parts: &mut Vec<(bool, String)>, coefficient: &Rational, variable: &str) {
        if coefficient.is_zero() {
            return;
        }
        parts.push((
            coefficient.is_negative(),
            Self::format_variable_body(&coefficient.abs(), variable),
        ));
    }

    fn join_signed_parts(parts: Vec<(bool, String)>) -> String {
        let mut result = String::new();

        for (index, (is_negative, body)) in parts.into_iter().enumerate() {
            if index == 0 {
                if is_negative {
                    result.push('-');
                }
            } else if is_negative {
                result.push_str(" - ");
            } else {
                result.push_str(" + ");
            }
            result.push_str(&body);
        }

        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }

    fn format_variable_term(coefficient: &Rational, variable: &str) -> String {
        if coefficient.is_zero() {
            return "0".to_string();
        }
        Self::join_signed_parts(vec![(
            coefficient.is_negative(),
            Self::format_variable_body(&coefficient.abs(), variable),
        )])
    }

    fn format_variable_body(coefficient_abs: &Rational, variable: &str) -> String {
        if coefficient_abs == &Rational::one() {
            variable.to_string()
        } else if variable == "*" {
            format!("{}*(*)", coefficient_abs.to_display_string())
        } else {
            format!("{}*{variable}", coefficient_abs.to_display_string())
        }
    }
}

impl LinearEquationSolution {
    pub(super) fn to_value(&self) -> Value {
        if self.value_expression.terms.is_empty() {
            Value::equation_solution(
                self.variable.clone(),
                self.value_expression.constant.clone(),
            )
        } else {
            Value::symbolic_equation_solution(
                self.variable.clone(),
                self.value_expression.format_solution_expression(),
            )
        }
    }

    pub(super) fn derivation_steps(&self) -> Vec<String> {
        let mut steps = vec![
            format!(
                "Original equation: {} = {}",
                self.left_form.format_terms_first(),
                self.right_form.format_terms_first()
            ),
            format!(
                "Linear form: {} = {}",
                self.left_form.format_terms_first(),
                self.right_form.format_terms_first()
            ),
            format!("Choose target variable: {}", self.variable),
            format!(
                "Combine target coefficients: {} - {} = {}",
                self.left_form
                    .coefficient_of(&self.variable)
                    .to_display_string(),
                self.right_form
                    .coefficient_of(&self.variable)
                    .to_display_string(),
                self.coefficient.to_display_string()
            ),
            format!(
                "Move non-target variable terms to the right: {}",
                self.format_variable_move_details()
            ),
            format!(
                "Move constants to the right: {} - {} = {}",
                self.right_form.constant.to_display_string(),
                self.left_form.constant.to_display_string(),
                self.isolated_expression.constant.to_display_string()
            ),
            format!(
                "Right side after moving terms: {}",
                self.isolated_expression.format_solution_expression()
            ),
            format!(
                "Isolate variable term: {} = {}",
                LinearForm::format_variable_term(&self.coefficient, &self.variable),
                self.isolated_expression.format_solution_expression()
            ),
            format!(
                "Divide both sides by {}: {} = {}",
                self.coefficient.to_display_string(),
                self.variable,
                self.value_expression.format_solution_expression()
            ),
        ];

        steps.push(format!(
            "Solve for {}: {} = {}",
            self.variable,
            self.variable,
            self.value_expression.format_solution_expression()
        ));

        steps
    }

    fn format_variable_move_details(&self) -> String {
        if self.variable_move_details.is_empty() {
            return "no other variables to move".to_string();
        }

        self.variable_move_details
            .iter()
            .map(|detail| {
                format!(
                    "{}: {} - {} = {}",
                    detail.variable,
                    detail.right_coefficient.to_display_string(),
                    detail.left_coefficient.to_display_string(),
                    detail.moved_coefficient.to_display_string()
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub(super) fn solve(
    left: &Expression,
    right: &Expression,
) -> Result<LinearEquationSolution, CalculatorError> {
    let left_form = LinearForm::from_expression(left)?;
    let right_form = LinearForm::from_expression(right)?;
    let variables = collect_variable_order(&left_form, &right_form);
    let variable =
        select_target_variable(&left_form, &right_form, &variables).ok_or_else(|| {
            CalculatorError::InvalidOperation("linear equation has no unique solution".into())
        })?;

    let coefficient = left_form.coefficient_of(&variable) - right_form.coefficient_of(&variable);
    if coefficient.is_zero() {
        return Err(CalculatorError::InvalidOperation(
            "linear equation has no unique solution".into(),
        ));
    }

    let isolated_expression = isolate_right_side(&left_form, &right_form, &variables, &variable);
    let value_expression = isolated_expression
        .clone()
        .divide_by_rational(&coefficient)?;
    let variable_move_details =
        build_variable_move_details(&left_form, &right_form, &variables, &variable);

    Ok(LinearEquationSolution {
        variable,
        coefficient,
        isolated_expression,
        value_expression,
        left_form,
        right_form,
        variable_move_details,
    })
}

fn collect_variable_order(left: &LinearForm, right: &LinearForm) -> Vec<String> {
    let mut variables = Vec::new();
    left.push_variable_order(&mut variables);
    right.push_variable_order(&mut variables);
    variables
}

fn select_target_variable(
    left: &LinearForm,
    right: &LinearForm,
    variables: &[String],
) -> Option<String> {
    for preferred in ["?", "*"] {
        if variables.iter().any(|variable| variable == preferred)
            && !combined_target_coefficient(left, right, preferred).is_zero()
        {
            return Some(preferred.to_string());
        }
    }

    variables
        .iter()
        .find(|variable| !combined_target_coefficient(left, right, variable).is_zero())
        .cloned()
}

fn combined_target_coefficient(left: &LinearForm, right: &LinearForm, variable: &str) -> Rational {
    left.coefficient_of(variable) - right.coefficient_of(variable)
}

fn isolate_right_side(
    left: &LinearForm,
    right: &LinearForm,
    variables: &[String],
    target_variable: &str,
) -> LinearForm {
    let mut isolated = LinearForm::constant(right.constant.clone() - left.constant.clone());

    for variable in variables {
        if variable == target_variable {
            continue;
        }

        let coefficient = right.coefficient_of(variable) - left.coefficient_of(variable);
        isolated.add_term(variable.clone(), coefficient);
    }

    isolated
}

fn build_variable_move_details(
    left: &LinearForm,
    right: &LinearForm,
    variables: &[String],
    target_variable: &str,
) -> Vec<VariableMoveDetail> {
    variables
        .iter()
        .filter(|variable| variable.as_str() != target_variable)
        .map(|variable| {
            let left_coefficient = left.coefficient_of(variable);
            let right_coefficient = right.coefficient_of(variable);
            let moved_coefficient = right_coefficient.clone() - left_coefficient.clone();

            VariableMoveDetail {
                variable: variable.clone(),
                left_coefficient,
                right_coefficient,
                moved_coefficient,
            }
        })
        .collect()
}
