//! Test vectors for js/src/types/rational.js (issue #254): the calculator's
//! own `src/types/rational.rs` (and the `decimal.rs` it uses), included
//! verbatim.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin rational > js/test/vectors/rational.json
//!
//! Run it with `--release`: the shipped engine wraps u128 overflow in
//! `detect_repeating_decimal`. Encoding: a `Rational` is "numer/denom" (the
//! denominator always written); an f64 is its 16-hex-digit bit pattern; an
//! `Err(e)` is {"e": e}; a caught panic is "!" and the message; `None` is
//! `null`; an `Ordering` is -1/0/1.
#![allow(dead_code, clippy::all)]

#[path = "../../../../../src/types/decimal.rs"]
mod decimal;
#[path = "../../../../../src/types/rational.rs"]
mod rational;
/// `crate::types::Decimal`, as `rational.rs` imports it.
mod types {
    pub use crate::decimal::Decimal;
}

use std::panic;
use std::str::FromStr;

use num_bigint::BigInt;
use serde_json::{json, Value};
use rational::{Rational, RepeatingDecimal};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    /// A random integer with up to `digits` decimal digits, maybe negative.
    fn bigint(&mut self, digits: u64) -> BigInt {
        let len = 1 + self.below(digits);
        let mut s = String::new();
        if self.next() & 1 == 1 {
            s.push('-');
        }
        for _ in 0..len {
            s.push((b'0' + self.below(10) as u8) as char);
        }
        s.parse().unwrap()
    }
}

fn r(x: &Rational) -> String {
    format!("{}/{}", x.numer_bigint(), x.denom_bigint())
}

fn catch<T>(f: impl FnOnce() -> T + panic::UnwindSafe, g: impl FnOnce(T) -> Value) -> Value {
    match panic::catch_unwind(f) {
        Ok(v) => g(v),
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_default();
            json!(format!("!{msg}"))
        }
    }
}

fn rd(x: Option<RepeatingDecimal>) -> Value {
    match x {
        None => Value::Null,
        Some(d) => json!([
            d.to_vinculum_notation(),
            d.to_parenthesis_notation(),
            d.to_ellipsis_notation(),
            d.to_latex()
        ]),
    }
}

