//! Test script for issue #21 - expression (1/3)*3 returning 0.9999...
//!
//! This example verifies the issue and tests different expressions.
//!
//! Run with: `cargo run --example test_issue_21`

use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    println!("=== Issue #21: Expression (1/3)*3 Test ===\n");

    // The problematic expression
    let test_cases = [
        "(1/3)*3", "1/3", "(1/3)", "1/3*3", "(2/3)*3", "(1/6)*6", "(1/7)*7", "1 + 2", "2 * 3",
    ];

    for expr in test_cases {
        println!("Expression: {expr}");
        let result = calc.calculate_internal(expr);
        if result.success {
            println!("  Result: {}", result.result);
            println!("  Links notation: {}", result.lino_interpretation);

            // Show fraction if available
            if let Some(ref frac) = result.fraction {
                println!("  Fraction: {frac}");
            }

            // Show repeating decimal notations if available
            if let Some(ref rd) = result.repeating_decimal {
                println!("  Repeating decimal notations:");
                println!("    Vinculum: {}", rd.vinculum);
                println!("    Parenthesis: {}", rd.parenthesis);
                println!("    Ellipsis: {}", rd.ellipsis);
                println!("    LaTeX: {}", rd.latex);
                println!("    Fraction: {}", rd.fraction);
            }

            println!("  Steps:");
            for step in &result.steps {
                println!("    {step}");
            }
        } else {
            println!("  Error: {}", result.error.unwrap_or_default());
        }
        println!();
    }
}
