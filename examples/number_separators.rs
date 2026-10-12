//! Run with `cargo run --example number_separators`.
use link_calculator::Calculator;

fn main() {
    let mut calculator = Calculator::new();
    for input in [
        "3,000 minus 12",
        "1,000 divided by 200",
        "100,000 + 200,000",
        "56.7% of 1,234 participants",
        "1,1",
        "12,3 ± 4,5",
        "1,08 Mrd. km/h",
        "1'234.5",
        "12,34,567",
        "max(1,2)",
    ] {
        let result = calculator.calculate_internal(input);
        assert!(result.success, "{input}: {:?}", result.error);
        println!("{input} = {}", result.result);
    }
}
