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
    Number { value: Decimal, unit: Unit },
    /// A literal datetime.
    DateTime(DateTime),
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
}

impl Expression {
    /// Creates a number expression.
    #[must_use]
    pub fn number(value: Decimal) -> Self {
        Self::Number {
            value,
            unit: Unit::None,
        }
    }

    /// Creates a number expression with a unit.
    #[must_use]
    pub fn number_with_unit(value: Decimal, unit: Unit) -> Self {
        Self::Number { value, unit }
    }

    /// Creates a currency expression.
    #[must_use]
    pub fn currency(amount: Decimal, code: &str) -> Self {
        Self::Number {
            value: amount,
            unit: Unit::currency(code),
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

    /// Converts the expression to links notation format.
    ///
    /// Links notation uses minimal parentheses, only adding them where
    /// necessary to clarify structure. The rules are:
    /// - Atomic values (numbers, variables) don't need parentheses
    /// - Binary operations are wrapped in a single set of parentheses
    /// - Explicit groups from the input are preserved
    #[must_use]
    pub fn to_lino(&self) -> String {
        self.to_lino_internal(None)
    }

    /// Internal helper for to_lino.
    /// `parent_op` is the parent operator's precedence (if any) to determine
    /// if we need parentheses for this subexpression.
    fn to_lino_internal(&self, _parent_op: Option<&BinaryOp>) -> String {
        match self {
            Self::Number { value, unit } => {
                let num_str = value.to_string();
                if *unit == Unit::None {
                    num_str
                } else {
                    format!("({num_str} {unit})")
                }
            }
            Self::DateTime(dt) => dt.to_string(),
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
                    name.clone()
                } else {
                    let args_str = args
                        .iter()
                        .map(|a| a.to_lino_internal(None))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{name}({args_str})")
                }
            }
            Self::Variable(name) => name.clone(),
            Self::Power { base, exponent } => {
                let base_str = base.to_lino_internal(None);
                let exp_str = exponent.to_lino_internal(None);
                // Add parens around base if it's a complex expression
                if base.needs_parens_for_power() {
                    format!("({base_str})^{exp_str}")
                } else {
                    format!("{base_str}^{exp_str}")
                }
            }
            Self::IndefiniteIntegral {
                integrand,
                variable,
            } => {
                let integrand_str = integrand.to_lino_internal(None);
                format!("integrate {integrand_str} d{variable}")
            }
        }
    }

    /// Returns true if this expression needs parentheses when used as a unary operand.
    fn needs_parens_for_unary(&self) -> bool {
        matches!(self, Self::Binary { .. } | Self::AtTime { .. })
    }

    /// Returns true if this expression needs parentheses when used as a power base.
    fn needs_parens_for_power(&self) -> bool {
        matches!(
            self,
            Self::Binary { .. } | Self::Negate(_) | Self::AtTime { .. } | Self::Power { .. }
        )
    }

    /// Returns the depth of the expression tree.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Number { .. } | Self::DateTime(_) | Self::Variable(_) => 1,
            Self::Binary { left, right, .. }
            | Self::Power {
                base: left,
                exponent: right,
            } => 1 + left.depth().max(right.depth()),
            Self::Negate(inner) | Self::Group(inner) => 1 + inner.depth(),
            Self::AtTime { value, time } => 1 + value.depth().max(time.depth()),
            Self::FunctionCall { args, .. } => {
                1 + args.iter().map(Expression::depth).max().unwrap_or(0)
            }
            Self::IndefiniteIntegral { integrand, .. } => 1 + integrand.depth(),
        }
    }

    /// Converts the expression to a LaTeX representation.
    #[must_use]
    pub fn to_latex(&self) -> String {
        match self {
            Self::Number { value, unit } => {
                let num_str = value.to_string();
                if *unit == Unit::None {
                    num_str
                } else {
                    format!("{num_str} \\text{{{unit}}}")
                }
            }
            Self::DateTime(dt) => format!("\\text{{{dt}}}"),
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
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number { value, unit } => {
                if *unit == Unit::None {
                    write!(f, "{value}")
                } else {
                    write!(f, "{value} {unit}")
                }
            }
            Self::DateTime(dt) => write!(f, "{dt}"),
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
