//! Basic usage example for Link Calculator.
//!
//! This example demonstrates the basic functionality of the calculator.
//!
//! Run with: `cargo run --example basic_usage`

use link_calculator::Calculator;

fn main() {
    let calculator = Calculator::new();

    println!("Link Calculator v{}\n", link_calculator::VERSION);

    // Example 1: Basic arithmetic
    println!("Example 1: Basic arithmetic");
    show_calculation(&calculator, "2 + 3");
    show_calculation(&calculator, "10 - 4");
    show_calculation(&calculator, "3 * 4");
    show_calculation(&calculator, "15 / 3");
    println!();

    // Example 2: Decimal numbers
    println!("Example 2: Decimal numbers");
    show_calculation(&calculator, "3.14 + 2.86");
    show_calculation(&calculator, "10.5 * 2");
    println!();

    // Example 3: Operator precedence
    println!("Example 3: Operator precedence");
    show_calculation(&calculator, "2 + 3 * 4");
    show_calculation(&calculator, "(2 + 3) * 4");
    println!();

    // Example 4: Negative numbers
    println!("Example 4: Negative numbers");
    show_calculation(&calculator, "-5 + 3");
    show_calculation(&calculator, "10 - -5");
    println!();

    // Example 5: Currency operations
    println!("Example 5: Currency operations");
    show_calculation(&calculator, "100 USD");
    show_calculation(&calculator, "84 USD - 34 EUR");
    println!();

    // Example 6: DateTime subtraction
    println!("Example 6: DateTime subtraction");
    show_calculation(&calculator, "(Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)");
}

fn show_calculation(calculator: &Calculator, input: &str) {
    let result = calculator.calculate_internal(input);
    if result.success {
        println!("  {} = {}", input, result.result);
        println!("    (lino: {})", result.lino_interpretation);
    } else {
        println!("  {} => Error: {}", input, result.error.unwrap_or_default());
    }
}
