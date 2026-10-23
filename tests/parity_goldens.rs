//! Parity fixtures for the JavaScript engine (issue #254).
//!
//! Evaluates every input of `parity/corpus.json` in each of its scenarios, at
//! the fixed instant `clock`, with the exchange rates of `data/currency/*.lino`
//! dated on or before `ratesCutoff`. The results are compared with
//! `parity/goldens/<scenario>.json`, which the JavaScript engine is checked
//! against in turn (`js/test/parity.test.js`).
//!
//! - `default.json` holds every case.
//! - The other scenarios hold only the cases whose plan or result differs from
//!   `default`.
//!
//! Regenerate after an intended behavior change:
//!
//! ```bash
//! UPDATE_PARITY_GOLDENS=1 node scripts/bounded-cargo.mjs test --test parity_goldens
//! ```
//!
//! `PARITY_VERBOSE=1` prints each input before it is evaluated.

use chrono::{DateTime, Utc};
use link_calculator::types::CurrencyDatabase;
use link_calculator::Calculator;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A `data/currency/*.lino` file without the rows dated after the cutoff.
struct RateFile {
    from: String,
    to: String,
    source: String,
    /// `(date, rate)` in file order.
    rows: Vec<(String, f64)>,
    /// The truncated file text, as the web worker would fetch it.
    content: String,
}

impl RateFile {
    /// The latest positive rate, chosen the way `build.rs` chooses it.
    fn latest(&self) -> Option<(&str, f64)> {
        let mut latest: Option<(&str, f64)> = None;
        for (date, rate) in &self.rows {
            let is_newer = latest.map_or(true, |(d, _)| date.as_str() > d);
            if rate.is_finite() && *rate > 0.0 && is_newer {
                latest = Some((date, *rate));
            }
        }
        latest
    }
}

fn read_rate_files(cutoff: &str) -> Vec<RateFile> {
    let mut paths: Vec<PathBuf> = fs::read_dir(root().join("data/currency"))
        .expect("data/currency exists")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "lino"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).expect("rate file is UTF-8");
            let mut file = RateFile {
                from: String::new(),
                to: String::new(),
                source: String::from("unknown"),
                rows: Vec::new(),
                content: String::new(),
            };
            let mut in_rates = false;
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed == "data:" || (trimmed == "rates:" && !file.from.is_empty()) {
                    in_rates = true;
                } else if in_rates {
                    let mut parts = trimmed.split_whitespace();
                    if let (Some(date), Some(value)) = (parts.next(), parts.next()) {
                        if date > cutoff {
                            continue;
                        }
                        if let Ok(rate) = value.parse::<f64>() {
                            file.rows.push((date.to_string(), rate));
                        }
                    }
                } else if let Some(rest) = trimmed.strip_prefix("from ") {
                    file.from = rest.trim().to_uppercase();
                } else if let Some(rest) = trimmed.strip_prefix("to ") {
                    file.to = rest.trim().to_uppercase();
                } else if let Some(rest) = trimmed.strip_prefix("source ") {
                    file.source = rest
                        .trim()
                        .trim_matches(|c| c == '\'' || c == '"')
                        .to_string();
                }
                file.content.push_str(line);
                file.content.push('\n');
            }
            file
        })
        .collect()
}

/// The newest date and the JSON object of latest rates of the files `pick`
/// keys (by the other currency, in lowercase), as a rate API would return them.
fn latest_rates(
    files: &[RateFile],
    pick: impl Fn(&RateFile) -> Option<String>,
) -> (String, String) {
    let mut rates = BTreeMap::new();
    let mut newest = String::new();
    for file in files {
        if let (Some(key), Some((date, rate))) = (pick(file), file.latest()) {
            rates.insert(key, rate);
            if date > newest.as_str() {
                newest = date.to_string();
            }
        }
    }
    (
        newest,
        serde_json::to_string(&rates).expect("rates serialize"),
    )
}

fn new_calculator(files: &[RateFile], scenario: &Value) -> Calculator {
    let latest: Vec<(&str, &str, &str, &str, f64)> = files
        .iter()
        .filter_map(|file| {
            let (date, rate) = file.latest()?;
            Some((
                file.from.as_str(),
                file.to.as_str(),
                file.source.as_str(),
                date,
                rate,
            ))
        })
        .collect();
    let mut calculator = Calculator::new();
    *calculator.parser_mut().currency_db_mut() = CurrencyDatabase::with_bundled_rates(&latest);

    if let Some(offset) = scenario["timezoneOffsetMinutes"].as_i64() {
        calculator.set_timezone_offset(i32::try_from(offset).expect("offset fits i32"));
    }
    if scenario["history"].as_bool() == Some(true) {
        for file in files {
            calculator.load_rates_from_consolidated_lino(&file.content);
        }
    }
    if let Some(base) = scenario["ecbBase"].as_str() {
        let (date, rates) = latest_rates(files, |file| {
            (file.from == base).then(|| file.to.to_lowercase())
        });
        calculator.update_rates_from_api(base, &date, &rates);
    }
    if scenario["cbr"].as_bool() == Some(true) {
        let (date, rates) = latest_rates(files, |file| {
            (file.to == "RUB").then(|| file.from.to_lowercase())
        });
        calculator.update_cbr_rates_from_api(&date, &rates);
    }
    if let Some(crypto) = scenario.get("crypto") {
        let base = crypto["base"].as_str().expect("crypto.base");
        let date = crypto["date"].as_str().expect("crypto.date");
        calculator.update_crypto_rates_from_api(base, date, &crypto["rates"].to_string());
    }
    calculator
}

