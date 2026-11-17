//! Vectors for js/src/rust/{str,num,fmt}.js: Rust `str`/`char` semantics,
//! `f64` formatting and parsing, integer parsing and `as` casts.
#![allow(
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use serde_json::{json, Value};
use std::panic;
use std::sync::{Arc, Mutex};

/// A deterministic xorshift64* generator.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

fn floats(rng: &mut Rng) -> Vec<f64> {
    let mut values = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        0.2,
        0.1 + 0.2,
        1.5,
        -2.5,
        0.5,
        2.5,
        3.5,
        -0.5,
        0.49999999999999994,
        1e-4,
        9.999999999999999e-5,
        1e-5,
        1.5e-5,
        1e15,
        1e16,
        9999999999999998.0,
        1e17,
        1.2345e20,
        1e21,
        1e22,
        1e23,
        1e100,
        1e300,
        1e-300,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        5e-324,
        -5e-324,
        2.2250738585072009e-308,
        f64::EPSILON,
        std::f64::consts::PI,
        std::f64::consts::E,
        1.0 / 3.0,
        2.0 / 3.0,
        123456789.123456789,
        0.000123,
        0.001,
        100.0,
        1234567.0,
        4503599627370496.0,
        4503599627370497.0,
        9007199254740992.0,
        9007199254740993.0,
        9223372036854775808.0,
        -9223372036854775808.0,
        18446744073709551616.0,
        1.7014118346046923e38,
        3.402823669209385e38,
        255.5,
        256.0,
        -1.5,
        -0.4,
        4294967295.5,
        4294967296.0,
        2147483647.5,
        -2147483648.5,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for exponent in -30..=30 {
        let power: f64 = format!("1e{exponent}").parse().unwrap();
        values.push(power);
        values.push(1.5 * power);
    }
    for _ in 0..300 {
        let x = f64::from_bits(rng.next());
        if x.is_finite() {
            values.push(x);
        }
    }
    for _ in 0..300 {
        let mantissa = (rng.next() % 2_000_000) as f64 - 1_000_000.0;
        let scale: f64 = format!("1e{}", (rng.next() % 24) as i32 - 12)
            .parse()
            .unwrap();
        values.push(mantissa * scale);
    }
    values
}

fn float_vectors(rng: &mut Rng) -> Value {
    let cases: Vec<Value> = floats(rng)
        .into_iter()
        .map(|x| {
            json!({
                "bits": bits(x),
                "display": x.to_string(),
                "debug": format!("{x:?}"),
                "round": bits(x.round()),
                "signum": bits(x.signum()),
                "degrees": bits(x.to_degrees()),
                "radians": bits(x.to_radians()),
                "as_i64": (x as i64).to_string(),
                "as_i32": (x as i32).to_string(),
                "as_u32": (x as u32).to_string(),
                "as_u64": (x as u64).to_string(),
                "as_u8": (x as u8).to_string(),
                "as_i128": (x as i128).to_string(),
                "max_nan": bits(x.max(f64::NAN)),
                "min_one": bits(x.min(1.0)),
            })
        })
        .collect();
    Value::Array(cases)
}

