//! Test vectors for js/src/grammar/lexer.js and arithmetic_words.js
//! (issue #254): the token stream of every input of `parity/corpus.json`
//! plus extra lexer edge cases, tokenized by the calculator crate itself
//! (`link_calculator::grammar::Lexer`, unchanged sources).
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --features calculator --bin lexer > js/test/vectors/lexer.json
//!
//! Encoding: `{"input": ..., "tokens": ["{:?}" of each Token]}` or
//! `{"input": ..., "error": "{:?}" of the CalculatorError}`.

use link_calculator::grammar::Lexer;
use serde_json::{json, Value};

/// Inputs beyond the corpus that exercise the lexer's corners.
const EXTRA: &[&str] = &[
    "", "   ", "2 ** 3", "2**3", "x ** -1", "**2", "5!", "5! != 120", "1 ≠ 2", "a <= b >= c < d > e == f",
    "3 ≤ 4 ≥ 2", "2 × 3 · 4 ⋅ 5 ÷ 6 − 1", "1 ± 0.1", "m² + s⁻¹ + 10⁺³", "√16 + ∛27", "°C", "20°C",
    "20 °F", "°Celsius", "℃ ℉", "6'2\"", "6′2″", "$10 + €5 + £3 + ¥1 + ₽2 + ₹3 + ₩4 + ₿5 + ₫6 + ₸7",
    ".5", ".", "1.", "1e5", "1E-5", "1e+5", "2e", "2e+", "1e1_000", "1_000_000", "1,234.5", "1.234,5",
    "1 234,5", "max(1,234, 5)", "jan 1,234", "0x1F", "0b101", "0o17", "1,1", "12,34,567",
    "2026-01-22", "2026-1-2", "15/10/2025", "15.10.2025", "1.2.3.4", "12.3456", "2026-20-5",
    "2026-01-22-1", "2026/01-22", "руб.", "кг. 5", "янв.2026", "abc_def1", "π * 2", "fact(5)",
    "arcsin(1) + arccos(0) + arctan(1)", "mod(17, 5)", "17 mod 5", "17 modulo 5", "a at b as c in d to e",
    "until of and compare vs versus", "38% от 100к", "1000 рублей в долларах", "11 по мск",
    "1000 рублей на 11 апреля 2026", "1000 dollars en euros", "1000美元换成欧元", "1000 डॉलर में यूरो",
    "1000 دولار إلى يورو", "नमस्ते", "café", "e\u{301}", "x\u{20DD}", "@", "#1", "1 & 2", "~", "\u{0}",
    "seven plus three", "twenty-one times two", "one hundred twenty three", "two three", "five-two",
    "ONE + one", "multiplied by", "multiplied by 3", "умножить на 7", "по модулю 3", "dividido por 2",
    "двадцать один плюс два", "二十三加五", "二加二", "千克", "十二月", "一万二千", "三百二十", "दो सौ",
    "10 percent of 50", "20% of 30", "50 的 20%", "百分之二十", "增加 10%", "?", "x + ? = 5", "a:b",
    "12:30", "1 = 1", "(1 + 2) * 3", "sin(1,5)", "sqrt(2,25)", "max(1, 2)", "now", "now + 1,5",
    "ten five", "zero", "one zero", "a hundred", "thousand", "two thousand thousand",
    "eleven hundred", "nine hundred ninety nine thousand nine hundred ninety nine",
    "TEN", "MIL", "DOS", "mil", "dos", "sek", "SEK", "tres mil", "cien", "cento e vinte",
    "ﬁ", "Ⅻ", "٣ + ٤", "१२", "x²y", "a_b²", "e²", "3.14.15", "1-2-3", "2026.01.22", "01/22/2026",
];

fn tokenize(input: &str) -> Value {
    match Lexer::new(input).tokenize() {
        Ok(tokens) => json!({
            "input": input,
            "tokens": tokens.iter().map(|t| format!("{t:?}")).collect::<Vec<_>>(),
        }),
        Err(error) => json!({ "input": input, "error": format!("{error:?}") }),
    }
}

fn main() {
    let corpus: Value = serde_json::from_str(include_str!("../../../../../parity/corpus.json"))
        .expect("parity/corpus.json is JSON");
    let mut inputs: Vec<String> = corpus["inputs"]
        .as_array()
        .expect("inputs")
        .iter()
        .map(|input| input.as_str().expect("string input").to_string())
        .collect();
    inputs.extend(EXTRA.iter().map(|s| (*s).to_string()));
    let vectors: Vec<Value> = inputs.iter().map(|input| tokenize(input)).collect();
    println!("{}", serde_json::to_string_pretty(&vectors).expect("JSON"));
}
