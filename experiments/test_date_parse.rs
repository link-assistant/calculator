// Test if "Jan 25, 2021" is correctly parsed as a DateTime

use link_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();
    
    // Simple test: just parse the date
    let result = calc.calculate_internal("Jan 25, 2021");
    println!("Parse 'Jan 25, 2021': success={}, result={}, error={:?}", 
        result.success, result.result, result.error);
    
    // Test the actual expression
    let lino_content = "conversion:
  from USD
  to EUR
  source 'frankfurter.dev (ECB)'
  rates:
    2021-01-25 0.8234
    2021-02-01 0.8315";
    
    let load_result = calc.load_rates_from_consolidated_lino(lino_content);
    println!("Load rates: {:?}", load_result);
    
    let result2 = calc.calculate_internal("(0 EUR + 1 USD) at Jan 25, 2021");
    println!("Calculate: success={}, result={}, error={:?}",
        result2.success, result2.result, result2.error);
    println!("Steps:");
    for step in &result2.steps {
        println!("  {}", step);
    }
}
