//! Test vectors for js/src/grammar/expression_parser.js and token_parser.js
//! (issue #254): the parsed Expression of every input of `parity/corpus.json`,
//! parsed by the calculator crate itself (`ExpressionParser::parse` and
//! `parse_interpretations`, unchanged sources).
//!
//!   node scripts/bounded-cargo.mjs run --release \
//!     --manifest-path experiments/issue-254/rust-vectors/Cargo.toml \
//!     --features calculator --bin expression_parser > js/test/vectors/expression_parser.json
//!
//! Encoding: `{"input", "lino": to_lino(), "display": "{}", "json": serde
//! JSON, "live": contains_live_time(), "interpretations": [to_lino() of each]}`
//! or `{"input", "error": "{:?}" of the CalculatorError}`. Live expressions
//! (`now`) embed the current instant in "json"/"display".

use link_calculator::grammar::ExpressionParser;
use serde_json::{json, Value};

fn parse(parser: &ExpressionParser, input: &str) -> Value {
    match parser.parse(input) {
        Ok(expr) => {
            let interpretations = parser
                .parse_interpretations(input)
                .map(|all| all.iter().map(|e| e.to_lino()).collect::<Vec<_>>())
                .unwrap_or_default();
            json!({
                "input": input,
                "lino": expr.to_lino(),
                "display": expr.to_string(),
                "json": serde_json::to_value(&expr).expect("JSON"),
                "live": expr.contains_live_time(),
                "interpretations": interpretations,
            })
        }
        Err(error) => json!({ "input": input, "error": format!("{error:?}") }),
    }
}

fn main() {
    let corpus: Value = serde_json::from_str(include_str!("../../../../../parity/corpus.json"))
        .expect("parity/corpus.json is JSON");
    let parser = ExpressionParser::new();
    let vectors: Vec<Value> = corpus["inputs"]
        .as_array()
        .expect("inputs")
        .iter()
        .map(|input| parse(&parser, input.as_str().expect("string input")))
        .collect();
    println!("{}", serde_json::to_string_pretty(&vectors).expect("JSON"));
}
