//! Test vectors for js/src/rust/decimal.js (issue #254): rust_decimal 1.43.0
//! with its default features (the calculator enables `serde`).
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --bin decimal > js/test/vectors/decimal.json
//!
//! Encoding: a `Decimal` is "[-]mantissa:scale" from the raw parts (the "-" is
//! the sign flag, so "-0:2" is a negative zero); an f64 is its 16-hex-digit
//! bit pattern; an `Err(e)` is {"e": Display, "d": Debug}; a caught panic is
//! "!" followed by the panic message; `None` is `null`; an `Ordering` is
//! -1/0/1. For every binary operation the generator asserts that `checked_*`
//! is `None` exactly when the operator panics, so one column covers both.

use std::panic::{self, AssertUnwindSafe};
use std::str::FromStr;

use num_traits::ToPrimitive;
use rust_decimal::{Decimal, Error};
use serde_json::{json, Value};

const MAX_MANTISSA: u128 = (1u128 << 96) - 1;

struct Rng(u64);

impl Rng {
    /// splitmix64.
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
    fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
    /// A random integer of at most `bits` bits (0..=96).
    fn bits(&mut self, bits: u64) -> u128 {
        if bits == 0 {
            return 0;
        }
        let wide = (u128::from(self.next_u64()) << 64) | u128::from(self.next_u64());
        wide >> (128 - bits)
    }
    /// A mantissa: random digits, or a shape near a power of two or ten.
    fn mantissa(&mut self) -> u128 {
        let small = self.below(3) as u128;
        let m = match self.below(12) {
            0 => MAX_MANTISSA - small,
            1 => 10u128.pow(self.below(29) as u32).saturating_sub(small),
            2 => 10u128.pow(self.below(29) as u32) + small,
            3 => (1u128 << self.below(96)) + small,
            4 => (1u128 << (1 + self.below(96))) - 1 - small.min(1),
            5 => 0,
            6 | 7 => {
                let b = self.below(33);
                self.bits(b)
            }
            8 => {
                let b = 64 + self.below(33);
                self.bits(b)
            }
            _ => {
                let b = self.below(97);
                self.bits(b)
            }
        };
        m.min(MAX_MANTISSA)
    }
    fn scale(&mut self) -> u32 {
        (match self.below(4) {
            0 => 0,
            1 => self.below(8),
            2 => 20 + self.below(9),
            _ => self.below(29),
        }) as u32
    }
    fn decimal(&mut self) -> Decimal {
        let m = self.mantissa();
        let s = self.scale();
        let neg = self.below(3) == 0;
        mk(m, neg, s)
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }
    fn digits(&mut self, n: u64) -> String {
        (0..n).map(|_| char::from(b'0' + self.below(10) as u8)).collect()
    }
}

/// The raw parts; `-d` is applied after `from_parts`, so a zero can be negative.
fn mk(m: u128, neg: bool, scale: u32) -> Decimal {
    let d = Decimal::from_parts(m as u32, (m >> 32) as u32, (m >> 64) as u32, false, scale);
    if neg {
        -d
    } else {
        d
    }
}

fn enc(d: &Decimal) -> String {
    let sign = if d.is_sign_negative() { "-" } else { "" };
    format!("{sign}{}:{}", d.mantissa().unsigned_abs(), d.scale())
}

fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

fn result(r: Result<Decimal, Error>) -> Value {
    match r {
        Ok(d) => json!(enc(&d)),
        Err(e) => json!({ "e": e.to_string(), "d": format!("{e:?}") }),
    }
}

fn caught<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|p| {
        p.downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_default()
    })
}

fn op(checked: Option<Decimal>, operator: impl FnOnce() -> Decimal) -> Value {
    let r = caught(operator);
    match (checked, r) {
        (Some(c), Ok(o)) => {
            assert_eq!(enc(&c), enc(&o), "checked and operator differ");
            json!(enc(&c))
        }
        (None, Err(msg)) => json!(format!("!{msg}")),
        _ => panic!("checked/operator mismatch"),
    }
}

fn binary_row(a: Decimal, b: Decimal) -> Value {
    json!([
        enc(&a),
        enc(&b),
        op(a.checked_add(b), || a + b),
        op(a.checked_sub(b), || a - b),
        op(a.checked_mul(b), || a * b),
        op(a.checked_div(b), || a / b),
        op(a.checked_rem(b), || a % b),
        a.cmp(&b) as i8,
    ])
}

