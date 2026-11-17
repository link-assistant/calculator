//! num-rational 0.4.2 sections.

use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{One, Pow, Signed, ToPrimitive, Zero};
use serde_json::{json, Value};

use crate::feature::feature_to_f64;
use crate::util::{big, catch, flag, int_op, op, opt_hex, ord, p10, p2, raw, rawi, rs, Rng, R};

pub fn ratio_new(rng: &mut Rng) -> Vec<Value> {
    let mut pairs: Vec<(BigInt, BigInt)> = [
        (0, 0),
        (5, 0),
        (-5, 0),
        (0, 5),
        (0, -5),
        (5, 5),
        (-5, -5),
        (5, -5),
        (6, -4),
        (-6, -4),
        (6, 4),
        (1, -1),
        (-1, 1),
        (1, 1),
        (10, 1),
        (1, 10),
    ]
    .iter()
    .map(|&(a, b)| (big(a), big(b)))
    .collect();
    pairs.push((p10(40) * 6, p10(39) * -4));
    for _ in 0..300 {
        let d = if rng.below(50) == 0 {
            BigInt::zero()
        } else {
            rng.nonzero(160)
        };
        let k = rng.shaped(40);
        let k = if k.is_zero() { BigInt::one() } else { k };
        pairs.push((rng.shaped(160) * &k, d * &k));
    }
    pairs
        .iter()
        .map(|(n, d)| {
            json!([
                n.to_string(),
                d.to_string(),
                op(|| Ratio::new(n.clone(), d.clone()))
            ])
        })
        .collect()
}

fn ratio_specials() -> Vec<R> {
    let mut v: Vec<R> = Vec::new();
    for n in -3..=3 {
        v.push(rawi(n, 1));
    }
    for &(n, d) in &[
        (1, 2),
        (3, 2),
        (5, 2),
        (7, 2),
        (1, 3),
        (2, 3),
        (4, 3),
        (5, 3),
        (7, 4),
        (9, 4),
        (10, 7),
        (2, 7),
        (999, 1000),
        (1001, 1000),
    ] {
        v.push(rawi(n, d));
        v.push(rawi(-n, d));
    }
    // Raw (unreduced, negative or zero denominator) values.
    for &(n, d) in &[
        (2, 4),
        (-2, 4),
        (2, -4),
        (-2, -4),
        (3, -2),
        (-3, -2),
        (5, -1),
        (0, -1),
        (0, 5),
        (1, -3),
        (6, 3),
        (-6, -3),
        (4, 4),
        (-4, 4),
        (0, 0),
        (5, 0),
        (-5, 0),
    ] {
        v.push(rawi(n, d));
    }
    v.push(raw(p10(400), big(3)));
    v.push(raw(-p10(400), big(3)));
    v.push(raw(big(3), p10(400)));
    v.push(raw(p2(64) + 1, big(2)));
    v.push(raw(-p2(64) - 1, big(2)));
    v.push(raw(big(i64::MAX as i128), big(1)));
    v.push(raw(big(i64::MIN as i128), big(1)));
    v.push(raw(big(i64::MIN as i128) - 1, big(1)));
    v.push(raw(BigInt::from(u64::MAX), big(1)));
    v.push(raw(BigInt::from(u64::MAX) + 1, big(1)));
    v.push(raw(BigInt::from(u128::MAX) * 2 + 1, big(2)));
    v.push(raw(BigInt::from(u128::MAX) + 1, big(1)));
    v.push(raw(big(i128::MIN) * 2 - 1, big(2)));
    v.push(raw(big(i128::MIN) - 1, big(1)));
    v
}

pub fn ratio_unary(rng: &mut Rng) -> Vec<Value> {
    let mut inputs = ratio_specials();
    for _ in 0..650 {
        inputs.push(rng.ratio(150));
    }
    inputs
        .iter()
        .map(|r| {
            let to_int = |f: fn(&BigInt) -> Option<String>| match catch(|| r.to_integer()) {
                Ok(n) => match f(&n) {
                    Some(s) => json!(s),
                    None => Value::Null,
                },
                Err(m) => json!(format!("!{m}")),
            };
            json!([
                rs(r),
                op(|| -r.clone()),
                op(|| r.abs()),
                op(|| r.signum()),
                op(|| r.recip()),
                op(|| r.floor()),
                op(|| r.ceil()),
                op(|| r.round()),
                op(|| r.trunc()),
                op(|| r.fract()),
                int_op(|| r.to_integer()),
                flag(r.is_integer()),
                flag(r.is_zero()),
                flag(r.is_positive()),
                flag(r.is_negative()),
                flag(r.is_one()),
                r.to_string(),
                format!("{r:?}"),
                opt_hex(feature_to_f64(r)),
                to_int(|n| n.to_i64().map(|v| v.to_string())),
                to_int(|n| n.to_u64().map(|v| v.to_string())),
                to_int(|n| n.to_i128().map(|v| v.to_string())),
                to_int(|n| n.to_u128().map(|v| v.to_string()))
            ])
        })
        .collect()
}

