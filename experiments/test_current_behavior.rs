// Test current behavior of expressions
// Run with: cargo test --test integration_test test_inline
use link_calculator::Calculator;

fn test(expr: &str) {
    let mut calc = Calculator::new();
    let r = calc.calculate_internal(expr);
    println!("  '{}' -> success={}, result='{}', error={:?}", 
             expr, r.success, r.result, r.error);
}

fn main() {
    println!("=== Current Behavior Tests ===");
    test("100 USD");
    test("100 USD in EUR");  // Does 'in' work as unit conversion?
    test("19 TON");          // TON as crypto currency?
    test("19 ton");          // ton as unit? (ambiguous)
    test("10 tons to kg");   // mass conversion
}
