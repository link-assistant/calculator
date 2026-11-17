//! Test vectors for js/src/grammar/math_functions.js (issue #254): the
//! calculator's own `src/grammar/math_functions.rs` (and the `decimal.rs` and
//! `rational.rs` it uses), included verbatim. The rest of the crate it
//! imports (`CalculatorError`, `Value`, the dimensional functions) is
//! replaced by the minimal stand-ins below; only `evaluate_function` and
//! `integrate`, which do not reach them, are exercised.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin math_functions > js/test/vectors/math_functions.json
//!
//! Encoding: a `Decimal` result is its `Display`; an f64 is its 16-hex-digit
//! bit pattern; an `Err(e)` is {"e": "<variant>: <payload>"}.
#![allow(dead_code, clippy::all)]

#[path = "../../../../../src/types/decimal.rs"]
mod decimal;
#[path = "../../../../../src/types/rational.rs"]
mod rational;

/// Stand-in for `crate::error::CalculatorError` (only the constructors
/// `math_functions.rs` calls).
mod error {
    #[derive(Debug)]
    pub enum CalculatorError {
        Overflow,
        DomainError(String),
        UnknownFunction(String),
        InvalidFunctionArgs { function: String, reason: String },
    }

    impl CalculatorError {
        pub fn domain(msg: impl Into<String>) -> Self {
            Self::DomainError(msg.into())
        }
        pub fn unknown_function(name: &str) -> Self {
            Self::UnknownFunction(name.to_string())
        }
        pub fn invalid_args(function: &str, reason: impl Into<String>) -> Self {
            Self::InvalidFunctionArgs {
                function: function.to_string(),
                reason: reason.into(),
            }
        }
    }
}

/// Stand-in for `crate::types`: the real `Decimal` and `Rational`, and a
/// `Value` with the methods `evaluate_function_values` names.
mod types {
    pub use crate::decimal::Decimal;
    pub use crate::rational::Rational;
    use crate::error::CalculatorError;

    pub struct Unit;
    impl Unit {
        pub fn is_opaque_compound(&self) -> bool {
            unreachable!()
        }
    }

    pub struct Value {
        pub unit: Unit,
    }
    impl Value {
        pub fn number(_: Decimal) -> Self {
            unreachable!()
        }
        pub fn rational(_: Rational) -> Self {
            unreachable!()
        }
        pub fn as_decimal(&self) -> Option<Decimal> {
            unreachable!()
        }
        pub fn to_rational(&self) -> Option<Rational> {
            unreachable!()
        }
        pub fn with_uncertainty(&self, _: &Value) -> Result<Value, CalculatorError> {
            unreachable!()
        }
    }

    pub fn evaluate_ratio_function(
        _: &str,
        _: &[Value],
    ) -> Option<Result<Value, CalculatorError>> {
        unreachable!()
    }
}

#[path = "../../../../../src/grammar/math_functions.rs"]
mod math_functions;

/// Stand-in for the sibling `super::dimensional_functions`.
mod dimensional_functions {
    use crate::error::CalculatorError;
    use crate::types::Value;

    pub fn dimensional_function(_: &str, _: &[Value]) -> Option<Result<Value, CalculatorError>> {
        unreachable!()
    }
}

use std::str::FromStr;

use decimal::Decimal;
use error::CalculatorError;
use math_functions::{evaluate_function, integrate};
use serde_json::{json, Value};

fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

fn encode(result: Result<Decimal, CalculatorError>) -> Value {
    match result {
        Ok(d) => json!(d.to_string()),
        Err(CalculatorError::Overflow) => json!({ "e": "Overflow" }),
        Err(CalculatorError::DomainError(m)) => json!({ "e": format!("DomainError: {m}") }),
        Err(CalculatorError::UnknownFunction(m)) => {
            json!({ "e": format!("UnknownFunction: {m}") })
        }
        Err(CalculatorError::InvalidFunctionArgs { function, reason }) => {
            json!({ "e": format!("InvalidFunctionArgs: {function}: {reason}") })
        }
    }
}

const NAMES: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "exp", "ln", "log",
    "log2", "log10", "sqrt", "cbrt", "abs", "floor", "ceil", "round", "trunc", "sign", "factorial",
    "deg", "rad", "pi", "Sin",
];
const BINARY: &[&str] = &["atan2", "log", "pow", "min", "max", "mod"];
const ARGS: &[&str] = &[
    "0", "1", "-1", "0.5", "-0.5", "2", "2.5", "-2.5", "3", "5", "10", "0.1", "1.5707963267948966",
    "3.141592653589793", "-7.25", "12.3456789", "100", "170", "171", "1e-20", "70", "-70",
    "123456789.987654321", "1000", "0.0000001",
];

fn main() {
    let args: Vec<Decimal> = ARGS
        .iter()
        .map(|s| Decimal::from_str(s).or_else(|_| Decimal::from_scientific(s)).unwrap())
        .collect();
    let mut functions = Vec::new();
    for name in NAMES {
        functions.push(json!([name, [], encode(evaluate_function(name, &[]))]));
        for (s, a) in ARGS.iter().zip(&args) {
            let r = evaluate_function(name, &[a.clone()]);
            functions.push(json!([name, [s], encode(r)]));
        }
    }
    for name in BINARY {
        for (s, a) in ARGS.iter().zip(&args).step_by(2) {
            for (t, b) in ARGS.iter().zip(&args).step_by(3) {
                let r = evaluate_function(name, &[a.clone(), b.clone()]);
                functions.push(json!([name, [s, t], encode(r)]));
            }
        }
    }
    functions.push(json!(["min", ["3", "1", "2"], encode(evaluate_function("min", &[args[8].clone(), args[1].clone(), args[5].clone()]))]));

    let bounds: &[(f64, f64)] = &[(0.0, 1.0), (0.0, std::f64::consts::PI), (-2.5, 7.25), (1.0, 0.1)];
    let fs: &[(&str, fn(f64) -> f64)] = &[
        ("one", |_| 1.0),
        ("x", |x| x),
        ("x2", |x| x * x),
        ("sin", libm::sin),
        ("exp", libm::exp),
        ("sin_over_x", |x| libm::sin(x) / x),
    ];
    let mut integrals = Vec::new();
    for (name, f) in fs {
        for (a, b) in bounds {
            integrals.push(json!([name, bits(*a), bits(*b), bits(integrate(f, *a, *b))]));
        }
    }
    println!("{}", json!({ "functions": functions, "integrals": integrals }));
}