fn binary_row(a: &R, b: &R) -> Value {
    json!([
        rs(a),
        rs(b),
        op(|| a.clone() + b.clone()),
        op(|| a.clone() - b.clone()),
        op(|| a.clone() * b.clone()),
        op(|| a.clone() / b.clone()),
        op(|| a.clone() % b.clone()),
        match catch(|| a.cmp(b)) {
            Ok(o) => ord(o),
            Err(m) => json!(format!("!{m}")),
        },
        match catch(|| a == b) {
            Ok(e) => flag(e),
            Err(m) => json!(format!("!{m}")),
        }
    ])
}

pub fn ratio_binary(rng: &mut Rng) -> Vec<Value> {
    let set: Vec<R> = vec![
        rawi(0, 1),
        rawi(1, 1),
        rawi(-1, 1),
        rawi(1, 2),
        rawi(-1, 2),
        rawi(1, 3),
        rawi(-2, 7),
        rawi(3, 2),
        rawi(2, 4),
        rawi(1, -2),
        rawi(-1, -2),
        rawi(0, -3),
        rawi(5, 0),
        raw(p10(30), big(7)),
        raw(-p10(30), big(7)),
    ];
    let mut out = Vec::new();
    for a in &set {
        for b in &set {
            out.push(binary_row(a, b));
        }
    }
    for _ in 0..1100 {
        let a = rng.ratio(130);
        let b = if rng.below(10) == 0 {
            // Equal values with different representations, and shared parts.
            match rng.below(3) {
                0 => raw(a.numer() * 3, a.denom() * 3),
                1 => raw(a.numer().clone(), rng.nonzero(130)),
                _ => raw(rng.shaped(130), a.denom().clone()),
            }
        } else {
            rng.ratio(130)
        };
        out.push(binary_row(&a, &b));
    }
    out
}

pub fn ratio_binary_int(rng: &mut Rng) -> Vec<Value> {
    let set = ratio_specials();
    let ints = [big(0), big(1), big(-1), big(2), big(-3), p10(20)];
    let mut pairs: Vec<(R, BigInt)> = Vec::new();
    for a in set.iter().step_by(2) {
        for t in &ints {
            pairs.push((a.clone(), t.clone()));
        }
    }
    for _ in 0..300 {
        pairs.push((rng.ratio(130), rng.shaped(130)));
    }
    pairs
        .iter()
        .map(|(a, t)| {
            json!([
                rs(a),
                t.to_string(),
                op(|| a.clone() + t.clone()),
                op(|| a.clone() - t.clone()),
                op(|| a.clone() * t.clone()),
                op(|| a.clone() / t.clone()),
                op(|| a.clone() % t.clone())
            ])
        })
        .collect()
}

pub fn ratio_pow(rng: &mut Rng) -> Vec<Value> {
    let mut bases: Vec<R> = vec![
        rawi(0, 1),
        rawi(1, 1),
        rawi(-1, 1),
        rawi(2, 1),
        rawi(-2, 1),
        rawi(1, 2),
        rawi(-1, 2),
        rawi(2, 3),
        rawi(-2, 3),
        rawi(10, 1),
        rawi(1, 10),
        rawi(-7, 5),
        rawi(2, 4),
        rawi(1, -2),
        rawi(-1, -2),
        rawi(0, -1),
        rawi(-3, -1),
    ];
    for _ in 0..13 {
        bases.push(rng.ratio(40));
    }
    let mut out = Vec::new();
    for b in &bases {
        for e in -7i32..=7 {
            out.push(json!([rs(b), e, op(|| Pow::pow(b, e))]));
        }
    }
    for (b, e) in [
        (rawi(1, 1), i32::MAX),
        (rawi(1, 1), i32::MIN),
        (rawi(-1, 1), i32::MAX),
        (rawi(-1, 1), i32::MIN),
        (rawi(0, 1), i32::MIN),
        (rawi(0, 1), i32::MAX),
        (rawi(2, 1), 100),
        (rawi(1, 10), -50),
        (rawi(-2, 3), 31),
        (rawi(-2, 3), -31),
    ] {
        out.push(json!([rs(&b), e, op(|| Pow::pow(&b, e))]));
    }
    out
}

