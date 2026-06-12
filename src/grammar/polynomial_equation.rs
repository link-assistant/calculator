//! Polynomial-equation helpers for the expression evaluator.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::CalculatorError;
use crate::types::{BinaryOp, Expression, Rational, Unit, Value};

const MAX_POLYNOMIAL_DEGREE: u32 = 32;
const ROOT_SEARCH_BOUND: i128 = 100;
const MAX_FACTORABLE_COEFFICIENT_ABS: i128 = 1_000_000_000;

#[derive(Debug, Clone)]
struct PolynomialForm {
    variable: Option<String>,
    terms: BTreeMap<u32, Rational>,
}

#[derive(Debug, Clone)]
pub(super) struct PolynomialEquationSolution {
    variable: String,
    polynomial: PolynomialForm,
    degree: u32,
    roots: Vec<Rational>,
    original_left: String,
    original_right: String,
}

impl PolynomialForm {
    fn constant(value: Rational) -> Self {
        let mut terms = BTreeMap::new();
        if !value.is_zero() {
            terms.insert(0, value);
        }
        Self {
            variable: None,
            terms,
        }
    }

    fn variable(name: String) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(1, Rational::one());
        Self {
            variable: Some(name),
            terms,
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
                    BinaryOp::Divide => left.divide(&right),
                    BinaryOp::Modulo => Err(Self::unsupported_equation()),
                }
            }
            Expression::Negate(inner) => Ok(Self::from_expression(inner)?.negate()),
            Expression::Group(inner) => Self::from_expression(inner),
            Expression::Power { base, exponent } => {
                let base = Self::from_expression(base)?;
                let exponent =
                    Self::from_expression(exponent)?.as_non_negative_integer_exponent()?;
                base.pow(exponent)
            }
            Expression::DateTime(_)
            | Expression::Now
            | Expression::Until(_)
            | Expression::AtTime { .. }
            | Expression::FunctionCall { .. }
            | Expression::IndefiniteIntegral { .. }
            | Expression::UnitConversion { .. }
            | Expression::Equality { .. } => Err(Self::unsupported_equation()),
        }
    }

    fn add(self, other: Self) -> Result<Self, CalculatorError> {
        let variable = merge_variable_names(self.variable, other.variable)?;
        let mut result = Self {
            variable,
            terms: self.terms,
        };

        for (degree, coefficient) in other.terms {
            result.add_term(degree, coefficient);
        }

        Ok(result)
    }

    fn subtract(self, other: Self) -> Result<Self, CalculatorError> {
        self.add(other.negate())
    }

    fn multiply(self, other: Self) -> Result<Self, CalculatorError> {
        let variable = merge_variable_names(self.variable, other.variable)?;
        let mut result = Self {
            variable,
            terms: BTreeMap::new(),
        };

        for (left_degree, left_coefficient) in self.terms {
            for (right_degree, right_coefficient) in &other.terms {
                let degree = left_degree
                    .checked_add(*right_degree)
                    .filter(|degree| *degree <= MAX_POLYNOMIAL_DEGREE)
                    .ok_or_else(Self::unsupported_equation)?;
                result.add_term(degree, left_coefficient.clone() * right_coefficient.clone());
            }
        }

        Ok(result)
    }

    fn divide(self, other: &Self) -> Result<Self, CalculatorError> {
        if other.has_variable_terms() {
            return Err(CalculatorError::InvalidOperation(
                "polynomial equation has a variable denominator".into(),
            ));
        }

        let denominator = other.coefficient(0);
        if denominator.is_zero() {
            return Err(CalculatorError::DivisionByZero);
        }

        self.divide_by_rational(&denominator)
    }

    fn pow(self, exponent: u32) -> Result<Self, CalculatorError> {
        if exponent == 0 {
            return Ok(Self::constant(Rational::one()));
        }

        let mut result = Self::constant(Rational::one());
        for _ in 0..exponent {
            result = result.multiply(self.clone())?;
        }

        Ok(result)
    }

    fn negate(self) -> Self {
        self.scale(&Rational::from_integer(-1))
    }

    fn scale(mut self, scale: &Rational) -> Self {
        for coefficient in self.terms.values_mut() {
            *coefficient = coefficient.clone() * scale.clone();
        }
        self.remove_zero_terms();
        self
    }

    fn divide_by_rational(mut self, denominator: &Rational) -> Result<Self, CalculatorError> {
        if denominator.is_zero() {
            return Err(CalculatorError::DivisionByZero);
        }

        for coefficient in self.terms.values_mut() {
            *coefficient = coefficient.clone() / denominator.clone();
        }
        self.remove_zero_terms();
        Ok(self)
    }

    fn add_term(&mut self, degree: u32, coefficient: Rational) {
        if coefficient.is_zero() {
            return;
        }

        let new_coefficient = self.coefficient(degree) + coefficient;
        if new_coefficient.is_zero() {
            self.terms.remove(&degree);
        } else {
            self.terms.insert(degree, new_coefficient);
        }
    }

    fn remove_zero_terms(&mut self) {
        self.terms.retain(|_, coefficient| !coefficient.is_zero());
        if !self.has_variable_terms() {
            self.variable = None;
        }
    }

    fn has_variable_terms(&self) -> bool {
        self.terms.keys().any(|degree| *degree > 0)
    }

    fn coefficient(&self, degree: u32) -> Rational {
        self.terms
            .get(&degree)
            .map_or_else(Rational::zero, Clone::clone)
    }

    fn degree(&self) -> u32 {
        self.terms.keys().next_back().copied().unwrap_or(0)
    }

    fn as_non_negative_integer_exponent(&self) -> Result<u32, CalculatorError> {
        if self.has_variable_terms() {
            return Err(Self::unsupported_equation());
        }

        let exponent = self.coefficient(0);
        if !exponent.is_integer() {
            return Err(Self::unsupported_equation());
        }

        let exponent = exponent.numer();
        if exponent < 0 || exponent > i128::from(MAX_POLYNOMIAL_DEGREE) {
            return Err(Self::unsupported_equation());
        }

        u32::try_from(exponent).map_err(|_| Self::unsupported_equation())
    }

    fn evaluate_at(&self, value: &Rational) -> Rational {
        let mut result = Rational::zero();

        for degree in (0..=self.degree()).rev() {
            result = result * value.clone() + self.coefficient(degree);
        }

        result
    }

    fn format(&self) -> String {
        let Some(variable) = &self.variable else {
            return self.coefficient(0).to_display_string();
        };

        let parts = self
            .terms
            .iter()
            .rev()
            .map(|(degree, coefficient)| {
                let coefficient_abs = coefficient.abs();
                (
                    coefficient.is_negative(),
                    Self::format_term_body(&coefficient_abs, *degree, variable),
                )
            })
            .collect::<Vec<_>>();

        join_signed_parts(parts)
    }

    fn format_term_body(coefficient_abs: &Rational, degree: u32, variable: &str) -> String {
        if degree == 0 {
            return coefficient_abs.to_display_string();
        }

        let variable_power = format_variable_power(variable, degree);
        if coefficient_abs == &Rational::one() {
            variable_power
        } else {
            format!("{}*{variable_power}", coefficient_abs.to_display_string())
        }
    }

    fn unsupported_equation() -> CalculatorError {
        CalculatorError::InvalidOperation(
            "only polynomial equations with one variable, unitless numbers, nonnegative integer powers, and real rational roots are supported".into(),
        )
    }
}

