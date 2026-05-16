//! Library consumer example modelled on the downstream `formal-ai` use case.
//!
//! Demonstrates that a Rust consumer of `link-calculator` can drive the
//! calculator end-to-end *and* recover every artefact in a structured form:
//!
//! 1. Parse an expression into an [`Expression`] AST.
//! 2. Plan the calculation to learn which rate sources are required.
//! 3. Evaluate the AST while recording per-step explanations.
//! 4. Re-encode every artefact (input, steps, final value) as Links Notation.
//! 5. Replay the calculation by walking the AST and using the public
//!    `apply_binary_op` primitive directly.
//!
//! Run with: `cargo run --example library_consumer`

use link_calculator::grammar::ExpressionParser;
use link_calculator::types::{BinaryOp, Expression, Value};
use link_calculator::{generate_issue_link, Calculator};

fn main() {
    println!("link-calculator v{}\n", link_calculator::VERSION);

    high_level_round_trip();
    parser_driven_round_trip();
    manual_reconstruction();
    failure_round_trip();
}

/// Demonstrates [`Calculator::calculate_with_value`] — the single call that
/// returns every artefact a downstream consumer needs.
fn high_level_round_trip() {
    println!("--- high-level round trip (calculate_with_value) ---");
    let mut calculator = Calculator::new();
    let input = "(2 + 3) * 4";

    let (expression, value, steps, lino) = calculator
        .calculate_with_value(input)
        .expect("expression parses and evaluates");

    println!("input              : {input}");
    println!("AST (Debug)        : {expression:?}");
    println!("Links Notation     : {lino}");
    println!(
        "structured value   : {} (kind: {:?})",
        value.to_display_string(),
        value.kind
    );
    println!("steps captured     : {}", steps.len());
    for (i, step) in steps.iter().enumerate() {
        println!("  step {i:>2}: {step}");
    }

    // Re-encode the parsed AST back as LINO — this proves that the structured
    // value the caller received is the same one that produced the original
    // representation, so nothing was lost in translation.
    let reencoded = expression.to_lino();
    println!("reencoded LINO     : {reencoded}");
    assert_eq!(reencoded, lino);
    println!();
}

/// Drives [`ExpressionParser`] directly. This is the path consumers take when
/// they already have a parsed [`Expression`] and want to evaluate it without
/// re-parsing.
fn parser_driven_round_trip() {
    println!("--- parser-driven round trip ---");
    let mut parser = ExpressionParser::new();
    let input = "100 USD + 50 USD";

    let expression = parser.parse(input).expect("parses");
    let lino = expression.to_lino();

    let (value, steps) = parser.evaluate_with_steps(&expression).expect("evaluates");

    println!("input              : {input}");
    println!("Links Notation     : {lino}");
    println!("value              : {}", value.to_display_string());
    println!(
        "first step         : {}",
        steps.first().cloned().unwrap_or_default()
    );
    println!(
        "last step          : {}",
        steps.last().cloned().unwrap_or_default()
    );
    println!();
}

/// Reconstructs a calculation by walking the AST manually and reusing the
/// calculator's public primitives ([`ExpressionParser::evaluate_expr`] and
/// [`ExpressionParser::apply_binary_op`]). This is the pattern formal-ai uses
/// when it wants to inject custom behaviour around each operator.
fn manual_reconstruction() {
    println!("--- manual reconstruction with apply_binary_op ---");
    let mut parser = ExpressionParser::new();
    let expr = parser.parse("8 + 4").expect("parses");

    let (left, op, right) = match &expr {
        Expression::Binary { left, op, right } => (left, *op, right),
        _ => panic!("expected a binary expression"),
    };
    assert_eq!(op, BinaryOp::Add);

    let left_value: Value = parser.evaluate_expr(left).expect("evaluates lhs");
    let right_value: Value = parser.evaluate_expr(right).expect("evaluates rhs");
    let total: Value = parser
        .apply_binary_op(&left_value, op, &right_value)
        .expect("operator applies");

    println!("lhs                : {}", left_value.to_display_string());
    println!("rhs                : {}", right_value.to_display_string());
    println!("reconstructed sum  : {}", total.to_display_string());
    println!();
}

/// Demonstrates that even the failure path stays exportable: downstream code
/// can render the same GitHub issue link the calculator uses internally.
fn failure_round_trip() {
    println!("--- failure round trip ---");
    let mut calculator = Calculator::new();
    let bad = "???not-an-expression???";
    let err = calculator
        .calculate_with_value(bad)
        .expect_err("should fail to parse");
    let link = generate_issue_link(bad, &err.to_string());
    println!("input              : {bad}");
    println!("error              : {err}");
    println!("issue link prefix  : {}", &link[..link.len().min(60)]);
}
