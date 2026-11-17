//! Test vectors for js/src/rust/ratio.js (issue #254): num-rational 0.4.2
//! `Ratio<BigInt>` over num-bigint 0.5.1, num-integer 0.1.47 and
//! num-traits 0.2.19, with the features the calculator enables.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin ratio > js/test/vectors/ratio.json
//!
//! Encoding: a bigint is a decimal string; an f64 is its 16-hex-digit bit
//! pattern; a `Ratio` is "numer/denom" (the raw fields, the denominator is
//! always written); a caught panic is "!" followed by the panic message;
//! `None` is `null`; booleans are 0/1.
//!
//! The calculator builds num-rational without its `num-bigint` feature, so
//! `Ratio<BigInt>` has no `ToPrimitive`. The `to_f64` / `to_i64` columns below
//! come from a verbatim copy of that feature's impl (and of the private
//! `ratio_to_f64` / `ldexp`), checked here against the real
//! `Ratio<i64>::to_f64` for every input that fits in i64.

mod bigint;
mod feature;
mod rational;
mod util;

use std::panic;

use serde_json::{json, Value};

use crate::bigint::{
    bigint_from_f64, bigint_int, bigint_parse, bigint_pow, bigint_to_f64, bigint_to_int,
    parse_errors,
};
use crate::rational::{
    ratio_binary, ratio_binary_int, ratio_new, ratio_pow, ratio_pow_u32, ratio_to_f64_rows,
    ratio_unary,
};
use crate::util::Rng;

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);

    let sections: Vec<(&str, Vec<Value>)> = vec![
        ("bigint_to_f64", bigint_to_f64(&mut rng)),
        ("bigint_from_f64", bigint_from_f64(&mut rng)),
        ("bigint_to_int", bigint_to_int(&mut rng)),
        ("bigint_parse", bigint_parse(&mut rng)),
        ("bigint_parse_errors", parse_errors()),
        ("bigint_pow", bigint_pow()),
        ("bigint_int", bigint_int(&mut rng)),
        ("ratio_new", ratio_new(&mut rng)),
        ("ratio_unary", ratio_unary(&mut rng)),
        ("ratio_binary", ratio_binary(&mut rng)),
        ("ratio_binary_int", ratio_binary_int(&mut rng)),
        ("ratio_pow", ratio_pow(&mut rng)),
        ("ratio_pow_u32", ratio_pow_u32()),
        ("ratio_to_f64", ratio_to_f64_rows(&mut rng)),
    ];

    // One entry per line keeps diffs of the generated file readable.
    let mut out = String::from("{\n");
    for (i, (name, rows)) in sections.iter().enumerate() {
        out.push_str(&format!("{}: [\n", json!(name)));
        for (j, row) in rows.iter().enumerate() {
            out.push_str(&serde_json::to_string(row).expect("json"));
            out.push_str(if j + 1 < rows.len() { ",\n" } else { "\n" });
        }
        out.push_str(if i + 1 < sections.len() {
            "],\n"
        } else {
            "]\n"
        });
    }
    out.push_str("}\n");
    print!("{out}");
}
