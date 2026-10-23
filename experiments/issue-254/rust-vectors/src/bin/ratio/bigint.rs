//! num-bigint 0.5.1 sections.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Pow, ToPrimitive};
use serde_json::{json, Value};

use crate::util::{big, flag, hex, int_op, opt_str, p10, p2, Rng};

fn bigint_specials() -> Vec<BigInt> {
    let mut v = vec![
        big(0),
        big(1),
        big(2),
        big(3),
        big(255),
        big(1 << 52),
        p2(53) - 1,
        p2(53),
        p2(53) + 1,
        p2(53) + 2,
        p2(53) + 3,
        p2(54) + 1,
        p2(54) + 2,
        p2(54) + 3,
        p2(54) + 6,
        p2(63) - 1,
        p2(63),
        p2(64) - 1,
        p2(64),
        p2(64) + 1,
        p2(64) + p2(11),
        p2(64) + p2(11) + 1,
        p2(64) + p2(12) + p2(11),
        p2(100) + p2(47),
        p2(100) + p2(47) + 1,
        p2(100) + p2(48) + p2(47),
        p2(200) + p2(147),
        p2(200) + p2(147) + 1,
        p2(200) - 1,
        p2(1023),
        p2(1023) + 1,
        p2(1024) - p2(971),
        p2(1024) - p2(970) - 1,
        p2(1024) - p2(970),
        p2(1024) - 1,
        p2(1024),
        p2(1025),
        p2(2000),
        p10(15),
        p10(22),
        p10(23),
        p10(100),
        p10(308),
        p10(309),
        p10(400),
        p10(400) / big(3),
        big(123_456_789_012_345_678_901_234_567_890),
    ];
    let negs: Vec<BigInt> = v.iter().map(|n| -n).collect();
    v.extend(negs);
    v
}

pub fn bigint_to_f64(rng: &mut Rng) -> Vec<Value> {
    let mut inputs = bigint_specials();
    for _ in 0..600 {
        inputs.push(rng.shaped(1100));
    }
    for _ in 0..80 {
        let bits = 1018 + rng.below(12);
        let m = rng.magnitude(bits);
        inputs.push(if rng.coin() { -m } else { m });
    }
    inputs
        .iter()
        .map(|n| {
            json!([
                n.to_string(),
                hex(n.to_f64().expect("BigInt::to_f64 is total"))
            ])
        })
        .collect()
}

pub fn bigint_from_f64(rng: &mut Rng) -> Vec<Value> {
    let mut inputs: Vec<f64> = vec![
        0.0,
        -0.0,
        0.5,
        -0.5,
        0.9999,
        -0.9999,
        1.0,
        -1.0,
        1.5,
        -1.5,
        2.5,
        -2.5,
        123456.789,
        -123456.789,
        2251799813685248.5,
        -2251799813685248.5,
        4503599627370495.5,
        9007199254740992.0,
        9007199254740993.0,
        9.223372036854776e18,
        1.8446744073709552e19,
        1e300,
        -1e300,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::EPSILON,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        f64::from_bits(0xfff8_0000_0000_0000),
        f64::from_bits(0x7ff0_0000_0000_0001),
    ];
    for _ in 0..300 {
        inputs.push(f64::from_bits(rng.next_u64()));
    }
    for _ in 0..100 {
        // Moderate magnitudes, so most results are non-trivial.
        let e = rng.below(140) as i32 - 20;
        let m = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let x = (1.0 + m) * 2f64.powi(e);
        inputs.push(if rng.coin() { -x } else { x });
    }
    inputs
        .iter()
        .map(|&x| json!([hex(x), opt_str(BigInt::from_f64(x))]))
        .collect()
}

pub fn bigint_to_int(rng: &mut Rng) -> Vec<Value> {
    let mut inputs = vec![
        big(0),
        big(1),
        big(-1),
        big(i64::MAX as i128),
        big(i64::MAX as i128 + 1),
        big(i64::MIN as i128),
        big(i64::MIN as i128 - 1),
        big(u64::MAX as i128),
        big(u64::MAX as i128 + 1),
        -big(u64::MAX as i128),
        big(i128::MAX),
        big(i128::MAX) + 1,
        big(i128::MIN),
        big(i128::MIN) - 1,
        BigInt::from(u128::MAX),
        BigInt::from(u128::MAX) + 1,
        -BigInt::from(u128::MAX),
        p2(200),
        -p2(200),
    ];
    for _ in 0..150 {
        inputs.push(rng.shaped(140));
    }
    inputs
        .iter()
        .map(|n| {
            json!([
                n.to_string(),
                opt_str(n.to_i64()),
                opt_str(n.to_i128()),
                opt_str(n.to_u64()),
                opt_str(n.to_u128())
            ])
        })
        .collect()
}

