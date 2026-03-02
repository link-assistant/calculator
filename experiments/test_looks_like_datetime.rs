//! Test DateTimeGrammar::looks_like_datetime for various inputs
use link_calculator::Calculator;

fn main() {
    // We can't directly access DateTimeGrammar from outside, so let's use Calculator
    let mut calc = Calculator::new();
    
    // Test various inputs
    let inputs = [
        "11:59pm EST on Monday, January 26th",
        "11:59pm",
        "11",
        "11 : 59 pm EST on Monday , January 26th",
    ];
    
    for input in &inputs {
        let result = calc.calculate_internal(input);
        println!("Input: '{}'", input);
        println!("Success: {}", result.success);
        println!("Result: '{}'", result.result);
        if let Some(e) = &result.error {
            println!("Error: {}", e);
        }
        println!("---");
    }
}