/// Sorts object keys, so `HashMap` iteration order does not show up in the
/// fixtures.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, canonical(v))).collect();
            Value::Object(sorted.into_iter().collect::<Map<String, Value>>())
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonical).collect()),
        other => other,
    }
}

fn parse_output(text: &str) -> Value {
    canonical(serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string())))
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic".to_string())
}

/// Evaluates every input with one calculator, as the web worker does. A panic
/// is recorded as `{"panic": message}` and the calculator is rebuilt.
fn evaluate(files: &[RateFile], scenario: &Value, inputs: &[String]) -> Vec<(Value, Value)> {
    let verbose = std::env::var("PARITY_VERBOSE").as_deref() == Ok("1");
    let mut calculator = new_calculator(files, scenario);
    let mut cases = Vec::with_capacity(inputs.len());
    for input in inputs {
        if verbose {
            eprintln!("parity: {input:?}");
        }
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let plan = calculator.plan(input);
            let result = calculator.calculate(input);
            (parse_output(&plan), parse_output(&result))
        }));
        let case = outcome.unwrap_or_else(|payload| {
            calculator = new_calculator(files, scenario);
            let mut panic = Map::new();
            panic.insert(
                "panic".into(),
                Value::String(panic_message(payload.as_ref())),
            );
            (Value::Object(panic.clone()), Value::Object(panic))
        });
        cases.push(case);
    }
    cases
}

fn render(corpus: &Value, scenario: &str, cases: &[(&String, &Value, &Value)]) -> String {
    let mut text = String::from("{\n");
    for key in ["clock", "ratesCutoff"] {
        writeln!(text, "  {:?}: {},", key, corpus[key]).expect("write to String");
    }
    writeln!(text, "  \"scenario\": {},", Value::String(scenario.into())).expect("write");
    text.push_str("  \"cases\": [");
    for (index, (input, plan, result)) in cases.iter().enumerate() {
        let mut case = Map::new();
        case.insert("input".into(), Value::String((*input).clone()));
        case.insert("plan".into(), (*plan).clone());
        case.insert("result".into(), (*result).clone());
        text.push_str(if index == 0 { "\n    " } else { ",\n    " });
        text.push_str(&serde_json::to_string(&Value::Object(case)).expect("case serializes"));
    }
    text.push_str(if cases.is_empty() {
        "]\n}\n"
    } else {
        "\n  ]\n}\n"
    });
    text
}

fn first_differences(expected: &str, actual: &str) -> Vec<String> {
    let expected: Vec<&str> = expected.lines().collect();
    actual
        .lines()
        .enumerate()
        .filter(|(index, line)| expected.get(*index) != Some(line))
        .take(5)
        .map(|(index, line)| {
            format!(
                "line {}:\n  expected {}\n  actual   {}",
                index + 1,
                expected.get(index).unwrap_or(&"<missing>"),
                line
            )
        })
        .collect()
}

#[test]
fn parity_goldens_match_the_rust_engine() {
    let corpus: Value = serde_json::from_str(
        &fs::read_to_string(root().join("parity/corpus.json")).expect("parity/corpus.json"),
    )
    .expect("corpus is JSON");
    let clock: DateTime<Utc> = corpus["clock"]
        .as_str()
        .and_then(|text| text.parse().ok())
        .expect("corpus.clock is an RFC 3339 instant");
    let cutoff = corpus["ratesCutoff"].as_str().expect("corpus.ratesCutoff");
    let inputs: Vec<String> = serde_json::from_value(corpus["inputs"].clone()).expect("inputs");
    let files = read_rate_files(cutoff);
    let update = std::env::var("UPDATE_PARITY_GOLDENS").as_deref() == Ok("1");

    link_calculator::clock::set_fixed_now(Some(clock));
    let scenarios = corpus["scenarios"].as_object().expect("scenarios");
    let default = evaluate(&files, &scenarios["default"], &inputs);
    let mut stale = Vec::new();
    for (name, scenario) in scenarios {
        let results = (name != "default").then(|| evaluate(&files, scenario, &inputs));
        let cases: Vec<(&String, &Value, &Value)> = results.as_ref().map_or_else(
            || {
                inputs
                    .iter()
                    .zip(&default)
                    .map(|(i, (p, r))| (i, p, r))
                    .collect()
            },
            |results| {
                inputs
                    .iter()
                    .zip(&default)
                    .zip(results)
                    .filter(|((_, base), case)| *base != *case)
                    .map(|((input, _), (plan, result))| (input, plan, result))
                    .collect()
            },
        );
        let text = render(&corpus, name, &cases);
        let path = root().join("parity/goldens").join(format!("{name}.json"));
        if update {
            fs::create_dir_all(path.parent().expect("goldens directory")).expect("mkdir");
            fs::write(&path, &text).expect("write golden");
            eprintln!("wrote {} ({} cases)", relative(&path), cases.len());
        } else {
            let expected = fs::read_to_string(&path).unwrap_or_default();
            if expected != text {
                stale.push(format!(
                    "{} is out of date:\n{}",
                    relative(&path),
                    first_differences(&expected, &text).join("\n")
                ));
            }
        }
    }
    link_calculator::clock::set_fixed_now(None);
    assert!(
        stale.is_empty(),
        "{}\n\nIf the change is intended, regenerate the fixtures with\n  \
         UPDATE_PARITY_GOLDENS=1 node scripts/bounded-cargo.mjs test --test parity_goldens\n\
         and make the JavaScript engine (js/) match them.",
        stale.join("\n\n")
    );
}

fn relative(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .display()
        .to_string()
}
