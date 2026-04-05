//! Experiment script to reproduce issue #125:
//! "17 февраля 2027 - 6 месяцев" should be interpreted as "(17 февраля 2027) - 6 месяцев"
//! but instead it only produces "17 февраля" (truncated).

fn main() {
    // Demonstrate the root cause analysis of issue #125

    // Test 1: What tokens does the lexer produce for "17 февраля 2027 - 6 месяцев"?
    println!("=== Issue #125 Root Cause Analysis ===");
    println!();

    println!("Input: '17 февраля 2027 - 6 месяцев'");
    println!();

    println!("Step 1: Lexer tokenizes the input.");
    println!("  Token sequence: [Number(17), Ident(февраля), Number(2027), Minus(-), Number(6), Ident(месяцев), EOF]");
    println!();

    println!("Step 2: Token parser encounters Number(17), then Ident(февраля).");
    println!("  'февраля' is NOT in the DurationUnit::parse() table (only English).");
    println!("  'февраля' is NOT recognized as a month name by looks_like_datetime() (only English).");
    println!("  So 17 is parsed as a plain number, and февраля is an unknown identifier.");
    println!();

    println!("Wait: The number 17 is parsed. Then the parser looks at the next token:");
    println!("  - Ident(февраля) is seen as a 'unit' for number 17.");
    println!("  - 'февраля' does not parse as any known unit -> Unit::Custom(февраля).");
    println!("  - So '17 февраля' becomes Expression::Number {{ value: 17, unit: Custom(февраля) }}.");
    println!();

    println!("Step 3: After '17 февраля', the token stream has: [Number(2027), Minus(-), ...]");
    println!("  The parser is in parse_additive, and the next token Number(2027) cannot be an additive op.");
    println!("  But wait: how is '17 февраля' and then '2027' handled?");
    println!();

    println!("The key issue: '17 февраля' is parsed as a number with custom unit.");
    println!("Then '2027' is an unexpected number after a complete expression.");
    println!("But it's at the top level; the parser probably just ignores it or errors.");
    println!();

    println!("Actually, looking at parse_primary: Number(17) -> consumes Ident(февраля) as unit.");
    println!("After that we have Number(2027) -> this can't follow, so probably parse stops.");
    println!("Result: '17 февраля' (as a custom unit expression), and '2027 - 6 месяцев' is ignored.");
    println!();

    println!("The REAL issue: 'февраля' is a Russian genitive form of 'February' (февраль).");
    println!("The parser should recognize Russian month names for datetime parsing.");
    println!();

    println!("=== Required Fix ===");
    println!("1. Add Russian month names to looks_like_datetime() in DateTimeGrammar");
    println!("2. Add Russian month name normalization to DateTime::normalize_month_name() or a new function");
    println!("3. Add Russian duration unit names to DurationUnit::parse()");
    println!("   - месяцев / месяца / месяц = months");
    println!("   - недель / недели / неделя = weeks");
    println!("   - дней / дня / день = days");
    println!("   - часов / часа / час = hours");
    println!("   - минут / минуты / минута = minutes");
    println!("   - секунд / секунды / секунда = seconds");
    println!("   - лет / года / год = years");
    println!();

    println!("Also need: Russian month name to English translation for DateTime::parse().");
    println!("  январь/января -> January");
    println!("  февраль/февраля -> February");
    println!("  ... etc.");
}
