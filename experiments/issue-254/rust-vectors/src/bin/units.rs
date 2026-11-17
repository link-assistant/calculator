//! Test vectors for js/src/types/{unit,unit_db,physical}.js (issue #254): the
//! calculator's own `src/types/{unit,unit_db,physical}.rs` (and the
//! `dimension.rs`, `rational.rs` and `decimal.rs` they use), included verbatim.
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin units > js/test/vectors/units.json
//!
//! Encoding: a `Rational` is "numer/denom"; an f64 is its 16-hex-digit bit
//! pattern; `None` is `null`; a `Dimension` is its `Display`.
#![allow(dead_code, clippy::all)]

#[path = "../../../../../src/types/decimal.rs"]
mod decimal;
#[path = "../../../../../src/types/dimension.rs"]
mod dimension;
#[path = "../../../../../src/types/physical.rs"]
mod physical;
#[path = "../../../../../src/types/rational.rs"]
mod rational;
#[path = "../../../../../src/types/unit.rs"]
mod unit;
#[path = "../../../../../src/types/unit_db.rs"]
mod unit_db;

// The sources import their siblings through `super::` / `crate::types::`.
use decimal::Decimal;
use dimension::Dimension;
use physical::PhysicalUnit;
use rational::Rational;
use unit::{DataSizeUnit, DurationUnit, MassUnit, Unit};
mod types {
    pub use crate::decimal::Decimal;
}

use serde_json::{json, Map, Value};
use unit_db::{lookup_symbol, lookup_unit, named_unit_for};

const UNITS_LINO: &str = include_str!("../../../../../data/units/units.lino");

const SI: [&str; 27] = [
    "Q", "R", "Y", "Z", "E", "P", "T", "G", "M", "k", "h", "da", "d", "c", "m", "µ", "u", "μ", "n",
    "p", "f", "a", "z", "y", "r", "q", "Ki",
];
const SI_WORDS: [&str; 26] = [
    "quetta", "ronna", "yotta", "zetta", "exa", "peta", "tera", "giga", "mega", "kilo", "hecto",
    "deca", "deci", "centi", "milli", "micro", "nano", "pico", "femto", "atto", "zepto", "yocto",
    "ronto", "quecto", "kibi", "mebi",
];

fn r(x: &Rational) -> String {
    format!("{}/{}", x.numer_bigint(), x.denom_bigint())
}

fn f(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

/// Every quoted name of the table.
fn names() -> Vec<String> {
    let mut names = Vec::new();
    for line in UNITS_LINO.lines() {
        let line = line.trim();
        if !line.starts_with("(unit") && !line.starts_with("(affine") {
            continue;
        }
        let parts: Vec<&str> = line.split('\'').collect();
        // Quoted strings are the odd parts; the first is the definition.
        for part in parts.iter().skip(3).step_by(2) {
            names.push((*part).to_string());
        }
    }
    names
}

fn main() {
    let base = names();
    let mut inputs: Vec<String> = Vec::new();
    for name in &base {
        inputs.push(name.clone());
        inputs.push(name.to_uppercase());
        inputs.push(name.to_lowercase());
        let chars: Vec<char> = name.chars().collect();
        if chars.len() <= 3 {
            for prefix in SI {
                inputs.push(format!("{prefix}{name}"));
            }
        } else {
            for word in SI_WORDS {
                inputs.push(format!("{word}{name}"));
            }
        }
    }
    inputs.extend(
        ["am", "as", "pm", "ft2", "CUP", "kilo", "1", "", "x", "Gis", "Mibit", "°F", "°C"]
            .iter()
            .map(|s| s.to_string()),
    );
    let mut seen = std::collections::HashSet::new();
    inputs.retain(|s| seen.insert(s.clone()));

    let mut lookups = Map::new();
    for name in &inputs {
        let value = match lookup_unit(name) {
            None => Value::Null,
            Some(u) => json!([
                u.symbol,
                r(&u.factor),
                f(u.factor.to_f64()),
                u.dimension().to_string(),
                u.offset().map(r),
                named_unit_for(&u.dimension()),
                lookup_symbol(&u.symbol).map(|s| s.symbol),
            ]),
        };
        lookups.insert(name.clone(), value);
    }

    // Products of two or three units with powers, simplified.
    let sample: Vec<&str> = vec![
        "m", "cm", "km", "ft", "mi", "h", "s", "min", "mph", "kg", "lb", "g", "N", "J", "W", "kW",
        "V", "Ω", "A", "mA", "kΩ", "Hz", "°C", "K", "L", "mL", "B", "KiB", "bit", "Pa", "day",
    ];
    let mut products = Vec::new();
    for (i, a) in sample.iter().enumerate() {
        for (j, b) in sample.iter().enumerate() {
            for pa in [1, 2] {
                for pb in [-1] {
                    let ua = PhysicalUnit::parse(a).unwrap().powi(pa);
                    let ub = PhysicalUnit::parse(b).unwrap().powi(pb);
                    let mut product = ua.product(&ub);
                    if (i + j) % 5 == 0 {
                        let c = sample[(i * 7 + j) % sample.len()];
                        product = product.product(&PhysicalUnit::parse(c).unwrap().powi(-1));
                    }
                    let (unit, scale) = product.simplify();
                    products.push(json!([
                        product
                            .terms()
                            .iter()
                            .map(|t| json!([t.symbol, t.power]))
                            .collect::<Vec<_>>(),
                        product.to_string(),
                        product.dimension().to_string(),
                        r(&product.factor()),
                        product.has_affine(),
                        product.affine_offset().map(|o| r(&o)),
                        product.root(2).map(|u| u.to_string()),
                        unit.to_string(),
                        r(&scale),
                        f(scale.to_f64()),
                        unit.clone().into_unit().to_string(),
                    ]));
                }
            }
        }
    }

    let sizes = [
        "bit", "Kb", "Mb", "Gb", "Tb", "Pb", "Kib", "Mib", "Gib", "Tib", "Pib", "B", "KB", "MB",
        "GB", "TB", "PB", "KiB", "MiB", "GiB", "TiB", "PiB",
    ];
    let values = [1.0, 741.0, 0.1, 1e300, 3.5e-7, 123456789.123];
    let mut data_size = Vec::new();
    for a in sizes {
        for b in sizes {
            let ua = DataSizeUnit::parse(a).unwrap();
            let ub = DataSizeUnit::parse(b).unwrap();
            for v in values {
                data_size.push(f(ua.convert(v, ub)));
            }
        }
    }
    let masses = ["mg", "g", "kg", "t", "lb", "oz"];
    let mut mass = Vec::new();
    for a in masses {
        for b in masses {
            let ua = MassUnit::parse(a).unwrap();
            let ub = MassUnit::parse(b).unwrap();
            for v in values {
                mass.push(f(ua.convert(v, ub)));
            }
        }
    }

    let out = json!({
        "values": values.iter().map(|v| f(*v)).collect::<Vec<_>>(),
        "lookups": lookups,
        "sample": sample,
        "products": products,
        "sizes": sizes,
        "data_size": data_size,
        "masses": masses,
        "mass": mass,
    });
    println!("{}", serde_json::to_string(&out).unwrap());
}
