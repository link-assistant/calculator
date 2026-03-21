// Test for issue #91: UTC time display format
// Run with: cargo script experiments/test_issue_91.rs
use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();
    
    let inputs = [
        "UTC time",
        "current UTC time",
        "time UTC",
        "current time",
        "GMT time",
        "EST time",
        "PST time",
    ];
    
    for input in &inputs {
        let result = calc.calculate_internal(input);
        println!("Input: {:?}", input);
        println!("Result: {}", result.result);
        println!("Lino: {}", result.lino_interpretation);
        println!("is_live_time: {:?}", result.is_live_time);
        println!("---");
    }
}
