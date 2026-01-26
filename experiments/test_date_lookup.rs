use link_calculator::Calculator;
use link_calculator::types::DateTime;

fn main() {
    // First debug the date parsing
    println!("=== Date Parsing Debug ===");
    let test_dates = [
        "Feb 8, 2021",
        "2021-02-08",
    ];
    for date_str in test_dates {
        match DateTime::parse(date_str) {
            Ok(dt) => {
                let formatted = dt.as_chrono().format("%Y-%m-%d").to_string();
                println!("Input: '{}' -> chrono format: '{}'", date_str, formatted);
            }
            Err(e) => {
                println!("Input: '{}' -> Error: {:?}", date_str, e);
            }
        }
    }

    println!("\n=== Rate Loading and Lookup ===");
    let mut calculator = Calculator::new();

    let lino_content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602";

    let result = calculator.load_rates_from_consolidated_lino(lino_content);
    println!("Load result: {:?}", result);

    // Check using the direct database lookup (like the unit test does)
    println!("\n=== Direct Database Lookup ===");
    let date = DateTime::from_date(chrono::NaiveDate::from_ymd_opt(2021, 2, 8).unwrap());
    let date_str = date.as_chrono().format("%Y-%m-%d").to_string();
    println!("Created DateTime from_date: chrono format = '{}'", date_str);

    // Now test through calculate_internal
    println!("\n=== Calculate Internal Test ===");
    let calc_result = calculator.calculate_internal("(0 RUB + 1 USD) at Feb 8, 2021");
    println!("Success: {}", calc_result.success);
    println!("Result: {}", calc_result.result);
    println!("Steps:");
    for step in &calc_result.steps {
        println!("  {}", step);
    }
}