impl PolynomialEquationSolution {
    pub(super) fn to_value(&self) -> Value {
        if self.roots.len() == 1 {
            Value::equation_solution(self.variable.clone(), self.roots[0].clone())
        } else {
            Value::equation_solutions(self.variable.clone(), self.roots.clone())
        }
    }

    pub(super) fn derivation_steps(&self) -> Vec<String> {
        let mut steps = vec![
            format!(
                "Original equation: {} = {}",
                self.original_left, self.original_right
            ),
            format!("Move all terms to the left: {} = 0", self.polynomial.format()),
            format!("Polynomial form: {} = 0", self.polynomial.format()),
            format!("Polynomial degree: {}", self.degree),
            format!("Choose target variable: {}", self.variable),
            "Find real rational roots: test exact rational candidates and keep the values that make the polynomial equal 0".to_string(),
        ];

        for root in &self.roots {
            steps.push(format!(
                "Verify root {}: substituting {} = {} gives 0",
                root.to_display_string(),
                self.variable,
                root.to_display_string()
            ));
        }

        steps.push(format!(
            "Factor roots: {}",
            format_root_factors(&self.variable, &self.roots)
        ));
        steps.push(format!(
            "Solutions: {}",
            format_solutions(&self.variable, &self.roots)
        ));

        steps
    }
}

