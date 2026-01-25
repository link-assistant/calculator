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

    /// Converts the expression to links notation format.
    #[must_use]
    pub fn to_lino(&self) -> String {
        self.to_lino_internal(false)
    }

    fn to_lino_internal(&self, in_binary: bool) -> String {
        match self {
            Self::Number { value, unit } => {
                let num_str = value.to_string();
                if *unit == Unit::None {
                    num_str
                } else {
                    format!("({num_str} {unit})")
                }
            }
            Self::DateTime(dt) => format!("({})", dt),
            Self::Binary { left, op, right } => {
                let left_str = left.to_lino_internal(true);
                let right_str = right.to_lino_internal(true);
                // in_binary is used by recursive calls to determine context
                let _ = in_binary;
                format!("(({left_str}) {op} ({right_str}))")
            }
            Self::Negate(inner) => {
                format!("(- ({}))", inner.to_lino_internal(false))
            }
            Self::Group(inner) => {
                format!("({})", inner.to_lino_internal(false))
            }
            Self::AtTime { value, time } => {
                format!(
                    "(({}) at ({}))",
                    value.to_lino_internal(false),
                    time.to_lino_internal(false)
                )
            }
            Self::FunctionCall { name, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_lino_internal(false))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({name} ({args_str}))")
            }
            Self::Variable(name) => name.clone(),
            Self::Power { base, exponent } => {
                format!(
                    "(({})^({}))",
                    base.to_lino_internal(false),
                    exponent.to_lino_internal(false)
                )
            }
        }
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
        assert_eq!(expr.to_lino(), "((2) + (3))");
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