/// `[parts, Display, JSON, to_f64, normalize, [[dp, round_dp(dp)], ...]]`.
fn unary_row(d: Decimal) -> Value {
    let s = d.scale();
    let mut dps = vec![0, 1, 2, s / 2, s.saturating_sub(1), s, s + 1];
    dps.sort_unstable();
    dps.dedup();
    let rounds: Vec<Value> = dps.iter().map(|&dp| json!([dp, enc(&d.round_dp(dp))])).collect();
    let f = d.to_f64().map(bits);
    assert_eq!(format!("{d:?}"), d.to_string(), "Debug differs from Display");
    json!([
        enc(&d),
        d.to_string(),
        serde_json::to_value(d).expect("json"),
        f,
        enc(&d.normalize()),
        rounds
    ])
}

const SPECIALS: &[&str] = &[
    "0",
    "0.00",
    "1",
    "-1",
    "1.0",
    "1.00",
    "2",
    "3",
    "-4",
    "7",
    "10",
    "100",
    "0.1",
    "0.5",
    "-0.5",
    "0.25",
    "2.5",
    "-3.5",
    "12345.6789",
    "-98765.4321",
    "0.3333333333333333333333333333",
    "0.6666666666666666666666666667",
    "79228162514264337593543950335",
    "-79228162514264337593543950335",
    "7.9228162514264337593543950335",
    "39614081257132168796771975168",
    "0.0000000000000000000000000001",
    "-0.0000000000000000000000000005",
    "4294967295",
    "4294967296",
    "18446744073709551616",
    "0.4294967295",
    "9999999999999999999999999999",
    "0.9999999999999999999999999999",
    "2.718281828459045235360287471",
    "0.000001",
    "0.3048",
    "1609.344",
    "0.45359237",
    "273.15",
];

fn specials() -> Vec<Decimal> {
    let mut v: Vec<Decimal> = SPECIALS.iter().map(|s| Decimal::from_str(s).expect("special")).collect();
    v.push(mk(0, true, 0));
    v.push(mk(0, true, 3));
    v.push(mk(0, false, 28));
    v
}

fn parse_inputs(rng: &mut Rng) -> Vec<String> {
    let listed = [
        "", "0", "-0", "+0", "00", "0.0", "-0.000", ".5", "5.", ".", "-", "+", "-.", "+.5", "-.5", "1.2.3", "1..2",
        "--1", "+-1", "1-", "1+", "1_000", "_1", "-_1", "1__0", "1._5", "1_.5", "1_", "0x10", " 1", "1 ", "1,5",
        "\u{661}", "\u{e9}", "1\u{e9}", "3.14159", "-3.14159", "+3.14159", "1e5", "1E-3", "1.5e", "e", "1.5e-30",
        "79228162514264337593543950335",
        "79228162514264337593543950336",
        "-79228162514264337593543950335",
        "79228162514264337593543950335.4",
        "79228162514264337593543950335.5",
        "79228162514264337593543950334.5",
        "7922816251426433759354395033.55",
        "7922816251426433759354395033.45",
        "792281625142643375935439503.355",
        "0.00000000000000000000000000001",
        "0.00000000000000000000000000005",
        "0.00000000000000000000000000015",
        "-0.000000000000000000000000000149",
        "1.00000000000000000000000000005",
        "1.0000000000000000000000000000500",
        "1.00000000000000000000000000004999",
        "123456789012345678901234567890",
        "12345678901234567890.123456789012345",
        "0.33333333333333333333333333333333333",
        "0.66666666666666666666666666666666666",
        "9.9999999999999999999999999999999",
        "99999999999999999999999999999.9",
        "999999999999999999999999999.99",
        "1844674407370954906",
        "18446744073709549060",
        "18446744073709551615",
        "18446744073709551616",
        "1_000_000_000_000_000_000_000",
        "1.000_000_000_000_000_000_000_000_000_5",
        "1.0000000000000000000000000000_5",
        "1.0000000000000000000000000000_",
        "1.0000000000000000000000000000__",
        "1.0000000000000000000000000000__7",
        "1.0000000000000000000000000000x",
        "1.0000000000000000000000000000.5",
        "1.00000000000000000000000000005.5",
        "1.00000000000000000000000000005x",
        "1.00000000000000000000000000005_",
        "1.00000000000000000000000000006-",
        "12345678901234567890123456789.5",
        "1234567890123456789012345678.95",
        "123456789012345678901234567.895",
        "1234567890123456789012345678901.5",
        "0.0000000000000000000000000000000000000000001",
        "00000000000000000000000000000000000000001.5",
        "-00000000000000000000000000000000000000000",
        "1844674407370954906.5",
        "18446744073709549065.5",
        "184467440737095490655.5_5",
        "184467440737095490655_5.5",
        "184467440737095490655.5.5",
        "184467440737095490655x",
        "18446744073709549065_",
        "18446744073709549065.",
        "18446744073709549065..",
        "79228162514264337593543950335_",
        "792281625142643375935439503350",
        "7922816251426433759354395033.5_",
        "7922816251426433759354395033_.5",
        "79228162514264337593543950335.0000000",
        "4.99999999999999999999999999995",
        "9.99999999999999999999999999995",
        "0.99999999999999999999999999995",
        "-0.99999999999999999999999999999",
    ];
    let mut out: Vec<String> = listed.iter().map(|s| (*s).to_string()).collect();
    // Free-form strings over a small alphabet, mostly digits.
    for _ in 0..1200 {
        let len = 1 + rng.below(40);
        let s: String = (0..len)
            .map(|i| match rng.below(30) {
                0 => '.',
                1 => '_',
                2 if i == 0 => '-',
                3 if i == 0 => '+',
                4 => ['e', 'E', 'x', ' ', '-'][rng.below(5) as usize],
                _ => char::from(b'0' + rng.below(10) as u8),
            })
            .collect();
        out.push(s);
    }
    // Well-formed numbers of every length, sometimes with underscores.
    for _ in 0..1200 {
        let sign = rng.pick(&["", "", "-", "+"]);
        let int_len = rng.below(36);
        let frac_len = match rng.below(3) {
            0 => 0,
            _ => rng.below(40),
        };
        let mut s = format!("{sign}{}", rng.digits(int_len));
        if frac_len > 0 || rng.below(4) == 0 {
            s.push('.');
            s.push_str(&rng.digits(frac_len));
        }
        if rng.below(5) == 0 && s.len() > 2 {
            let at = 1 + rng.below(s.len() as u64 - 1) as usize;
            if s.is_char_boundary(at) {
                s.insert(at, '_');
            }
        }
        if rng.below(10) == 0 {
            s = s.replacen('0', "9", 1);
        }
        out.push(s);
    }
    out
}