pub(super) fn solve(
    left: &Expression,
    right: &Expression,
) -> Result<PolynomialEquationSolution, CalculatorError> {
    let left_form = PolynomialForm::from_expression(left)?;
    let right_form = PolynomialForm::from_expression(right)?;
    let polynomial = left_form.subtract(right_form)?;
    let variable = polynomial.variable.clone().ok_or_else(|| {
        CalculatorError::InvalidOperation("polynomial equation has no variable".into())
    })?;
    let degree = polynomial.degree();

    if degree <= 1 {
        return Err(CalculatorError::InvalidOperation(
            "polynomial equation is linear".into(),
        ));
    }

    let roots = find_real_rational_roots(&polynomial);
    if roots.is_empty() {
        return Err(CalculatorError::InvalidOperation(
            "polynomial equation has no supported real rational solutions".into(),
        ));
    }

    Ok(PolynomialEquationSolution {
        variable,
        polynomial,
        degree,
        roots,
        original_left: left.to_string(),
        original_right: right.to_string(),
    })
}

fn merge_variable_names(
    left: Option<String>,
    right: Option<String>,
) -> Result<Option<String>, CalculatorError> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(CalculatorError::InvalidOperation(
            "polynomial equations with multiple variables are not supported".into(),
        )),
        (Some(variable), _) | (_, Some(variable)) => Ok(Some(variable)),
        (None, None) => Ok(None),
    }
}

fn find_real_rational_roots(polynomial: &PolynomialForm) -> Vec<Rational> {
    if let Some(roots) = binomial_power_roots(polynomial) {
        return roots;
    }

    let mut roots = BTreeSet::new();

    if polynomial.degree() == 2 {
        roots.extend(quadratic_roots(polynomial));
    }

    for candidate in rational_root_candidates(polynomial) {
        if polynomial.evaluate_at(&candidate).is_zero() {
            roots.insert(candidate);
        }
    }

    roots.into_iter().collect()
}

fn binomial_power_roots(polynomial: &PolynomialForm) -> Option<Vec<Rational>> {
    let degree = polynomial.degree();
    if degree < 2 {
        return None;
    }

    let has_only_high_and_constant_terms = polynomial
        .terms
        .keys()
        .all(|term_degree| *term_degree == 0 || *term_degree == degree);
    if !has_only_high_and_constant_terms {
        return None;
    }

    let leading = polynomial.coefficient(degree);
    if leading.is_zero() {
        return None;
    }

    let target = -polynomial.coefficient(0) / leading;
    if target.is_zero() {
        return Some(vec![Rational::zero()]);
    }

    if target.is_negative() && degree % 2 == 0 {
        return Some(Vec::new());
    }

    let root = rational_nth_root(&target, degree)?;
    if degree % 2 == 0 {
        let negative_root = -root.clone();
        Some(vec![negative_root, root])
    } else {
        Some(vec![root])
    }
}

fn quadratic_roots(polynomial: &PolynomialForm) -> Vec<Rational> {
    let a = polynomial.coefficient(2);
    let b = polynomial.coefficient(1);
    let c = polynomial.coefficient(0);
    if a.is_zero() {
        return Vec::new();
    }

    let four = Rational::from_integer(4);
    let two = Rational::from_integer(2);
    let discriminant = b.clone() * b.clone() - four * a.clone() * c;
    let Some(square_root) = rational_square_root(&discriminant) else {
        return Vec::new();
    };

    let denominator = two * a;
    let first = (-b.clone() - square_root.clone()) / denominator.clone();
    let second = (-b + square_root) / denominator;

    match first.cmp(&second) {
        std::cmp::Ordering::Equal => vec![first],
        std::cmp::Ordering::Less => vec![first, second],
        std::cmp::Ordering::Greater => vec![second, first],
    }
}

fn rational_square_root(value: &Rational) -> Option<Rational> {
    rational_nth_root(value, 2)
}

fn rational_nth_root(value: &Rational, degree: u32) -> Option<Rational> {
    if value.is_negative() {
        if degree % 2 == 0 {
            return None;
        }

        return rational_nth_root(&value.abs(), degree).map(std::ops::Neg::neg);
    }

    let numerator = u128::try_from(value.numer()).ok()?;
    let denominator = u128::try_from(value.denom()).ok()?;
    let numerator_root = integer_nth_root(numerator, degree)?;
    let denominator_root = integer_nth_root(denominator, degree)?;

    Some(Rational::new(
        i128::try_from(numerator_root).ok()?,
        i128::try_from(denominator_root).ok()?,
    ))
}

