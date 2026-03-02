//! Test issue #23: until with 'on' keyword and natural date expressions
//!
//! Run with: cargo run --example test_issue23

use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();
    
    let inputs = [
        "until 11:59pm EST on Monday, January 26th",
        "11:59pm EST on Monday, January 26th",
        "until Jan 27, 11:59pm UTC",
        "11:59pm EST January 26th",
        "11:59pm EST on January 26",
        "Jan 1, 12:00am UTC",  // past date for comparison
    ];
    
    for input in &inputs {
        let result = calc.calculate_internal(input);
        println!("Input: '{}'", input);
        println!("Success: {}", result.success);
        println!("Result: '{}'", result.result);
        if let Some(e) = &result.error {
            println!("Error: {}", e);
        }
        println!("Lino: '{}'", result.lino_interpretation);
        println!("Steps:");
        for step in &result.steps {
            println!("  {}", step);
        }
        println!("---");
    }
}
