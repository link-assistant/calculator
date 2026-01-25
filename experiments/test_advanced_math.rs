//! Experiment to test advanced math expression detection.

use link_calculator::Calculator;

fn main() {
    let calculator = Calculator::new();

    let test_inputs = [
        "integrate sin(x)/x dx",
        "differentiate x^2",
        "solve x^2 = 4",
        "limit x -> 0 sin(x)/x",
        "sin(45)",
        "sqrt(16)",
    ];

    for input in test_inputs {
        println!("\n--- Testing: {input} ---");
        let result = calculator.calculate_internal(input);
        println!("Success: {}", result.success);
        if let Some(error) = &result.error {
            println!("Error: {error}");
        }
        if let Some(link) = &result.issue_link {
            println!("Link: {link}");
        }
    }
}
