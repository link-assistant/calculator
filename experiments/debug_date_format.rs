use link_calculator::types::DateTime;

fn main() {
    // Test what dates various formats produce
    let test_dates = [
        "Feb 8, 2021",
        "2021-02-08",
        "February 8, 2021",
        "8 Feb 2021",
    ];

    for date_str in test_dates {
        match DateTime::parse(date_str) {
            Ok(dt) => {
                let formatted = dt.as_chrono().format("%Y-%m-%d").to_string();
                println!("Input: '{}' -> Parsed chrono format: '{}'", date_str, formatted);
            }
            Err(e) => {
                println!("Input: '{}' -> Error: {:?}", date_str, e);
            }
        }
    }
}