fn parse_float_vectors(rng: &mut Rng) -> Value {
    let mut inputs: Vec<String> = [
        "", "0", "-0", "+0", "1", "1.", ".5", "-.5", "+.5e-3", ".", "-", "+", "e5", "1e", "1e+",
        "1e5", "1E5", "1e+5", "1e-5", "1.5e300", "1e400", "-1e400", "1e-400", "inf", "-inf",
        "+inf", "Inf", "INF", "infinity", "Infinity", "-INFINITY", "infinit", "nan", "NaN",
        "-nan", "+NaN", "nan(1)", " 1", "1 ", "1_000", "0x10", "1,5", "1.5.5", "--1", "+-1",
        "١", "0.1", "0.30000000000000004", "2.2250738585072011e-308", "4.9e-324", "2e-324",
        "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "123456789012345678901234567890", "0000000001", "1e0000000000000000000005",
        "9007199254740993", "1.7976931348623157e308", "1.7976931348623159e308", "1e-7",
        "12.34E-2", "5.", "5.e3", "infe", "nanx", "1d", "1f", "١٢٣",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    for _ in 0..300 {
        let x = f64::from_bits(rng.next());
        inputs.push(format!("{x:e}"));
        inputs.push(x.to_string());
    }
    let cases: Vec<Value> = inputs
        .into_iter()
        .map(|input| match input.parse::<f64>() {
            Ok(x) => json!({ "input": input, "ok": bits(x) }),
            Err(error) => json!({ "input": input, "err": error.to_string() }),
        })
        .collect();
    Value::Array(cases)
}

fn int_result<T: std::str::FromStr<Err = std::num::ParseIntError> + ToString>(
    input: &str,
) -> Value {
    match input.parse::<T>() {
        Ok(value) => json!({ "ok": value.to_string() }),
        Err(error) => json!({ "err": error.to_string() }),
    }
}

fn parse_int_vectors() -> Value {
    let inputs = [
        "",
        "0",
        "-0",
        "+0",
        "+",
        "-",
        "++1",
        "+-1",
        "-+1",
        "--1",
        "1",
        "-1",
        "42",
        "007",
        " 1",
        "1 ",
        "1_000",
        "1.0",
        "1e3",
        "a",
        "x1",
        "1x",
        "٣",
        "127",
        "128",
        "-128",
        "-129",
        "255",
        "256",
        "2147483647",
        "2147483648",
        "-2147483648",
        "-2147483649",
        "4294967295",
        "4294967296",
        "9223372036854775807",
        "9223372036854775808",
        "-9223372036854775808",
        "-9223372036854775809",
        "18446744073709551615",
        "18446744073709551616",
        "170141183460469231731687303715884105727",
        "170141183460469231731687303715884105728",
        "-170141183460469231731687303715884105728",
        "-170141183460469231731687303715884105729",
        "340282366920938463463374607431768211455",
        "340282366920938463463374607431768211456",
        "99999999999x",
        "-99999999999x",
        "x99999999999",
        "00000000000000000000000000000000000000000001",
    ];
    let cases: Vec<Value> = inputs
        .iter()
        .map(|input| {
            json!({
                "input": input,
                "i8": int_result::<i8>(input),
                "u8": int_result::<u8>(input),
                "i32": int_result::<i32>(input),
                "u32": int_result::<u32>(input),
                "i64": int_result::<i64>(input),
                "u64": int_result::<u64>(input),
                "i128": int_result::<i128>(input),
                "u128": int_result::<u128>(input),
            })
        })
        .collect();
    Value::Array(cases)
}

/// Inclusive code point ranges where `property` holds.
fn ranges(property: impl Fn(char) -> bool) -> Value {
    let mut result = Vec::new();
    let mut start: Option<u32> = None;
    for code in 0..=0x0011_0000_u32 {
        let holds = char::from_u32(code).is_some_and(&property);
        match (holds, start) {
            (true, None) => start = Some(code),
            (false, Some(first)) => {
                result.push(json!([first, code - 1]));
                start = None;
            }
            _ => {}
        }
    }
    Value::Array(result)
}

fn escapes_in_str(c: char) -> bool {
    let debug = format!("{:?}", c.to_string());
    debug != format!("\"{c}\"")
}

fn escapes_as_char(c: char) -> bool {
    format!("{c:?}") != format!("'{c}'")
}

fn char_vectors() -> Value {
    json!({
        "whitespace": ranges(char::is_whitespace),
        "alphabetic": ranges(char::is_alphabetic),
        "numeric": ranges(char::is_numeric),
        "alphanumeric": ranges(char::is_alphanumeric),
        "lowercase": ranges(char::is_lowercase),
        "uppercase": ranges(char::is_uppercase),
        "debug_str_escaped": ranges(escapes_in_str),
        "debug_char_escaped": ranges(escapes_as_char),
    })
}

const SAMPLES: &[&str] = &[
    "",
    " ",
    "  a  ",
    "\u{85}x\u{85}",
    "\u{feff}x\u{feff}",
    "\u{a0}x\u{3000}",
    "\t\n x \r\n",
    "a b\tc\u{2003}d",
    "Straße",
    "ΣΑΣ ΟΔΟΣ.",
    "İstanbul",
    "ǅ",
    "ﬃ",
    "Ⅻ",
    "x\u{301}",
    "\u{301}x",
    "a\"b'c\\d",
    "\0\u{7}\u{1b}\u{7f}",
    "\u{200b}\u{200d}\u{2028}",
    "😀 😀",
    "日本語",
    "ŉ",
    "line1\nline2\r\nline3\r",
    "\n",
    "\r\n",
    "a\n\n",
    "a\r",
    "\r",
    "x\r\ny\n",
    "Hello, World!",
    "\u{e000}\u{f8ff}\u{10ffff}",
    "\u{378}",
    "ß",
    "\u{1e9e}",
    "AbC xYz 123",
];

fn str_vectors() -> Value {
    let cases: Vec<Value> = SAMPLES
        .iter()
        .map(|s| {
            json!({
                "input": s,
                "trim": s.trim(),
                "trim_start": s.trim_start(),
                "trim_end": s.trim_end(),
                "split_whitespace": s.split_whitespace().collect::<Vec<_>>(),
                "lines": s.lines().collect::<Vec<_>>(),
                "to_lowercase": s.to_lowercase(),
                "to_uppercase": s.to_uppercase(),
                "to_ascii_lowercase": s.to_ascii_lowercase(),
                "to_ascii_uppercase": s.to_ascii_uppercase(),
                "debug": format!("{s:?}"),
                "len": s.len(),
                "char_count": s.chars().count(),
                "char_indices": s.char_indices().map(|(i, _)| i).collect::<Vec<_>>(),
                "split_space": s.split(' ').collect::<Vec<_>>(),
                "split_empty": s.split("").collect::<Vec<_>>(),
                "replace_empty": s.replace("", "-"),
                "replace_dollar": s.replace('a', "$&"),
                "splitn2": s.splitn(2, char::is_whitespace).collect::<Vec<_>>(),
                "split_once_ws": s.split_once(char::is_whitespace),
                "rsplit_once_ws": s.rsplit_once(char::is_whitespace),
                "trim_start_digits": s.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' '),
                "trim_end_ws_chars": s.trim_end_matches(&[' ', '\n', '\r'][..]),
                "matches_a": s.matches('a').count(),
            })
        })
        .collect();
    let pairs = [
        ("abc", "ABC"),
        ("ß", "SS"),
        ("Straße", "STRASSE"),
        ("é", "É"),
        ("a1", "A1"),
        ("", ""),
        ("a", "ab"),
    ];
    let eq: Vec<Value> = pairs
        .iter()
        .map(|(a, b)| json!([a, b, a.eq_ignore_ascii_case(b)]))
        .collect();
    let trims = [
        ("xxabcxx", "x"),
        ("ababcab", "ab"),
        ("aaa", "aa"),
        ("abc", ""),
    ];
    let trim_matches: Vec<Value> = trims
        .iter()
        .map(|(s, p)| json!([s, p, s.trim_start_matches(p), s.trim_end_matches(p)]))
        .collect();
    json!({ "cases": cases, "eq_ignore_ascii_case": eq, "trim_matches": trim_matches })
}

