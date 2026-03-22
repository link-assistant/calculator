//! Expression types for the calculator grammar.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::{DateTime, Decimal, Unit};

/// A binary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOp {
    /// Returns the symbol for the operation.
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        }
    }

    /// Returns the precedence of the operation (higher = binds tighter).
    #[must_use]
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide => 2,
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// An expression in the calculator grammar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// A literal number.
    Number {
        value: Decimal,
        unit: Unit,
        /// Alternative unit interpretations for ambiguous identifiers
        /// (e.g., "ton" → Mass(MetricTon) primary, Currency("TON") alternative).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        alternative_units: Vec<Unit>,
    },
    /// A literal datetime.
    DateTime(DateTime),
    /// The current time ("now").
    Now,
    /// "until <datetime>" - duration from now to a target datetime.
    Until(Box<Expression>),
    /// A binary operation.
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    /// A unary negation.
    Negate(Box<Expression>),
    /// A grouped expression (parenthesized).
    Group(Box<Expression>),
    /// A temporal context for a value (e.g., "at 22 Jan 2026").
    AtTime {
        value: Box<Expression>,
        time: Box<Expression>,
    },
    /// A function call (e.g., sin(x), sqrt(x), integrate(expr, var, lower, upper)).
    FunctionCall { name: String, args: Vec<Expression> },
    /// A variable reference (used in integration expressions).
    Variable(String),
    /// Power/exponentiation expression (x^y).
    Power {
        base: Box<Expression>,
        exponent: Box<Expression>,
    },
    /// Indefinite integral expression (e.g., "integrate sin(x)/x dx").
    IndefiniteIntegral {
        /// The integrand expression.
        integrand: Box<Expression>,
        /// The variable of integration.
        variable: String,
    },
    /// Unit conversion expression (e.g., "741 KB as MB").
    UnitConversion {
        /// The expression to convert.
        value: Box<Expression>,
        /// The target unit.
        target_unit: Unit,
    },
    /// Equality check expression (e.g., `1 * (2 / 3) = (1 * 2) / 3`).
    Equality {
        /// The left-hand side expression.
        left: Box<Expression>,
        /// The right-hand side expression.
        right: Box<Expression>,
    },
}

impl Expression {
    /// Creates a number expression.
    #[must_use]
    pub fn number(value: Decimal) -> Self {
        Self::Number {
            value,
            unit: Unit::None,
            alternative_units: Vec::new(),
        }
    }

    /// Creates a number expression with a unit.
    #[must_use]
    pub fn number_with_unit(value: Decimal, unit: Unit) -> Self {
        Self::Number {
            value,
            unit,
            alternative_units: Vec::new(),
        }
    }

    /// Creates a number expression with a unit and alternative unit interpretations.
    #[must_use]
    pub fn number_with_unit_alternatives(
        value: Decimal,
        unit: Unit,
        alternative_units: Vec<Unit>,
    ) -> Self {
        Self::Number {
            value,
            unit,
            alternative_units,
        }
    }

    /// Creates a currency expression.
    #[must_use]
    pub fn currency(amount: Decimal, code: &str) -> Self {
        Self::Number {
            value: amount,
            unit: Unit::currency(code),
            alternative_units: Vec::new(),
        }
    }

