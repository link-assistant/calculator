// Test the lino parsing logic

fn main() {
    let content = "conversion:
  from USD
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  rates:
    2021-02-08 74.2602
    2021-02-09 74.1192
    2026-01-25 76.03";

    let mut from_currency: Option<String> = None;
    let mut to_currency: Option<String> = None;
    let mut source: Option<String> = None;
    let mut in_data_section = false;
    let mut loaded = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        println!("Line: '{}', trimmed: '{}', in_data: {}", line, trimmed, in_data_section);

        // Skip empty lines and root markers (both new 'conversion:' and legacy 'rates:')
        if trimmed.is_empty() || trimmed == "conversion:" || trimmed == "rates:" {
            println!("  -> Skipping (empty/root marker)");
            continue;
        }

        // Check for data section marker (both new 'rates:' and legacy 'data:')
        // Only treat 'rates:' as data section if we already have from/to currencies parsed
        if trimmed == "data:" || (trimmed == "rates:" && from_currency.is_some()) {
            println!("  -> Entering data section");
            in_data_section = true;
            continue;
        }

        if in_data_section {
            // Parse date and value: "2021-01-25 0.8234"
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            println!("  -> Data section, parts: {:?}", parts);
            if parts.len() >= 2 {
                if let (Some(from), Some(to)) = (from_currency.as_ref(), to_currency.as_ref()) {
                    let date = parts[0];
                    if let Ok(value) = parts[1].parse::<f64>() {
                        println!("  -> Loaded rate: {}/{} @ {} = {}", from, to, date, value);
                        loaded += 1;
                    }
                }
            }
        } else {
            // Parse header section
            if let Some(rest) = trimmed.strip_prefix("from ") {
                from_currency = Some(rest.trim().to_uppercase());
                println!("  -> Found from: {:?}", from_currency);
            } else if let Some(rest) = trimmed.strip_prefix("to ") {
                to_currency = Some(rest.trim().to_uppercase());
                println!("  -> Found to: {:?}", to_currency);
            } else if let Some(rest) = trimmed.strip_prefix("source ") {
                let src = rest.trim();
                let src = src.trim_start_matches('\'').trim_end_matches('\'');
                let src = src.trim_start_matches('"').trim_end_matches('"');
                source = Some(src.to_string());
                println!("  -> Found source: {:?}", source);
            }
        }
    }

    println!("\nTotal loaded: {}", loaded);
}