fn slice_panics() -> Value {
    let message = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&message);
    panic::set_hook(Box::new(move |info| {
        let text = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .map(ToString::to_string)
            })
            .unwrap_or_default();
        *sink.lock().unwrap() = text;
    }));
    let long = "é".repeat(200);
    let inputs: Vec<(&str, usize, Option<usize>)> = vec![
        ("aé", 0, Some(2)),
        ("aé", 0, Some(3)),
        ("aé", 0, Some(4)),
        ("aé", 2, Some(1)),
        ("aé", 2, Some(9)),
        ("aé", 9, Some(2)),
        ("aé", 9, Some(1)),
        ("aé", 4, Some(4)),
        ("aé", 3, Some(3)),
        ("aé", 2, Some(2)),
        ("aé", 2, None),
        ("aé", 4, None),
        ("aé", 3, None),
        ("😀x", 1, Some(5)),
        ("😀x", 1, Some(3)),
        ("😀x", 3, Some(1)),
        ("😀x", 0, Some(3)),
        ("😀x", 4, Some(5)),
        ("😀x", 6, Some(7)),
        ("abcαβγ", 4, Some(9)),
        ("abcαβγ", 4, Some(10)),
        ("abcαβγ", 3, Some(10)),
        ("", 0, Some(0)),
        ("", 0, Some(1)),
        ("", 1, None),
        (&long, 0, Some(301)),
    ];
    let cases: Vec<Value> = inputs
        .into_iter()
        .map(|(s, from, to)| {
            let result = panic::catch_unwind(|| match to {
                Some(to) => s[from..to].to_string(),
                None => s[from..].to_string(),
            });
            match result {
                Ok(slice) => json!({ "input": s, "from": from, "to": to, "ok": slice }),
                Err(_) => json!({ "input": s, "from": from, "to": to, "panic": message.lock().unwrap().clone() }),
            }
        })
        .collect();
    let _ = panic::take_hook();
    Value::Array(cases)
}

fn main() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let vectors = json!({
        "floats": float_vectors(&mut rng),
        "parse_f64": parse_float_vectors(&mut rng),
        "parse_int": parse_int_vectors(),
        "unicode_version": char::UNICODE_VERSION,
        "chars": char_vectors(),
        "strings": str_vectors(),
        "slice_panics": slice_panics(),
    });
    println!("{vectors}");
}
