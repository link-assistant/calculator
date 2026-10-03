//! Build script: bundles the latest exchange rate of every
//! `data/currency/*.lino` file into the library (issue #224).
//!
//! A fresh `Calculator` used to know only a handful of hardcoded fallback
//! rates, so currencies such as HKD failed with "No exchange rate available"
//! until the live ECB fetch succeeded. The repository already keeps daily
//! central-bank rates in `data/currency/`; this script extracts the most recent
//! row of each file and generates `$OUT_DIR/bundled_rates.rs`, which
//! `CurrencyDatabase::new()` uses for currencies without any fallback rate.
//!
//! When the data directory is absent the generated table is simply empty.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// The most recent rate found in a consolidated `.lino` rate file.
struct LatestRate {
    from: String,
    to: String,
    source: String,
    date: String,
    rate: f64,
}

/// Parses the consolidated format:
///
/// ```text
/// conversion:
///   from USD
///   to HKD
///   source 'frankfurter.dev (ECB)'
///   rates:
///     2026-10-02 7.8471
/// ```
fn parse_latest_rate(content: &str) -> Option<LatestRate> {
    let mut from = None;
    let mut to = None;
    let mut source = String::from("unknown");
    let mut latest: Option<(String, f64)> = None;
    let mut in_rates = false;

    for line in content.lines() {
        let line = line.trim();
        if line == "rates:" || line == "data:" {
            in_rates = true;
            continue;
        }
        if in_rates {
            let mut parts = line.split_whitespace();
            if let (Some(date), Some(value)) = (parts.next(), parts.next()) {
                if let Ok(rate) = value.parse::<f64>() {
                    let is_newer = latest.as_ref().map_or(true, |(d, _)| date > d.as_str());
                    if rate.is_finite() && rate > 0.0 && is_newer {
                        latest = Some((date.to_string(), rate));
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix("from ") {
            from = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = line.strip_prefix("to ") {
            to = Some(rest.trim().to_uppercase());
        } else if let Some(rest) = line.strip_prefix("source ") {
            source = rest
                .trim()
                .trim_matches(|c| c == '\'' || c == '"')
                .to_string();
        }
    }

    let (date, rate) = latest?;
    Some(LatestRate {
        from: from?,
        to: to?,
        source,
        date,
        rate,
    })
}

fn main() {
    let data_dir = Path::new("data/currency");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", data_dir.display());

    let mut rates = Vec::new();
    if let Ok(entries) = fs::read_dir(data_dir) {
        let mut paths: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "lino"))
            .collect();
        paths.sort();
        for path in paths {
            println!("cargo:rerun-if-changed={}", path.display());
            if let Some(rate) = fs::read_to_string(&path)
                .ok()
                .and_then(|content| parse_latest_rate(&content))
            {
                rates.push(rate);
            }
        }
    }

    let mut out = String::from(
        "/// Latest rates bundled from `data/currency/*.lino`: \
         `(from, to, source, date, rate)`.\n\
         pub static BUNDLED_LATEST_RATES: &[(&str, &str, &str, &str, f64)] = &[\n",
    );
    for r in &rates {
        writeln!(
            out,
            "    ({:?}, {:?}, {:?}, {:?}, {:?}_f64),",
            r.from, r.to, r.source, r.date, r.rate
        )
        .expect("writing to a String cannot fail");
    }
    out.push_str("];\n");

    let out_dir = env::var("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
    fs::write(Path::new(&out_dir).join("bundled_rates.rs"), out)
        .expect("failed to write bundled_rates.rs");
}