fn scientific_inputs(rng: &mut Rng) -> Vec<String> {
    let listed = [
        "1e0", "1E0", "1.5e-6", "1.5E+6", "1e28", "1e29", "1e-28", "1e-29", "1.5e-28", "0.5e-28", "-1.5e-28",
        "12345e-28", "1.2345e-25", "9.5e-28", "-9.5e-28", "1.25e-27", "1.35e-27", "1e", "e5", "1e-", "1e+", "1e+5",
        "1e-+5", "1e--5", "1e5.5", "1.5", "abc", "1e4294967295", "1e4294967296", "1e-4294967296",
        "79228162514264337593543950335e0", "79228162514264337593543950335e1", "7.9228162514264337593543950335e28",
        "7.9228162514264337593543950336e28", "7.9228162514264337593543950335e27", "1e19", "1e20", "1.0e19",
        "12.34e1", "12.34e2", "12.34e3", "-0e5", "-0.0e-3", "0e-30", "0e-28", "1_000e3", "1e1_0", " 1e5", "1e5 ",
        "1ee5", "1e5e5", "1.5e-0", "1.50E-2", "-1.50E+2", "0.000000000000000000000000000001e-1", "5e-28",
        "123456789012345678901234567.8e-5", "-123456789012345678901234567.8e-20", "1.5e-28x", "4.5e-28",
        "79228162514264337593543950335e-28", "79228162514264337593543950335e-1", "9.99e27", "1.5e+28",
        "0.00000000000000000000000000015e-1", "1.11e-27", "-1.15e-27", "2.5e-28", "3.5e-28",
    ];
    let mut out: Vec<String> = listed.iter().map(|s| (*s).to_string()).collect();
    for _ in 0..1200 {
        let sign = rng.pick(&["", "", "-"]);
        let int_len = 1 + rng.below(12);
        let frac_len = rng.below(18);
        let mut s = format!("{sign}{}", rng.digits(int_len));
        if frac_len > 0 {
            s.push('.');
            s.push_str(&rng.digits(frac_len));
        }
        s.push_str(rng.pick(&["e", "E", "e-", "e+", "E-", "e-", "e-"]));
        s.push_str(&rng.below(36).to_string());
        out.push(s);
    }
    out
}