fn floats(rng: &mut Rng) -> Vec<f64> {
    let mut v = vec![
        0.0, -0.0, 1.0, -1.0, 0.5, 0.1, 0.2, 0.3, 1.0 / 3.0, 2.0 / 3.0, -0.75, 3.14159,
        std::f64::consts::PI, std::f64::consts::E, 1e-10, 1e-20, 1e10, 1e20, 1e30, 1e38, 1e39,
        1e300, f64::MAX, f64::MIN_POSITIVE, 5e-324, f64::NAN, f64::INFINITY, f64::NEG_INFINITY,
        123456.789, 0.999999999999, 1.0000000001, 2f64.sqrt(),
    ];
    for _ in 0..150 {
        let x = f64::from_bits(rng.next());
        if x.is_finite() {
            v.push(x);
        }
        let m = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
        let e = rng.below(40) as i32 - 20;
        v.push(m * 10f64.powi(e) * if rng.next() & 1 == 1 { -1.0 } else { 1.0 });
    }
    v
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(0x5eed_0254_7a71_0a11);

    let from_f64: Vec<Value> = floats(&mut rng)
        .into_iter()
        .map(|x| {
            let v = catch(move || Rational::from_f64(x), |v| json!(r(&v)));
            json!([format!("{:016x}", x.to_bits()), v])
        })
        .collect();

    let mut pairs: Vec<(BigInt, BigInt)> = vec![];
    for (n, d) in [
        (1, 3), (2, 3), (1, 6), (1, 7), (1, 4), (-1, 3), (22, 7), (1, 97), (1, 983), (5, 1),
        (0, 1), (-7, 12), (1, 1_000_000_007), (355, 113), (1, 81), (10, 4),
    ] {
        pairs.push((BigInt::from(n), BigInt::from(d)));
    }
    let ten = BigInt::from(10);
    for e in [28u32, 29, 30, 40, 60, 100] {
        pairs.push((BigInt::from(1), num_traits::Pow::pow(&ten, e)));
        pairs.push((BigInt::from(-3), num_traits::Pow::pow(&ten, e) * 7));
        pairs.push((num_traits::Pow::pow(&ten, e) + 1, BigInt::from(3)));
    }
    // Denominators near and above u128::MAX / 10 (wrapping long division).
    let big = BigInt::from(u128::MAX);
    pairs.push((BigInt::from(1), big.clone()));
    pairs.push((BigInt::from(2), &big / 10 + 1));
    pairs.push((BigInt::from(1), &big / 7));
    pairs.push((BigInt::from(1), &big + 2));
    for _ in 0..150 {
        let n = rng.bigint(40);
        let mut d = rng.bigint(40);
        if d == BigInt::from(0) {
            d = BigInt::from(1);
        }
        pairs.push((n, d));
    }

    let rationals: Vec<Rational> = pairs
        .iter()
        .map(|(n, d)| Rational::new_bigint(n.clone(), d.clone()))
        .collect();

    let unary: Vec<Value> = rationals
        .iter()
        .map(|x| {
            let pows: Vec<Value> = [-3, -1, 2, 3]
                .iter()
                .map(|&e| {
                    let x = x.clone();
                    catch(move || x.pow_i32(e), |v| json!(r(&v)))
                })
                .collect();
            json!({
                "x": r(x),
                "display": x.to_display_string(),
                "fraction": x.to_fraction_string(),
                "f64": format!("{:016x}", x.to_f64().to_bits()),
                "decimal": x.to_decimal().to_string(),
                "numer": x.numer().to_string(),
                "denom": x.denom().to_string(),
                "repeating": rd(x.to_repeating_decimal_notation()),
                "json": serde_json::to_value(x).unwrap(),
                "pow": pows,
            })
        })
        .collect();

    let mut binary = vec![];
    for i in 0..rationals.len() {
        let a = &rationals[i];
        let b = &rationals[(i * 7 + 3) % rationals.len()];
        binary.push(json!({
            "a": r(a),
            "b": r(b),
            "add": r(&(a.clone() + b.clone())),
            "sub": r(&(a.clone() - b.clone())),
            "mul": r(&(a.clone() * b.clone())),
            "div": a.checked_div(b).map(|v| r(&v)),
            "rem": a.remainder(b).map(|v| r(&v)),
            "cmp": a.cmp(b) as i8,
        }));
    }

    let inputs = [
        "1/3", " 2 / 6 ", "-4/8", "1/0", "a/2", "1/b", "1/", "/2", "0.5", "-0.125", "3",
        "1e5", "abc", "", "-", "1.5e-3", "170141183460469231731687303715884105728/1",
        "0.0000000000000000000000000001", "12345678901234567890.123", "1/2/3", "+3/4",
        "79228162514264337593543950336", "-.5", "5.",
    ];
    let from_str: Vec<Value> = inputs
        .iter()
        .map(|s| {
            let v = catch(
                || Rational::from_str(s),
                |res| match res {
                    Ok(v) => json!(r(&v)),
                    Err(e) => json!({ "e": e }),
                },
            );
            json!([s, v])
        })
        .collect();

    let sections: Vec<(&str, Vec<Value>)> = vec![
        ("from_f64", from_f64),
        ("unary", unary),
        ("binary", binary),
        ("from_str", from_str),
    ];
    println!("{{");
    for (i, (name, rows)) in sections.iter().enumerate() {
        println!("  {}: [", json!(name));
        for (j, row) in rows.iter().enumerate() {
            let comma = if j + 1 < rows.len() { "," } else { "" };
            println!("    {row}{comma}");
        }
        let comma = if i + 1 < sections.len() { "," } else { "" };
        println!("  ]{comma}");
    }
    println!("}}");
}