fn integer_nth_root(value: u128, degree: u32) -> Option<u128> {
    let mut low = 0;
    let mut high = value;

    while low <= high {
        let midpoint = low + (high - low) / 2;
        match checked_pow_u128(midpoint, degree) {
            Some(power) if power == value => return Some(midpoint),
            Some(power) if power < value => low = midpoint + 1,
            _ => {
                if midpoint == 0 {
                    break;
                }
                high = midpoint - 1;
            }
        }
    }

    None
}

fn checked_pow_u128(base: u128, exponent: u32) -> Option<u128> {
    let mut result = 1_u128;

    for _ in 0..exponent {
        result = result.checked_mul(base)?;
    }

    Some(result)
}

fn rational_root_candidates(polynomial: &PolynomialForm) -> BTreeSet<Rational> {
    let mut candidates = (-ROOT_SEARCH_BOUND..=ROOT_SEARCH_BOUND)
        .map(Rational::from_integer)
        .collect::<BTreeSet<_>>();

    if let Some(exact_candidates) = rational_root_theorem_candidates(polynomial) {
        candidates.extend(exact_candidates);
    }

    candidates
}

fn rational_root_theorem_candidates(polynomial: &PolynomialForm) -> Option<BTreeSet<Rational>> {
    let lowest_degree = *polynomial.terms.keys().next()?;
    let highest_degree = polynomial.degree();
    let mut denominator_lcm = 1_i128;

    for coefficient in polynomial.terms.values() {
        denominator_lcm = checked_lcm(denominator_lcm, coefficient.denom())?;
    }

    let constant = scaled_integer_coefficient(polynomial, lowest_degree, denominator_lcm)?;
    let leading = scaled_integer_coefficient(polynomial, highest_degree, denominator_lcm)?;
    let numerator_factors = positive_divisors(constant)?;
    let denominator_factors = positive_divisors(leading)?;
    let mut candidates = BTreeSet::new();

    for numerator in numerator_factors {
        for denominator in &denominator_factors {
            candidates.insert(Rational::new(numerator, *denominator));
            candidates.insert(Rational::new(-numerator, *denominator));
        }
    }

    if lowest_degree > 0 {
        candidates.insert(Rational::zero());
    }

    Some(candidates)
}

fn scaled_integer_coefficient(
    polynomial: &PolynomialForm,
    degree: u32,
    denominator_lcm: i128,
) -> Option<i128> {
    let coefficient = polynomial.coefficient(degree);
    coefficient
        .numer()
        .checked_mul(denominator_lcm.checked_div(coefficient.denom())?)
}

fn checked_lcm(left: i128, right: i128) -> Option<i128> {
    let gcd = gcd_i128(left, right);
    left.checked_div(gcd)?.checked_mul(right)?.checked_abs()
}

fn gcd_i128(mut left: i128, mut right: i128) -> i128 {
    left = left.abs();
    right = right.abs();

    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }

    left
}

fn positive_divisors(value: i128) -> Option<Vec<i128>> {
    let value = value.checked_abs()?;
    if value == 0 || value > MAX_FACTORABLE_COEFFICIENT_ABS {
        return None;
    }

    let mut divisors = Vec::new();
    let mut candidate = 1_i128;
    while candidate <= value / candidate {
        if value % candidate == 0 {
            divisors.push(candidate);
            let paired = value / candidate;
            if paired != candidate {
                divisors.push(paired);
            }
        }
        candidate += 1;
    }

    Some(divisors)
}

fn format_solutions(variable: &str, roots: &[Rational]) -> String {
    roots
        .iter()
        .map(|root| format!("{variable} = {}", root.to_display_string()))
        .collect::<Vec<_>>()
        .join(" or ")
}

fn format_root_factors(variable: &str, roots: &[Rational]) -> String {
    roots
        .iter()
        .map(|root| format_root_factor(variable, root))
        .collect::<Vec<_>>()
        .join(" * ")
}

fn format_root_factor(variable: &str, root: &Rational) -> String {
    let variable = format_variable_power(variable, 1);
    if root.is_zero() {
        return variable;
    }

    if root.is_negative() {
        format!("({variable} + {})", root.abs().to_display_string())
    } else {
        format!("({variable} - {})", root.to_display_string())
    }
}

fn format_variable_power(variable: &str, degree: u32) -> String {
    let variable = if variable == "*" {
        "(*)".to_string()
    } else {
        variable.to_string()
    };

    if degree == 1 {
        variable
    } else {
        format!("{variable}^{degree}")
    }
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