fn f64_inputs(rng: &mut Rng) -> Vec<f64> {
    let mut out = vec![
        0.0,
        -0.0,
        0.1,
        0.2,
        0.3,
        0.1 + 0.2,
        1.0 / 3.0,
        2.0 / 3.0,
        -1.0 / 3.0,
        100.0 / 3.0,
        1.0 / 7.0,
        1e-30,
        -1e-30,
        1e-28,
        1e-29,
        5e-29,
        -5e-29,
        5.000000000000001e-29,
        1.5e-28,
        1e30,
        1e28,
        7.9e28,
        7.922816251426434e28,
        7.922816251426433e28,
        8e28,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        5e-324,
        -5e-324,
        2.2250738585072009e-308,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.3048,
        1609.344,
        0.45359237,
        2.54,
        1.8,
        273.15,
        9.80665,
        std::f64::consts::PI,
        std::f64::consts::E,
        std::f64::consts::SQRT_2,
        1e15,
        1e16,
        1e17,
        1e22,
        1e23,
        123456789.123456789,
        0.000001,
        1e-7,
        1e-10,
        4294967296.0,
        9007199254740992.0,
        9007199254740994.0,
        9223372036854775808.0,
        18446744073709551616.0,
        79228162514264337593543950336.0,
        79228162514264328797450928128.0,
        0.5,
        1.5,
        2.5,
        -2.5,
        299792458.0,
        6.02214076e23,
        1.602176634e-19,
        6.62607015e-34,
        9.1093837015e-31,
        1.0,
        -1.0,
        42.0,
        0.07,
        0.035,
        1.005,
        1234.5678,
        -0.001,
        3.0e-5,
    ];
    for _ in 0..600 {
        out.push(f64::from_bits(rng.next_u64()));
    }
    for _ in 0..1400 {
        let x = match rng.below(4) {
            0 => {
                let unit = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
                unit * 10f64.powi(rng.below(62) as i32 - 31)
            }
            1 => (rng.next_u64() >> rng.below(64)) as f64,
            2 => rng.below(1_000_000) as f64 / 10f64.powi(rng.below(12) as i32),
            _ => {
                let e = rng.below(200) as i32 - 100;
                f64::from_bits(((1023 + e) as u64) << 52 | (rng.next_u64() >> 12))
            }
        };
        out.push(if rng.coin() { -x } else { x });
    }
    out
}

fn unary_inputs(rng: &mut Rng) -> Vec<Decimal> {
    let mut out = specials();
    for &m in &[1u128, 5, 15, 25, 123_456_789, 999_999_999_999, 1u128 << 53, (1u128 << 53) + 1, MAX_MANTISSA] {
        for s in 0..=28 {
            out.push(mk(m, false, s));
            out.push(mk(m, true, s));
        }
    }
    for s in 0..=28 {
        out.push(mk(10u128.pow(28) / 3, false, s));
        out.push(mk(0, true, s));
    }
    for _ in 0..1500 {
        out.push(rng.decimal());
    }
    out
}

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng(0x5eed_dec1_3a1_0254);

    let parse: Vec<Value> = parse_inputs(&mut rng)
        .into_iter()
        .map(|s| {
            let r = caught(|| Decimal::from_str(&s)).expect("from_str panicked");
            json!([s, result(r)])
        })
        .collect();

    let scientific: Vec<Value> = scientific_inputs(&mut rng)
        .into_iter()
        .map(|s| {
            let sci = caught(|| Decimal::from_scientific(&s)).expect("from_scientific panicked");
            let lossy = caught(|| Decimal::from_str(&s)).expect("from_str panicked");
            json!([s, result(sci), result(lossy)])
        })
        .collect();

    let from_f64: Vec<Value> = f64_inputs(&mut rng)
        .into_iter()
        .map(|x| json!([bits(x), Decimal::try_from(x).ok().map(|d| enc(&d))]))
        .collect();

    let unary: Vec<Value> = unary_inputs(&mut rng).into_iter().map(unary_row).collect();

    let debug: Vec<Value> = specials()
        .into_iter()
        .map(|d| json!([enc(&d), format!("{d:?}"), format!("{d}")]))
        .collect();

    let sp = specials();
    let mut binary: Vec<Value> = Vec::new();
    for &a in &sp {
        for &b in &sp {
            binary.push(binary_row(a, b));
        }
    }
    for _ in 0..1800 {
        let a = rng.decimal();
        let b = if rng.below(8) == 0 { sp[rng.below(sp.len() as u64) as usize] } else { rng.decimal() };
        binary.push(binary_row(a, b));
    }

    let sections: Vec<(&str, Vec<Value>)> = vec![
        ("parse", parse),
        ("scientific", scientific),
        ("from_f64", from_f64),
        ("unary", unary),
        ("debug", debug),
        ("binary", binary),
    ];

    // One entry per line keeps diffs of the generated file readable.
    let mut out = String::from("{\n");
    for (i, (name, rows)) in sections.iter().enumerate() {
        out.push_str(&format!("{}: [\n", json!(name)));
        for (j, row) in rows.iter().enumerate() {
            out.push_str(&serde_json::to_string(row).expect("json"));
            out.push_str(if j + 1 < rows.len() { ",\n" } else { "\n" });
        }
        out.push_str(if i + 1 < sections.len() { "],\n" } else { "]\n" });
    }
    out.push_str("}\n");
    print!("{out}");
}
