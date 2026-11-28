//! Named constants and their locale-independent input aliases.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::arithmetic_words::{is_cjk, starts_cjk_operator};
use super::lexer::is_identifier_continue;
use crate::types::{Decimal, PhysicalUnit, Unit};

pub(super) struct Constant {
    pub name: &'static str,
    amount: &'static str,
    units: &'static [(&'static str, i32)],
}

impl Constant {
    pub fn amount(&self) -> Decimal {
        self.amount.parse().expect("valid constant amount")
    }

    pub const fn is_scalar(&self) -> bool {
        self.units.is_empty()
    }

    pub fn unit(&self) -> Unit {
        PhysicalUnit::from_terms(
            self.units
                .iter()
                .map(|&(symbol, power)| (symbol.to_owned(), power))
                .collect(),
        )
        .into_unit()
    }
}

const CONSTANTS: &[Constant] = &[
    Constant {
        name: "golden_ratio",
        amount: "1.6180339887498948482045868344",
        units: &[],
    },
    Constant {
        name: "silver_ratio",
        amount: "2.4142135623730950488016887242",
        units: &[],
    },
    Constant {
        name: "pi",
        amount: "3.141592653589793",
        units: &[],
    },
    Constant {
        name: "tau",
        amount: "6.283185307179586",
        units: &[],
    },
    Constant {
        name: "e",
        amount: "2.718281828459045",
        units: &[],
    },
    Constant {
        name: "euler_mascheroni",
        amount: "0.5772156649015328606065120901",
        units: &[],
    },
    // Exact SI defining values, in m/s and mol^-1 respectively.
    Constant {
        name: "speed_of_light",
        amount: "299792458",
        units: &[("m", 1), ("s", -1)],
    },
    Constant {
        name: "avogadro_constant",
        amount: "602214076000000000000000",
        units: &[("mol", -1)],
    },
];

pub(super) fn by_name(name: &str) -> Option<&'static Constant> {
    CONSTANTS
        .iter()
        .find(|constant| name.eq_ignore_ascii_case(constant.name))
}

struct Alias {
    pattern: Vec<char>,
    constant: &'static Constant,
}

const ALIASES_LINO: &str = include_str!("../../data/words/named-constants.lino");

fn aliases() -> &'static HashMap<char, Vec<Alias>> {
    static ALIASES: OnceLock<HashMap<char, Vec<Alias>>> = OnceLock::new();
    ALIASES.get_or_init(|| {
        let mut aliases = HashMap::<_, Vec<Alias>>::new();
        for line in ALIASES_LINO.lines().map(str::trim) {
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let (head, surface) = line
                .strip_prefix("(constant ")
                .and_then(|body| body.strip_suffix("')"))
                .and_then(|body| body.split_once(" '"))
                .unwrap_or_else(|| panic!("malformed constant alias: {line}"));
            let (_, name) = head
                .split_once(' ')
                .unwrap_or_else(|| panic!("missing constant name: {line}"));
            let constant = by_name(name).unwrap_or_else(|| panic!("unknown constant: {line}"));
            let pattern: Vec<_> = surface.chars().map(match_char).collect();
            let first = *pattern.first().expect("nonempty constant alias");
            let entries = aliases.entry(first).or_default();
            if let Some(existing) = entries.iter().find(|alias| alias.pattern == pattern) {
                assert_eq!(
                    existing.constant.name, name,
                    "ambiguous constant alias: {line}"
                );
                continue;
            }
            entries.push(Alias { pattern, constant });
        }
        for entries in aliases.values_mut() {
            entries.sort_by_key(|alias| std::cmp::Reverse(alias.pattern.len()));
        }
        aliases
    })
}

fn match_char(ch: char) -> char {
    match ch {
        '’' | 'ʼ' => '\'',
        '‐' | '‑' | '–' => '-',
        'İ' | 'ı' => 'i',
        _ => ch.to_lowercase().next().unwrap_or(ch),
    }
}

/// Finds the longest whole alias at a word start and returns its original end.
pub(super) fn scan(input: &[char], start: usize) -> Option<(&'static Constant, usize)> {
    let candidates = aliases().get(&match_char(*input.get(start)?))?;
    candidates.iter().find_map(|alias| {
        let mut end = start;
        for &expected in &alias.pattern {
            let current = *input.get(end)?;
            if expected == ' ' && current.is_whitespace() {
                while input.get(end).is_some_and(|ch| ch.is_whitespace()) {
                    end += 1;
                }
            } else if match_char(current) == expected {
                end += 1;
            } else {
                return None;
            }
        }
        let ends_at_boundary = !input.get(end).copied().is_some_and(is_identifier_continue);
        (ends_at_boundary || (is_cjk(input[end - 1]) && starts_cjk_operator(input, end)))
            .then_some((alias.constant, end))
    })
}
