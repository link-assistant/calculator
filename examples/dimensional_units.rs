//! Dimensional unit algebra (issue #235) using the same API as the CLI and WASM.
use link_calculator::Calculator;

fn main() {
    let mut calculator = Calculator::new();
    for expression in [
        "10m × 10m",
        "(5 volts) / (2 ohms)",
        "1 mph * 1 hour",
        "9.81 m/s² * 70 kg in N",
        "299792458 m⋅s−1",
        "1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)",
        "20 celsius",
        "25 °C in °F",
        "5 µm in nm",
        "5 feet 10 inches in cm",
        "6'2\" in cm",
        "1 light year in miles",
        "5m in s",
    ] {
        let result = calculator.calculate_internal(expression);
        assert!(result.success, "{expression}: {:?}", result.error);
        println!("{expression} = {}", result.result);
    }
    for expression in ["sqrt(2m)", "25 °C * 2", "1 kg + 1 m"] {
        let result = calculator.calculate_internal(expression);
        assert!(!result.success, "{expression} = {}", result.result);
        println!("{expression}: {}", result.error.unwrap_or_default());
    }
}