    /// Creates a binary expression.
    #[must_use]
    pub fn binary(left: Expression, op: BinaryOp, right: Expression) -> Self {
        Self::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// Creates a negation expression.
    #[must_use]
    pub fn negate(expr: Expression) -> Self {
        Self::Negate(Box::new(expr))
    }

    /// Creates a grouped expression.
    #[must_use]
    pub fn group(expr: Expression) -> Self {
        Self::Group(Box::new(expr))
    }

    /// Creates an at-time expression.
    #[must_use]
    pub fn at_time(value: Expression, time: Expression) -> Self {
        Self::AtTime {
            value: Box::new(value),
            time: Box::new(time),
        }
    }

    /// Creates a function call expression.
    #[must_use]
    pub fn function_call(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Self::FunctionCall {
            name: name.into(),
            args,
        }
    }

    /// Creates a variable expression.
    #[must_use]
    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    /// Creates a power expression.
    #[must_use]
    pub fn power(base: Expression, exponent: Expression) -> Self {
        Self::Power {
            base: Box::new(base),
            exponent: Box::new(exponent),
        }
    }

    /// Creates an indefinite integral expression.
    #[must_use]
    pub fn indefinite_integral(integrand: Expression, variable: impl Into<String>) -> Self {
        Self::IndefiniteIntegral {
            integrand: Box::new(integrand),
            variable: variable.into(),
        }
    }

    /// Creates a unit conversion expression (e.g., "741 KB as MB").
    #[must_use]
    pub fn unit_conversion(value: Expression, target_unit: Unit) -> Self {
        Self::UnitConversion {
            value: Box::new(value),
            target_unit,
        }
    }

    /// Creates an equality check expression (e.g., `1 + 1 = 2`).
    #[must_use]
    pub fn equality(left: Expression, right: Expression) -> Self {
        Self::Equality {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Converts the expression to links notation format.
    ///
    /// Links notation wraps all compound expressions in parentheses:
    /// - Atomic values (numbers, variables) don't need parentheses
    /// - All other expressions are wrapped in outer `()`
    /// - Function calls use space-separated args: `(func (arg1 arg2 arg3))`
    /// - Power uses `^` operator with spaces: `(x ^ 2)`
    #[must_use]
    pub fn to_lino(&self) -> String {
        self.to_lino_internal(None)
    }

    /// Internal helper for to_lino.
    /// `parent_op` is the parent operator's precedence (if any) to determine
    /// if we need parentheses for this subexpression.
    fn to_lino_internal(&self, _parent_op: Option<&BinaryOp>) -> String {
        match self {
            Self::Number { value, unit, .. } => {
                let num_str = value.to_string();
                if *unit == Unit::None {
                    num_str
                } else {
                    format!("({num_str} {unit})")
                }
            }
            Self::DateTime(dt) => format!("({})", dt),
            Self::Now => "(now)".to_string(),
            Self::Until(inner) => {
                let inner_str = inner.to_lino_internal(None);
                format!("(until {inner_str})")
            }
            Self::Binary { left, op, right } => {
                let left_str = left.to_lino_internal(Some(op));
                let right_str = right.to_lino_internal(Some(op));
                let expr_str = format!("{left_str} {op} {right_str}");
                format!("({expr_str})")
            }
            Self::Negate(inner) => {
                let inner_str = inner.to_lino_internal(None);
                // Only add parens if the inner expression needs them
                if inner.needs_parens_for_unary() {
                    format!("(-({inner_str}))")
                } else {
                    format!("(-{inner_str})")
                }
            }
            Self::Group(inner) => {
                // Groups are explicitly requested by the user
                // If the inner expression already has its own parentheses, don't double-wrap
                let inner_str = inner.to_lino_internal(None);
                if inner_str.starts_with('(') && inner_str.ends_with(')') {
                    inner_str
                } else {
                    format!("({inner_str})")
                }
            }
            Self::AtTime { value, time } => {
                let value_str = value.to_lino_internal(None);
                let time_str = time.to_lino_internal(None);
                format!("({value_str} at {time_str})")
            }
            Self::FunctionCall { name, args } => {
                if args.is_empty() {
                    format!("({name})")
                } else {
                    let args_str = args
                        .iter()
                        .map(|a| a.to_lino_internal(None))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("({name} ({args_str}))")
                }
            }
            Self::Variable(name) => name.clone(),
            Self::Power { base, exponent } => {
                let base_str = base.to_lino_internal(None);
                let exp_str = exponent.to_lino_internal(None);
                format!("({base_str} ^ {exp_str})")
            }
            Self::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                let integrand_str = integrand.to_lino_internal(None);
                format!("(integrate ({integrand_str} * (differential of ({variable}))))")
            }
            Self::UnitConversion { value, target_unit } => {
                let value_str = value.to_lino_internal(None);
                format!("({value_str} as {target_unit})")
            }
            Self::Equality { left, right } => {
                let left_str = left.to_lino_internal(None);
                let right_str = right.to_lino_internal(None);
                format!("({left_str} = {right_str})")
            }
        }
    }

    /// Generates alternative links notation interpretations for this expression.
    ///
    /// Returns `None` if there is only one natural interpretation.
    /// Returns `Some(vec)` with the default interpretation first, followed by alternatives.
    ///
    /// Ambiguity arises from:
    /// - Binary operations where precedence changes meaning (e.g., `2 + 3 * 4`)
    /// - Function calls that could also be read as other structures
    #[must_use]
    pub fn alternative_lino(&self) -> Option<Vec<String>> {
        let default_lino = self.to_lino();
        let mut alternatives = Vec::new();

        self.collect_alternatives(&mut alternatives);

        if alternatives.is_empty() {
            return None;
        }

        // Remove duplicates and the default interpretation
        alternatives.retain(|a| a != &default_lino);
        alternatives.dedup();

        if alternatives.is_empty() {
            return None;
        }

        // Default first, then alternatives
        let mut result = vec![default_lino];
        result.extend(alternatives);
        Some(result)
    }

    /// Collects alternative lino representations based on expression structure.
    fn collect_alternatives(&self, alternatives: &mut Vec<String>) {
        match self {
            // For numbers with ambiguous units, generate alternatives for each interpretation
            Self::Number {
                value,
                alternative_units,
                ..
            } if !alternative_units.is_empty() => {
                for alt_unit in alternative_units {
                    let num_str = value.to_string();
                    let alt = if *alt_unit == Unit::None {
                        num_str
                    } else {
                        format!("({num_str} {alt_unit})")
                    };
                    alternatives.push(alt);
                }
            }
            // For expressions with mixed precedence, show the explicit grouping
            Self::Binary { left, op, right } => {
                // Check if either child has a different-precedence binary operation
                // e.g., in `2 + 3 * 4`, show both `(2 + (3 * 4))` and `((2 + 3) * 4)`
                let has_different_precedence_child =
                    Self::has_different_precedence_child(left, *op)
                        || Self::has_different_precedence_child(right, *op);

                if has_different_precedence_child {
                    // Alternative: re-group with opposite precedence assumption
                    // left-to-right grouping
                    let alt = self.to_lino_left_to_right();
                    alternatives.push(alt);
                }

                // Also collect alternatives from children (e.g., ambiguous units in operands)
                left.collect_alternatives(alternatives);
                right.collect_alternatives(alternatives);
            }
            // For function calls with multiple args, show the mathematical notation alternative
            Self::FunctionCall { name, args } if !args.is_empty() => {
                // Alternative: traditional mathematical notation
                let args_str = args
                    .iter()
                    .map(|a| a.to_lino_internal(None))
                    .collect::<Vec<_>>()
                    .join(", ");
                let alt = format!("(expression \"{name}({args_str})\")");
                alternatives.push(alt);
            }
            _ => {}
        }
    }

    /// Checks if a child expression has a binary op with different precedence.
    fn has_different_precedence_child(child: &Expression, parent_op: BinaryOp) -> bool {
        if let Self::Binary { op, .. } = child {
            return op.precedence() != parent_op.precedence();
        }
        false
    }

    /// Generates a left-to-right grouped lino notation (alternative grouping).
    fn to_lino_left_to_right(&self) -> String {
        match self {
            Self::Binary { left, op, right } => {
                // In left-to-right: ((left op1 right_left) op2 right_right)
                // or: (left_left op1 (left_right op2 right))
                // depending on where the different-precedence child is
                if let Self::Binary {
                    left: rl,
                    op: rop,
                    right: rr,
                } = right.as_ref()
                {
                    if rop.precedence() != op.precedence() {
                        // Default: left op (rl rop rr) => already `(left op (rl rop rr))`
                        // Alternative: (left op rl) rop rr => `((left op rl) rop rr)`
                        let new_left =
                            Self::binary(left.as_ref().clone(), *op, rl.as_ref().clone());
                        let new_expr = Self::binary(new_left, *rop, rr.as_ref().clone());
                        return new_expr.to_lino();
                    }
                }
                if let Self::Binary {
                    left: ll,
                    op: lop,
                    right: lr,
                } = left.as_ref()
                {
                    if lop.precedence() != op.precedence() {
                        // Default: (ll lop lr) op right => already `((ll lop lr) op right)`
                        // Alternative: ll lop (lr op right) => `(ll lop (lr op right))`
                        let new_right =
                            Self::binary(lr.as_ref().clone(), *op, right.as_ref().clone());
                        let new_expr = Self::binary(ll.as_ref().clone(), *lop, new_right);
                        return new_expr.to_lino();
                    }
                }
                self.to_lino()
            }
            _ => self.to_lino(),
        }
    }

    /// Returns true if this expression needs parentheses when used as a unary operand.
    fn needs_parens_for_unary(&self) -> bool {
        matches!(
            self,
            Self::Binary { .. } | Self::AtTime { .. } | Self::UnitConversion { .. }
        )
    }

    /// Returns true if this expression needs parentheses when used as a power base.
    /// Note: Currently unused since `to_lino()` always wraps Power in parens,
    /// but kept for potential use in `to_latex()` or other representations.
    #[allow(dead_code)]
    fn needs_parens_for_power(&self) -> bool {
        matches!(
            self,
            Self::Binary { .. }
                | Self::Negate(_)
                | Self::AtTime { .. }
                | Self::Power { .. }
                | Self::UnitConversion { .. }
        )
    }

    /// Returns true if this expression contains a live time reference
    /// (e.g., "now", "current UTC time", "UTC time").
    /// Used to determine if the result should auto-refresh.
    #[must_use]
    pub fn contains_live_time(&self) -> bool {
        match self {
            Self::DateTime(dt) => dt.is_live_time(),
            Self::Now => true,
            Self::Until(inner) => inner.contains_live_time(),
            Self::Binary { left, right, .. } => {
                left.contains_live_time() || right.contains_live_time()
            }
            Self::Negate(inner) | Self::Group(inner) => inner.contains_live_time(),
            Self::AtTime { value, time } => value.contains_live_time() || time.contains_live_time(),
            Self::FunctionCall { args, .. } => args.iter().any(Self::contains_live_time),
            Self::Power { base, exponent } => {
                base.contains_live_time() || exponent.contains_live_time()
            }
            Self::UnitConversion { value, .. } => value.contains_live_time(),
            Self::Equality { left, right } => {
                left.contains_live_time() || right.contains_live_time()
            }
            Self::IndefiniteIntegral { integrand, .. } => integrand.contains_live_time(),
            Self::Number { .. } | Self::Variable(_) => false,
        }
    }

    /// Collects all currency codes referenced in this expression.
    ///
    /// Walks the AST and gathers every `Unit::Currency(code)` value, including
    /// target units in `UnitConversion` nodes. The returned set contains
    /// uppercase currency codes (e.g., "USD", "RUB", "TON").
    #[must_use]
    pub fn collect_currencies(&self) -> std::collections::HashSet<String> {
        let mut currencies = std::collections::HashSet::new();
        self.collect_currencies_inner(&mut currencies);
        currencies
    }

    fn collect_currencies_inner(&self, currencies: &mut std::collections::HashSet<String>) {
        match self {
            Self::Number { unit, .. } => {
                if let Unit::Currency(code) = unit {
                    currencies.insert(code.to_uppercase());
                }
            }
            Self::Binary { left, right, .. }
            | Self::Power {
                base: left,
                exponent: right,
            }
            | Self::Equality { left, right } => {
                left.collect_currencies_inner(currencies);
                right.collect_currencies_inner(currencies);
            }
            Self::Negate(inner) | Self::Group(inner) | Self::Until(inner) => {
                inner.collect_currencies_inner(currencies);
            }
            Self::AtTime { value, time } => {
                value.collect_currencies_inner(currencies);
                time.collect_currencies_inner(currencies);
            }
            Self::FunctionCall { args, .. } => {
                for arg in args {
                    arg.collect_currencies_inner(currencies);
                }
            }
            Self::IndefiniteIntegral { integrand, .. } => {
                integrand.collect_currencies_inner(currencies);
            }
            Self::UnitConversion { value, target_unit } => {
                value.collect_currencies_inner(currencies);
                if let Unit::Currency(code) = target_unit {
                    currencies.insert(code.to_uppercase());
                }
            }
            Self::DateTime(_) | Self::Now | Self::Variable(_) => {}
        }
    }

    /// Returns the depth of the expression tree.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Number { .. } | Self::DateTime(_) | Self::Variable(_) | Self::Now => 1,
            Self::Binary { left, right, .. }
            | Self::Power {
                base: left,
                exponent: right,
            } => 1 + left.depth().max(right.depth()),
            Self::Negate(inner) | Self::Group(inner) | Self::Until(inner) => 1 + inner.depth(),
            Self::AtTime { value, time } => 1 + value.depth().max(time.depth()),
            Self::FunctionCall { args, .. } => {
                1 + args.iter().map(Expression::depth).max().unwrap_or(0)
            }
            Self::IndefiniteIntegral { integrand, .. } => 1 + integrand.depth(),
            Self::UnitConversion { value, .. } => 1 + value.depth(),
            Self::Equality { left, right } => 1 + left.depth().max(right.depth()),
        }
    }