pub fn bigint_parse(rng: &mut Rng) -> Vec<Value> {
    let mut inputs: Vec<String> = [
        "",
        "+",
        "-",
        "++",
        "--",
        "+-",
        "-+",
        "+-1",
        "-+1",
        "--1",
        "++1",
        "+1",
        "-1",
        "_",
        "_1",
        "-_1",
        "+_1",
        "-_",
        "1_",
        "1_000",
        "1__0",
        "1_000_000",
        "-1_0",
        " 1",
        "1 ",
        " ",
        "\t1",
        "1\n",
        "٣",
        "-٣",
        "+٣",
        "1٣",
        "１",
        "é",
        "∞",
        "+0",
        "-0",
        "0",
        "00",
        "0001",
        "-0001",
        "+0001",
        "-00",
        "12a",
        "a",
        "A",
        "z",
        "1e5",
        "1.0",
        "0x10",
        "1,000",
        "1+1",
        "1-1",
        "-1-",
        "0_",
        "+_",
        "-+_",
        "340282366920938463463374607431768211455",
        "340282366920938463463374607431768211456",
        "-170141183460469231731687303715884105729",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let hundred: String = (0..100)
        .map(|i| char::from(b'0' + ((i * 7 + 3) % 10) as u8))
        .collect();
    inputs.push(hundred.clone());
    inputs.push(format!("-{hundred}"));
    inputs.push(format!("+{hundred}"));
    inputs.push(format!("000{hundred}"));
    inputs.push(format!("{}_{}", &hundred[..50], &hundred[50..]));
    inputs.push(format!("{}x{}", &hundred[..50], &hundred[50..]));
    inputs.push("9".repeat(500));
    inputs.push(format!("-{}", "1".repeat(500)));
    for _ in 0..120 {
        let len = 1 + rng.below(120) as usize;
        let mut s = String::new();
        match rng.below(6) {
            0 => s.push('-'),
            1 => s.push('+'),
            _ => {}
        }
        for i in 0..len {
            if i > 0 && rng.below(25) == 0 {
                s.push('_');
            }
            s.push(char::from(b'0' + rng.below(10) as u8));
        }
        if rng.below(12) == 0 {
            let at = rng.below(s.len() as u64 + 1) as usize;
            let bad = [' ', '.', 'e', '-', '+', '/'][rng.below(6) as usize];
            s.insert(at, bad);
        }
        inputs.push(s);
    }
    inputs
        .iter()
        .map(|s| match s.parse::<BigInt>() {
            Ok(n) => json!([s, n.to_string(), null, null]),
            Err(e) => json!([s, null, format!("{e}"), format!("{e:?}")]),
        })
        .collect()
}

pub fn bigint_pow() -> Vec<Value> {
    let mut out = Vec::new();
    let bases = [
        big(-3),
        big(-2),
        big(-1),
        big(0),
        big(1),
        big(2),
        big(3),
        big(10),
        big(-10),
        big(7),
        big(123_456_789),
        big(-987_654_321),
        p2(64) + 1,
    ];
    for b in &bases {
        for e in [0u32, 1, 2, 3, 5, 8, 13, 21, 34] {
            out.push(json!([b.to_string(), e, Pow::pow(b, e).to_string()]));
        }
    }
    for (b, e) in [
        (big(10), 100u32),
        (big(10), 400),
        (big(2), 1000),
        (big(-2), 999),
        (big(-1), u32::MAX),
        (big(-1), u32::MAX - 1),
        (big(1), u32::MAX),
        (big(0), u32::MAX),
    ] {
        out.push(json!([b.to_string(), e, Pow::pow(&b, e).to_string()]));
    }
    // `Pow<u64>`, as `Rational::to_scientific_string` uses it.
    for e in [0u64, 1, 17, 60] {
        out.push(json!(["10", e, Pow::pow(&big(10), e).to_string()]));
    }
    out
}

pub fn bigint_int(rng: &mut Rng) -> Vec<Value> {
    let mut pairs: Vec<(BigInt, BigInt)> = [
        (0, 0),
        (0, 5),
        (5, 0),
        (-5, 0),
        (0, -5),
        (7, 2),
        (-7, 2),
        (7, -2),
        (-7, -2),
        (6, 3),
        (-6, 3),
        (6, -3),
        (12, 18),
        (-12, 18),
        (12, -18),
        (-12, -18),
        (1, 1),
        (-1, -1),
    ]
    .iter()
    .map(|&(a, b)| (big(a), big(b)))
    .collect();
    pairs.push((p2(100), p2(60)));
    pairs.push((-p2(100) - 1, p2(60)));
    for _ in 0..300 {
        pairs.push((rng.shaped(200), rng.shaped(200)));
    }
    pairs
        .iter()
        .map(|(a, b)| {
            json!([
                a.to_string(),
                b.to_string(),
                a.gcd(b).to_string(),
                a.lcm(b).to_string(),
                int_op(|| a.div_floor(b)),
                int_op(|| a.mod_floor(b)),
                int_op(|| a.div_rem(b).0),
                int_op(|| a.div_rem(b).1),
                int_op(|| a / b),
                int_op(|| a % b),
                flag(a.is_even())
            ])
        })
        .collect()
}

pub fn parse_errors() -> Vec<Value> {
    let empty = "".parse::<BigInt>().unwrap_err();
    let invalid = "x".parse::<BigInt>().unwrap_err();
    vec![
        json!(["Empty", format!("{empty}"), format!("{empty:?}")]),
        json!(["InvalidDigit", format!("{invalid}"), format!("{invalid:?}")]),
    ]
}
