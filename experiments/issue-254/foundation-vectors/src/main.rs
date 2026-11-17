//! Prints JSON vectors of the ported foundation modules (issue #254).

#![allow(dead_code)]

#[path = "../../../../src/error.rs"]
mod error;
#[path = "../../../../src/utils.rs"]
mod utils;
#[path = "../../../../src/types/dimension.rs"]
mod dimension;
#[path = "../../../../src/grammar/locale_numbers.rs"]
mod locale_numbers;

use dimension::Dimension;
use error::CalculatorError;
use serde_json::{json, Value};

fn error_vector(error: &CalculatorError) -> Value {
    let info = error.to_error_info();
    let params = info.params.as_ref().map(|params| {
        let mut entries: Vec<_> = params.iter().collect();
        entries.sort();
        entries
            .into_iter()
            .map(|(k, v)| json!([k, v]))
            .collect::<Vec<_>>()
    });
    json!({
        "display": error.to_string(),
        "debug": format!("{error:?}"),
        "key": info.key,
        "params": params,
        "json_keys": serde_json::to_value(&info).unwrap().as_object().unwrap().keys().cloned().collect::<Vec<_>>(),
    })
}

/// xorshift64*, so the fuzz inputs are reproducible.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn scan_vector(input: &str, start: usize, allow_comma: bool, decimal_comma: bool) -> Value {
    let chars: Vec<char> = input.chars().collect();
    let result = std::panic::catch_unwind(|| {
        locale_numbers::scan(&chars, start, allow_comma, decimal_comma)
    });
    let outcome = match result {
        Ok(Ok((text, end))) => json!({ "ok": [text, end] }),
        Ok(Err(error)) => json!({ "err": format!("{error:?}") }),
        Err(_) => json!({ "panic": true }),
    };
    json!([input, start, allow_comma, decimal_comma, outcome])
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let errors = vec![
        CalculatorError::parse("invalid syntax"),
        CalculatorError::unexpected_token("+", "number", 5),
        CalculatorError::unit_mismatch("add", "USD", "hours"),
        CalculatorError::eval("bad \"quote\"\n\tline é \u{7f} \u{200b}"),
        CalculatorError::DivisionByZero,
        CalculatorError::InvalidDateTime("not a date".to_string()),
        CalculatorError::UnknownCurrency("XYZ".to_string()),
        CalculatorError::CurrencyConversion {
            from: "USD".to_string(),
            to: "EUR".to_string(),
            reason: "no rate".to_string(),
        },
        CalculatorError::NoHistoricalRate {
            currency: "USD".to_string(),
            date: "2020-01-01".to_string(),
        },
        CalculatorError::Overflow,
        CalculatorError::InvalidOperation("Cannot add".to_string()),
        CalculatorError::EmptyInput,
        CalculatorError::unknown_function("foo"),
        CalculatorError::invalid_args("sqrt", "needs 1 argument"),
        CalculatorError::domain("sqrt of negative"),
        CalculatorError::SymbolicResult {
            expression: "integrate x dx".to_string(),
            result: "x^2/2 + C".to_string(),
            latex_input: "\\int x\\,dx".to_string(),
            latex_result: "\\frac{x^{2}}{2} + C".to_string(),
        },
    ];
    let errors: Vec<Value> = errors.iter().map(error_vector).collect();

    let long = "x".repeat(60);
    let links: Vec<Value> = [
        ("invalid input", "Parse error"),
        ("2 + € × 3 — 😀 ~a_b.c-d", "Unexpected token '€'"),
        ("line1\nline2\r\n\t%&?=#+", "Error: \"x\""),
        (long.as_str(), "e"),
        ("ééééééééééééééééééééééééééééééééééééééééééééééééééééé", "e"),
        ("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀", ""),
        ("", ""),
    ]
    .iter()
    .map(|(input, error)| json!([input, error, utils::generate_issue_link(input, error)]))
    .collect();
    let truncates: Vec<Value> = [
        ("hello world", 5),
        ("hi", 10),
        ("", 0),
        ("abc", 0),
        ("abc", 3),
        ("😀é漢x", 2),
        ("😀é漢x", 3),
        ("😀é漢x", 4),
    ]
    .iter()
    .map(|(s, n)| json!([s, n, utils::truncate(s, *n)]))
    .collect();

    let length = Dimension::base("length").unwrap();
    let mass = Dimension::base("mass").unwrap();
    let time = Dimension::base("time").unwrap();
    let currency = Dimension::base("currency").unwrap();
    let energy = mass.mul(&length.pow(2)).div(&time.pow(2));
    let dimensions = json!({
        "speed": length.div(&time).to_string(),
        "energy": energy.to_string(),
        "energy_root2": energy.root(2).map(|d| d.to_string()),
        "energy_pow_neg3": energy.pow(-3).to_string(),
        "energy_root_neg2": energy.root(-2).map(|d| d.to_string()),
        "energy_root0": energy.root(0).map(|d| d.to_string()),
        "currency_per_mass": currency.div(&mass).to_string(),
        "none": Dimension::NONE.to_string(),
        "default": Dimension::default().to_string(),
        "unknown": Dimension::base("speed").map(|d| d.to_string()),
        "pow0": energy.pow(0).to_string(),
        "neg_root": length.pow(-4).root(2).map(|d| d.to_string()),
        "neg_root_odd": length.pow(-3).root(2).map(|d| d.to_string()),
    });

    let mut scans: Vec<Value> = Vec::new();
    let curated = [
        "1,234.5", "1.234,5", "1 234,5", "1'234.5", "1’234.5", "12,34,567", "1,1", "1,2",
        "1.000 000", "123_456\u{2009}789", "1,23,4567", "1'23", "1 23,4", "1,234,", "1,,234",
        "1,234", "12,345", "1,234,567.89", "1.234.567,89", "1 234 567", "1\u{a0}234",
        "1\u{202f}234,56", "1\u{2007}234", "1 2345", "1234 567", "0.5", ".5", ",5", "1_000",
        "1__000", "_1", "1_", "1_.5", "1._5", "1.5_5", "6'2\"", "1'234'567", "1'234’567",
        "12'345", "1'2345", "3.14159", "1.2.3", "1,2,3", "1,23", "1,2345", "1,234.567.8",
        "1.234,567,8", "1,234.5 678", "0.123 456", "0.123 45", "1 234.5", "1 234_567",
        "1_234 567", "0b1010", "0B11.1", "0o17", "0x1F", "0xff_ff", "0x1.8", "0b1e10",
        "0b1e-10", "0x1e+2", "0b1e", "0b1e-", "0b1e_1", "0x", "0b2", "0b", "0b1_",
        "0b1__0", "0x.8", "0b0.1", "0b0.0001", "0x0.1", "0o0.1", "0b0.01e-1", "0x1.",
        "0b1.0.1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "0b1e1111111", "0b1e100000000000000000000000000000000", "0x1e-10",
        "0o1e-77", "0b0.00000000000000000000000000001", "0x0.000000000000000000000001",
        "123", "0", "00", "007", "1e5", "12 345 678", "1 234 5678", "12 34", "1 234,5 6",
        "1.234 567,8", "1 000.5", "1.5", "1,5", "12,3", "123,4", "1234,5", "1,2345,6",
        "99,99,999", "9,99,99,999", "1,00,000", "10,00,000.50", "1,00,00", "100,00,000",
        "1,000,00", "١٢٣", "1.", "1,", "1 ", "1'", "1_000_000.000_1",
    ];
    for input in curated {
        for (allow_comma, decimal_comma) in [(true, false), (true, true), (false, false)] {
            scans.push(scan_vector(input, 0, allow_comma, decimal_comma));
        }
    }
    for (input, start) in [("x 1,234.5", 2), ("ab0x1F", 2), ("é 1 234", 2), ("😀1,1", 1), ("12", 2)] {
        scans.push(scan_vector(input, start, true, false));
    }

    let alphabet: Vec<char> = "0123456789000111.,,'’ \u{a0}\u{2009}\u{202f}\u{2007}_bxoeE-+aFB."
        .chars()
        .collect();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut fuzz: Vec<Value> = Vec::new();
    for _ in 0..200_000 {
        let len = 1 + rng.below(14);
        let mut input = String::new();
        for i in 0..len {
            let ch = if i == 0 && rng.below(3) > 0 {
                char::from(b'0' + rng.below(10) as u8)
            } else {
                alphabet[rng.below(alphabet.len())]
            };
            input.push(ch);
        }
        let allow_comma = rng.below(4) > 0;
        let decimal_comma = rng.below(3) == 0;
        fuzz.push(scan_vector(&input, 0, allow_comma, decimal_comma));
    }

    println!(
        "{}",
        json!({
            "errors": errors,
            "links": links,
            "truncates": truncates,
            "dimensions": dimensions,
            "scans": scans,
            "fuzz": fuzz,
        })
    );
}
