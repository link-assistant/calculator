//! Exact percentage calculations using the same API as the CLI and WASM.
use link_calculator::Calculator;

fn main() {
    let mut calculator = Calculator::new();
    for expression in [
        "200 + 10%",
        "100 + 10% + 10%",
        "10% + 20%",
        "180 is what % off 200",
        "700 увеличить на 30 процентов",
        "700的百分之三十",
        "700の30パーセント",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(result.success, "{expression}: {:?}", result.error);
        println!("{expression} = {}", result.result);
    }
}
