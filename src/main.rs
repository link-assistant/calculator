//! Link Calculator CLI - A command-line interface for the calculator.

use link_calculator::Calculator;
use std::io::{self, BufRead, Write};

fn main() {
    let mut calculator = Calculator::new();

    // One-shot mode: `link-calculator "35usd - 200cny - 1hkd"` evaluates the
    // arguments as a single expression and exits with a non-zero code on error.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        let success = evaluate_and_print(&mut calculator, &args.join(" "));
        std::process::exit(i32::from(!success));
    }

    println!("Link Calculator v{}", link_calculator::VERSION);
    println!("Type expressions to calculate, or 'quit' to exit.\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().expect("Failed to flush stdout");

        let mut line = String::new();
        // `Ok(0)` means end of input (e.g. piped stdin or Ctrl+D): stop instead
        // of looping forever on an empty read.
        let read = stdin.lock().read_line(&mut line);
        match read {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }

        evaluate_and_print(&mut calculator, input);
        println!();
    }
}

/// Evaluates `input` and prints the result (or error); returns whether it succeeded.
fn evaluate_and_print(calculator: &mut Calculator, input: &str) -> bool {
    let result = calculator.calculate_internal(input);

    if result.success {
        println!("Result: {}", result.result);
        println!("Links notation: {}", result.lino_interpretation);

        if !result.steps.is_empty() {
            println!("\nSteps:");
            for step in &result.steps {
                println!("  {step}");
            }
        }
    } else {
        println!("Error: {}", result.error.unwrap_or_default());
        if let Some(link) = result.issue_link {
            println!("\nReport this issue: {link}");
        }
    }
    result.success
}

fn print_help() {
    println!(
        r"
Link Calculator - Help

Basic Operations:
  2 + 3              Addition
  10 - 4             Subtraction
  3 * 4              Multiplication
  15 / 3             Division
  (2 + 3) * 4        Parentheses for grouping

Numbers with Units:
  84 USD             Currency amounts
  34 EUR
  100 USD + 50 EUR   Currency arithmetic (auto-conversion)

DateTime Operations:
  (Jan 27, 8:59am UTC) - (Jan 25, 12:51pm UTC)
                     Calculate time difference

Temporal Context:
  84 USD - 34 EUR at 22 Jan 2026
                     Use historical exchange rates

Commands:
  help               Show this help
  quit               Exit the calculator
"
    );
}
