// Experiment script to test issue #75: Russian currency conversion
// Run with: cargo script experiments/test_issue_75.rs (or add as a main.rs temporarily)
//
// Tests:
// - "1000 рублей в долларах" should be recognized as conversion from RUB to USD
// - "1000 рублей в евро" should convert RUB to EUR
// - "1000 рублей в юанях" should convert RUB to CNY

use link_calculator::Calculator;

fn main() {
    let test_cases = vec![
        "1000 рублей в долларах",
        "1000 рублей в евро",
        "1000 рублей в фунтах",
        "1000 рублей в юанях",
        "100 долларов в рублях",
        "100 евро в долларах",
        "1000 рублей in USD",
        "1000 RUB в долларах",
    ];

    let mut calc = Calculator::new();
    for input in &test_cases {
        let result = calc.calculate_internal(input);
        println!("Input: {}", input);
        println!("  Success: {}", result.success);
        println!("  Result: {}", result.result);
        if let Some(err) = &result.error {
            println!("  Error: {}", err);
        }
        println!();
    }
}