    /// Converts the expression to a LaTeX representation.
    #[must_use]
    pub fn to_latex(&self) -> String {
        match self {
            Self::Number { value, unit, .. } => {
                let num_str = value.to_string();
                if *unit == Unit::None {
                    num_str
                } else {
                    format!("{num_str} \\text{{{unit}}}")
                }
            }
            Self::DateTime(dt) => format!("\\text{{{dt}}}"),
            Self::Now => "\\text{now}".to_string(),
            Self::Until(inner) => {
                format!("\\text{{until }} {}", inner.to_latex())
            }
            Self::Binary { left, op, right } => {
                let left_str = left.to_latex();
                let right_str = right.to_latex();
                match op {
                    BinaryOp::Add => format!("{left_str} + {right_str}"),
                    BinaryOp::Subtract => format!("{left_str} - {right_str}"),
                    BinaryOp::Multiply => format!("{left_str} \\cdot {right_str}"),
                    BinaryOp::Divide => format!("\\frac{{{left_str}}}{{{right_str}}}"),
                }
            }
            Self::Negate(inner) => format!("-{}", inner.to_latex()),
            Self::Group(inner) => format!("\\left({} \\right)", inner.to_latex()),
            Self::AtTime { value, time } => {
                format!("{} \\text{{ at }} {}", value.to_latex(), time.to_latex())
            }
            Self::FunctionCall { name, args } => {
                let name_lower = name.to_lowercase();
                match name_lower.as_str() {
                    "sin" | "cos" | "tan" | "cot" | "sec" | "csc" | "sinh" | "cosh" | "tanh"
                    | "coth" | "sech" | "csch" | "arcsin" | "arccos" | "arctan" | "ln" | "log"
                    | "exp" => {
                        if args.len() == 1 {
                            format!("\\{name_lower}\\left({} \\right)", args[0].to_latex())
                        } else {
                            let args_str = args
                                .iter()
                                .map(Expression::to_latex)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("\\{name_lower}\\left({args_str} \\right)")
                        }
                    }
                    "sqrt" => {
                        if args.len() == 1 {
                            format!("\\sqrt{{{}}}", args[0].to_latex())
                        } else {
                            let args_str = args
                                .iter()
                                .map(Expression::to_latex)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("\\sqrt{{{args_str}}}")
                        }
                    }
                    "abs" => {
                        if args.len() == 1 {
                            format!("\\left| {} \\right|", args[0].to_latex())
                        } else {
                            let args_str = args
                                .iter()
                                .map(Expression::to_latex)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("\\left| {args_str} \\right|")
                        }
                    }
                    "pi" => "\\pi".to_string(),
                    "e" => "e".to_string(),
                    "integrate" => {
                        if args.len() == 4 {
                            format!(
                                "\\int_{{{}}}^{{{}}} {} \\, d{}",
                                args[2].to_latex(),
                                args[3].to_latex(),
                                args[0].to_latex(),
                                args[1].to_latex()
                            )
                        } else {
                            let args_str = args
                                .iter()
                                .map(Expression::to_latex)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("\\text{{integrate}}({args_str})")
                        }
                    }
                    _ => {
                        let args_str = args
                            .iter()
                            .map(Expression::to_latex)
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("\\text{{{name}}}({args_str})")
                    }
                }
            }
            Self::Variable(name) => name.clone(),
            Self::Power { base, exponent } => {
                let base_latex = base.to_latex();
                let exp_latex = exponent.to_latex();
                // Wrap base in braces if it's complex
                match base.as_ref() {
                    Self::Number { .. } | Self::Variable(_) => {
                        format!("{base_latex}^{{{exp_latex}}}")
                    }
                    _ => format!("\\left({base_latex}\\right)^{{{exp_latex}}}"),
                }
            }
            Self::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                format!("\\int {} \\, d{}", integrand.to_latex(), variable)
            }
            Self::UnitConversion { value, target_unit } => {
                format!("{} \\to \\text{{{target_unit}}}", value.to_latex())
            }
            Self::Equality { left, right } => {
                format!("{} = {}", left.to_latex(), right.to_latex())
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number { value, unit, .. } => {
                if *unit == Unit::None {
                    write!(f, "{value}")
                } else {
                    write!(f, "{value} {unit}")
                }
            }
            Self::DateTime(dt) => write!(f, "{dt}"),
            Self::Now => write!(f, "now"),
            Self::Until(inner) => write!(f, "until {inner}"),
            Self::Binary { left, op, right } => write!(f, "{left} {op} {right}"),
            Self::Negate(inner) => write!(f, "-{inner}"),
            Self::Group(inner) => write!(f, "({inner})"),
            Self::AtTime { value, time } => write!(f, "{value} at {time}"),
            Self::FunctionCall { name, args } => {
                let args_str = args
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{name}({args_str})")
            }
            Self::Variable(name) => write!(f, "{name}"),
            Self::Power { base, exponent } => write!(f, "{base}^{exponent}"),
            Self::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                write!(f, "integrate {integrand} d{variable}")
            }
            Self::UnitConversion { value, target_unit } => {
                write!(f, "{value} as {target_unit}")
            }
            Self::Equality { left, right } => write!(f, "{left} = {right}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_expression() {
        let expr = Expression::number(Decimal::new(42));
        assert_eq!(expr.to_string(), "42");
        assert_eq!(expr.to_lino(), "42");
    }

    #[test]
    fn test_currency_expression() {
        let expr = Expression::currency(Decimal::new(100), "USD");
        assert_eq!(expr.to_string(), "100 USD");
        assert_eq!(expr.to_lino(), "(100 USD)");
    }

    #[test]
    fn test_binary_expression() {
        let left = Expression::number(Decimal::new(2));
        let right = Expression::number(Decimal::new(3));
        let expr = Expression::binary(left, BinaryOp::Add, right);
        assert_eq!(expr.to_string(), "2 + 3");
        // Improved links notation with minimal parentheses
        assert_eq!(expr.to_lino(), "(2 + 3)");
    }

    #[test]
    fn test_complex_expression() {
        let usd = Expression::currency(Decimal::new(84), "USD");
        let eur = Expression::currency(Decimal::new(34), "EUR");
        let expr = Expression::binary(usd, BinaryOp::Subtract, eur);
        assert!(expr.to_lino().contains("84 USD"));
        assert!(expr.to_lino().contains("34 EUR"));
    }

    #[test]
    fn test_binary_op_precedence() {
        assert!(BinaryOp::Multiply.precedence() > BinaryOp::Add.precedence());
        assert_eq!(BinaryOp::Add.precedence(), BinaryOp::Subtract.precedence());
    }

    #[test]
    fn test_depth() {
        let simple = Expression::number(Decimal::new(1));
        assert_eq!(simple.depth(), 1);

        let binary = Expression::binary(
            Expression::number(Decimal::new(1)),
            BinaryOp::Add,
            Expression::number(Decimal::new(2)),
        );
        assert_eq!(binary.depth(), 2);
    }
}
