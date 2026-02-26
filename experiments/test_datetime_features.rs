//! Test script for new datetime features (issues #34, #23, #68, #45)
use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    // Issue #34: "now" keyword
    println!("=== Issue #34: 'now' keyword ===");
    for input in &["now", "now UTC", "UTC now", "now EST"] {
        let result = calc.calculate_internal(input);
        println!("  Input: {}", input);
        println!("  Success: {}, Result: {}", result.success, result.result);
        if let Some(e) = &result.error {
            println!("  Error: {}", e);
        }
        println!();
    }

    // Issue #34: now in subtraction
    println!("=== Issue #34: 'now' in expressions ===");
    let result = calc.calculate_internal("(Jan 27, 8:59am UTC) - (now)");
    println!("  Input: (Jan 27, 8:59am UTC) - (now)");
    println!("  Success: {}, Result: {}", result.success, result.result);
    println!("  Steps: {:?}", result.steps);
    println!();

    // Issue #68: "UTC time"
    println!("=== Issue #68: 'UTC time' ===");
    for input in &["UTC time", "time UTC", "current time", "current UTC time"] {
        let result = calc.calculate_internal(input);
        println!("  Input: {}", input);
        println!("  Success: {}, Result: {}", result.success, result.result);
        if let Some(e) = &result.error {
            println!("  Error: {}", e);
        }
        println!();
    }

    // Issue #23: "until" keyword
    println!("=== Issue #23: 'until' keyword ===");
    let result = calc.calculate_internal("until Jan 27, 11:59pm UTC");
    println!("  Input: until Jan 27, 11:59pm UTC");
    println!("  Success: {}, Result: {}", result.success, result.result);
    println!("  Steps: {:?}", result.steps);
    println!();

    // Issue #23: timezone abbreviations
    println!("=== Issue #23: timezone abbreviations ===");
    for input in &["8:59am EST", "11:59pm PST", "2:30pm CET"] {
        let result = calc.calculate_internal(input);
        println!("  Input: {}", input);
        println!("  Success: {}, Result: {}", result.success, result.result);
        if let Some(e) = &result.error {
            println!("  Error: {}", e);
        }
        println!();
    }

    // Issue #23: ordinal dates
    println!("=== Issue #23: ordinal dates and day names ===");
    let result = calc.calculate_internal("until 11:59pm EST January 26th");
    println!("  Input: until 11:59pm EST January 26th");
    println!("  Success: {}, Result: {}", result.success, result.result);
    if let Some(e) = &result.error {
        println!("  Error: {}", e);
    }
    println!();

    // Issue #45: standalone datetime shows elapsed/remaining
    println!("=== Issue #45: standalone datetime ===");
    let result = calc.calculate_internal("Jan 27, 9:33am UTC");
    println!("  Input: Jan 27, 9:33am UTC");
    println!("  Success: {}, Result: {}", result.success, result.result);
    println!("  Lino: {}", result.lino_interpretation);
    println!("  Steps: {:?}", result.steps);
    println!();
}