pub fn ratio_pow_u32() -> Vec<Value> {
    let mut out = Vec::new();
    for b in [
        rawi(0, 1),
        rawi(-2, 3),
        rawi(2, -4),
        rawi(10, 1),
        rawi(-1, 1),
    ] {
        for e in [0u32, 1, 2, 3, 10, 33] {
            out.push(json!([rs(&b), e, op(|| Pow::pow(&b, e))]));
        }
    }
    out
}

pub fn ratio_to_f64_rows(rng: &mut Rng) -> Vec<Value> {
    let mut inputs: Vec<R> = vec![
        rawi(1, 3),
        rawi(-2, 7),
        rawi(0, 1),
        rawi(0, -1),
        rawi(1, 0),
        rawi(-1, 0),
        rawi(0, 0),
        rawi(1, -3),
        rawi(-1, -3),
        raw(p10(400), big(3)),
        raw(-p10(400), big(3)),
        raw(big(3), p10(400)),
        raw(big(1), p10(308)),
        raw(big(1), p10(320)),
        raw(big(1), p10(323)),
        raw(big(1), p10(324)),
        raw(big(-1), p10(330)),
        raw(big(1), p2(1074)),
        raw(big(1), p2(1075)),
        raw(big(1), p2(1075) - 1),
        raw(big(3), p2(1076)),
        raw(big(1), p2(1076)),
        raw(big(3), p2(1075)),
        raw(big(1), p2(1022)),
        raw(p2(52) - 1, p2(1074)),
        raw(p2(53) - 1, p2(1075)),
        raw(p2(1024) - p2(970), big(1)),
        raw(p2(1024) - p2(970) - 1, big(1)),
        raw(p2(1025) - p2(971), big(2)),
        raw(p2(1024) - p2(971), big(1)),
        raw(p2(1024), big(1)),
        raw(p2(1024), big(2)),
        raw(p2(1025), big(3)),
        raw(p2(54) + 2, big(2)),
        raw(p2(54) + 6, big(2)),
        raw(p2(53) + 1, big(1)),
        raw(p2(53) + 1, big(3)),
        raw(p2(60) + 1, p2(60) - 1),
        raw(big(1), p2(60) + 1),
        raw(p10(30), p10(29) * 3),
        raw(p10(200) + 1, p10(200)),
        raw(p10(200), p10(200) + 1),
        raw(-p10(250), p10(200) * 7),
        raw(p10(600), p10(300)),
        raw(p10(300), p10(600)),
        rawi(i64::MIN as i128, 1),
        rawi(i64::MIN as i128, -1),
        rawi(i64::MAX as i128, i64::MIN as i128),
        rawi(i64::MIN as i128, i64::MIN as i128),
        rawi(1, i64::MIN as i128),
        rawi(i64::MAX as i128, 3),
    ];
    for _ in 0..250 {
        let nb = rng.below(1200);
        let db = rng.below(1200);
        let n = rng.magnitude(nb);
        let d = rng.magnitude(db.max(1));
        inputs.push(raw(if rng.coin() { -n } else { n }, d));
    }
    for _ in 0..120 {
        // Bit-length differences close to overflow, to the subnormal range
        // and to the underflow cut-off.
        let small = 1 + rng.below(80);
        let large = small
            + match rng.below(3) {
                0 => 1021 + rng.below(6),
                1 => 1020 + rng.below(56),
                _ => 1072 + rng.below(6),
            };
        let (a, b) = (rng.magnitude(large), rng.magnitude(small));
        let (n, d) = if rng.coin() { (a, b) } else { (b, a) };
        inputs.push(raw(if rng.coin() { -n } else { n }, d));
    }
    // i64-range inputs: the fast path and the i128 path of `ratio_to_f64`.
    let mut small: Vec<(i64, i64)> = Vec::new();
    for _ in 0..350 {
        let nb = rng.below(64) as u32;
        let db = rng.below(64) as u32;
        let n = (rng.next_u64() >> (63 - nb)) as i64;
        let d = (rng.next_u64() >> (63 - db)) as i64;
        let n = if rng.coin() { n.wrapping_neg() } else { n };
        let d = if rng.below(4) == 0 {
            d.wrapping_neg()
        } else {
            d
        };
        small.push((n, d));
    }
    for &(n, d) in &small {
        inputs.push(rawi(n as i128, d as i128));
    }
    let mut out = Vec::new();
    for r in &inputs {
        let got = feature_to_f64(r);
        if let (Some(n), Some(d)) = (r.numer().to_i64(), r.denom().to_i64()) {
            // The copy must agree with the real crate wherever both exist.
            let real = Ratio::<i64>::new_raw(n, d).to_f64();
            assert_eq!(got.map(f64::to_bits), real.map(f64::to_bits), "{n}/{d}");
        }
        out.push(json!([rs(r), opt_hex(got)]));
    }
    out
}
