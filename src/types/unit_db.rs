//! The physical unit database (issue #235).
//!
//! Units live in `data/units/units.lino` (one link per unit, see the format
//! notes at the top of that file), so adding a unit means editing data, not
//! code. Each unit has a dimension, a factor to the coherent SI unit of that
//! dimension and, for `°C`/`°F`, an offset.

use std::collections::HashMap;
use std::sync::OnceLock;

use num_bigint::BigInt;

use super::dimension::Dimension;
use super::Rational;

/// The bundled unit table.
const UNITS_LINO: &str = include_str!("../../data/units/units.lino");

/// SI prefixes: symbol, word, power of ten. `u` and Greek `μ` are accepted
/// spellings of the micro sign `µ`.
const SI_PREFIXES: [(&str, &str, i32); 24] = [
    ("Q", "quetta", 30),
    ("R", "ronna", 27),
    ("Y", "yotta", 24),
    ("Z", "zetta", 21),
    ("E", "exa", 18),
    ("P", "peta", 15),
    ("T", "tera", 12),
    ("G", "giga", 9),
    ("M", "mega", 6),
    ("k", "kilo", 3),
    ("h", "hecto", 2),
    ("da", "deca", 1),
    ("d", "deci", -1),
    ("c", "centi", -2),
    ("m", "milli", -3),
    ("µ", "micro", -6),
    ("n", "nano", -9),
    ("p", "pico", -12),
    ("f", "femto", -15),
    ("a", "atto", -18),
    ("z", "zepto", -21),
    ("y", "yocto", -24),
    ("r", "ronto", -27),
    ("q", "quecto", -30),
];

/// Binary prefixes (IEC 80000-13): symbol, word, power of 1024.
const BINARY_PREFIXES: [(&str, &str, u32); 8] = [
    ("Ki", "kibi", 1),
    ("Mi", "mebi", 2),
    ("Gi", "gibi", 3),
    ("Ti", "tebi", 4),
    ("Pi", "pebi", 5),
    ("Ei", "exbi", 6),
    ("Zi", "zebi", 7),
    ("Yi", "yobi", 8),
];

/// Which prefixes a unit accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Prefixes {
    None,
    /// SI prefixes, optionally restricted to the listed symbols.
    Si(Option<Vec<String>>),
    /// All SI prefixes plus binary prefixes.
    SiBinary,
}

impl Prefixes {
    fn allows_si(&self, symbol: &str) -> bool {
        match self {
            Self::None => false,
            Self::Si(None) | Self::SiBinary => true,
            Self::Si(Some(allowed)) => allowed.iter().any(|allowed| allowed == symbol),
        }
    }
}

/// One unit of the table.
#[derive(Debug, Clone)]
pub struct UnitDef {
    /// Canonical display symbol.
    pub symbol: String,
    /// Every accepted spelling, including the symbol.
    pub aliases: Vec<String>,
    /// Physical dimension.
    pub dimension: Dimension,
    /// Factor to the coherent SI unit of the dimension.
    pub factor: Rational,
    /// Offset of an affine unit (`°C`, `°F`): base = (value + offset) × factor.
    pub offset: Option<Rational>,
    /// Canonical unit terms of a definition made only of other units
    /// (`mph` = `mi h^-1`), used to simplify products such as `mph·h`.
    pub expansion: Option<Vec<(String, i32)>>,
    prefixes: Prefixes,
}

impl UnitDef {
    /// Whether this is an affine (offset) unit such as `°C`.
    #[must_use]
    pub const fn is_affine(&self) -> bool {
        self.offset.is_some()
    }
}

/// A unit name resolved against the table, including any prefix.
#[derive(Debug, Clone)]
pub struct ResolvedUnit {
    /// The unit definition.
    pub def: &'static UnitDef,
    /// Canonical symbol including the prefix (`km`, `µs`, `KiB`).
    pub symbol: String,
    /// Factor to the coherent SI unit, including the prefix.
    pub factor: Rational,
}

impl ResolvedUnit {
    /// The dimension of the unit.
    #[must_use]
    pub const fn dimension(&self) -> Dimension {
        self.def.dimension
    }

    /// The affine offset, for unprefixed affine units.
    #[must_use]
    pub fn offset(&self) -> Option<&Rational> {
        self.def.offset.as_ref()
    }
}

/// The parsed unit table.
#[derive(Debug, Default)]
pub struct UnitDatabase {
    units: Vec<UnitDef>,
    /// Case-sensitive spellings to unit index.
    exact: HashMap<String, usize>,
    /// Lowercase word aliases to unit index.
    words: HashMap<String, usize>,
    /// Coherent named units products collapse into.
    named: Vec<usize>,
    /// Words that are never read as a prefixed unit.
    reserved: Vec<String>,
}

/// Returns the bundled unit database.
pub fn units() -> &'static UnitDatabase {
    static UNITS: OnceLock<UnitDatabase> = OnceLock::new();
    UNITS.get_or_init(|| parse_database(UNITS_LINO).expect("data/units/units.lino is valid"))
}

/// Looks up a unit name in the bundled database.
#[must_use]
pub fn lookup_unit(name: &str) -> Option<ResolvedUnit> {
    let database = units();
    let (index, symbol, factor) = database.resolve(name)?;
    Some(ResolvedUnit {
        def: &database.units[index],
        symbol,
        factor,
    })
}

/// Returns the coherent named unit (`N`, `J`, `V`, …) of a dimension.
#[must_use]
pub fn named_unit_for(dimension: &Dimension) -> Option<&'static str> {
    let database = units();
    database
        .named
        .iter()
        .map(|index| &database.units[*index])
        .find(|def| def.dimension == *dimension)
        .map(|def| def.symbol.as_str())
}

/// Whether an alias is a word (`metre`, `feet`) rather than a symbol (`m`, `ft`).
fn is_word_alias(alias: &str, symbol: &str) -> bool {
    alias != symbol && alias.chars().count() >= 3 && alias.chars().all(char::is_alphabetic)
}

fn normalize_prefix(prefix: &str) -> &str {
    match prefix {
        "u" | "μ" => "µ",
        other => other,
    }
}

fn ten_pow(exponent: i32) -> Rational {
    Rational::from_integer(10).pow_i32(exponent)
}

impl UnitDatabase {
    /// Resolves a name to (unit index, canonical symbol, factor).
    fn resolve(&self, name: &str) -> Option<(usize, String, Rational)> {
        if let Some(&index) = self.exact.get(name) {
            let def = &self.units[index];
            return Some((index, def.symbol.clone(), def.factor.clone()));
        }
        if self.reserved.iter().any(|reserved| reserved == name) {
            return None;
        }
        self.resolve_symbol_prefix(name)
            .or_else(|| self.resolve_word_prefix(&name.to_lowercase()))
            .or_else(|| {
                let all_upper = name.chars().all(char::is_uppercase);
                let lower = name.to_lowercase();
                let &index = self.words.get(&lower).filter(|_| !all_upper)?;
                let def = &self.units[index];
                Some((index, def.symbol.clone(), def.factor.clone()))
            })
    }

    /// Prefix symbol + unit symbol (`km`, `µs`, `uF`, `KiB`).
    fn resolve_symbol_prefix(&self, name: &str) -> Option<(usize, String, Rational)> {
        let micro_spellings = [("u", -6), ("μ", -6)];
        let si = SI_PREFIXES
            .iter()
            .map(|(symbol, _, power)| (*symbol, *power))
            .chain(micro_spellings);
        for (prefix, power) in si {
            let Some(rest) = name.strip_prefix(prefix) else {
                continue;
            };
            let Some(&index) = self.exact.get(rest) else {
                continue;
            };
            let def = &self.units[index];
            let canonical = normalize_prefix(prefix);
            if !is_word_alias(rest, &def.symbol)
                && !def.is_affine()
                && def.prefixes.allows_si(canonical)
            {
                let symbol = format!("{canonical}{}", def.symbol);
                return Some((index, symbol, def.factor.clone() * ten_pow(power)));
            }
        }
        for (prefix, _, power) in BINARY_PREFIXES {
            let Some(rest) = name.strip_prefix(prefix) else {
                continue;
            };
            let Some(&index) = self.exact.get(rest) else {
                continue;
            };
            let def = &self.units[index];
            if def.prefixes == Prefixes::SiBinary && !is_word_alias(rest, &def.symbol) {
                let factor = def.factor.clone() * Rational::from_integer(1024_i128.pow(power));
                return Some((index, format!("{prefix}{}", def.symbol), factor));
            }
        }
        None
    }

    /// Prefix word + word alias (`kilometres`, `microseconds`, `kibibits`).
    fn resolve_word_prefix(&self, lower: &str) -> Option<(usize, String, Rational)> {
        for (symbol, word, power) in SI_PREFIXES {
            let Some(rest) = lower.strip_prefix(word) else {
                continue;
            };
            let Some(&index) = self.words.get(rest) else {
                continue;
            };
            let def = &self.units[index];
            if def.prefixes.allows_si(symbol) && !def.is_affine() {
                let factor = def.factor.clone() * ten_pow(power);
                return Some((index, format!("{symbol}{}", def.symbol), factor));
            }
        }
        for (symbol, word, power) in BINARY_PREFIXES {
            let Some(rest) = lower.strip_prefix(word) else {
                continue;
            };
            let Some(&index) = self.words.get(rest) else {
                continue;
            };
            let def = &self.units[index];
            if def.prefixes == Prefixes::SiBinary {
                let factor = def.factor.clone() * Rational::from_integer(1024_i128.pow(power));
                return Some((index, format!("{symbol}{}", def.symbol), factor));
            }
        }
        None
    }

    fn add(&mut self, def: UnitDef) -> Result<(), String> {
        let index = self.units.len();
        for alias in &def.aliases {
            if self.exact.insert(alias.clone(), index).is_some() {
                return Err(format!("duplicate unit name '{alias}'"));
            }
            if is_word_alias(alias, &def.symbol) {
                self.words.entry(alias.to_lowercase()).or_insert(index);
            }
        }
        self.units.push(def);
        Ok(())
    }

    /// Evaluates a definition such as `kg m s^-2` or `1/1000 m^3`.
    fn evaluate_definition(&self, definition: &str) -> Result<Evaluated, String> {
        let mut dimension = Dimension::NONE;
        let mut factor = Rational::one();
        let mut sign = 1;
        let mut terms = Vec::new();
        let mut pure = true;
        for part in definition.split_whitespace() {
            if part == "/" {
                sign = -1;
                continue;
            }
            if let Some(number) = parse_number(part) {
                factor = factor * number.pow_i32(sign);
                pure = false;
                continue;
            }
            if let Some(base) = part.strip_prefix('[').and_then(|p| p.strip_suffix(']')) {
                let base =
                    Dimension::base(base).ok_or_else(|| format!("unknown dimension '{base}'"))?;
                dimension = dimension.mul(&base.pow(sign));
                pure = false;
                continue;
            }
            let (name, power) = match part.split_once('^') {
                Some((name, power)) => (
                    name,
                    power
                        .parse::<i32>()
                        .map_err(|_| format!("invalid power in '{part}'"))?,
                ),
                None => (part, 1),
            };
            let (index, symbol, unit_factor) = self
                .resolve(name)
                .ok_or_else(|| format!("unknown unit '{name}'"))?;
            let power = power * sign;
            dimension = dimension.mul(&self.units[index].dimension.pow(power));
            factor = factor * unit_factor.pow_i32(power);
            terms.push((symbol, power));
        }
        let expansion = (pure && !matches!(terms.as_slice(), [(_, 1)])).then_some(terms);
        Ok((dimension, factor, expansion))
    }
}

/// Dimension, factor and optional expansion of a definition.
type Evaluated = (Dimension, Rational, Option<Vec<(String, i32)>>);

/// Parses `1000`, `0.3048`, `1.602176634e-19` or `5/9` exactly.
fn parse_number(text: &str) -> Option<Rational> {
    if let Some((numer, denom)) = text.split_once('/') {
        let denom = parse_number(denom)?;
        if denom.is_zero() {
            return None;
        }
        return Some(parse_number(numer)? / denom);
    }
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().ok()?),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{whole}{fraction}");
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let numer: BigInt = digits.parse().ok()?;
    let scale = i32::try_from(fraction.len()).ok()?;
    Some(Rational::from_bigint(numer) * ten_pow(exponent - scale))
}

/// Splits a link `(kind a 'b c' "d")` into its parts.
fn split_link(line: &str) -> Result<Vec<String>, String> {
    let inner = line
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or("expected a link in parentheses")?;
    let mut parts = Vec::new();
    let mut chars = inner.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == '\'' || ch == '"' {
            chars.next();
            let mut part = String::new();
            loop {
                match chars.next() {
                    Some(c) if c == ch => break,
                    Some(c) => part.push(c),
                    None => return Err("unterminated quote".to_string()),
                }
            }
            parts.push(part);
        } else {
            let mut part = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                part.push(c);
                chars.next();
            }
            parts.push(part);
        }
    }
    Ok(parts)
}

fn parse_prefixes(text: &str) -> Result<Prefixes, String> {
    Ok(match text {
        "none" => Prefixes::None,
        "si" => Prefixes::Si(None),
        "si-binary" => Prefixes::SiBinary,
        other => {
            let list = other
                .strip_prefix("si:")
                .ok_or_else(|| format!("unknown prefix class '{other}'"))?;
            Prefixes::Si(Some(list.split(',').map(str::to_string).collect()))
        }
    })
}

fn parse_database(content: &str) -> Result<UnitDatabase, String> {
    let mut database = UnitDatabase::default();
    for (line_index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let at_line = |message: String| format!("line {}: {message}", line_index + 1);
        let parts = split_link(line).map_err(at_line)?;
        let (kind, rest) = parts
            .split_first()
            .ok_or_else(|| at_line("empty link".to_string()))?;
        match kind.as_str() {
            "unit" | "affine" => {
                let (prefixes, definition, offset, names) = if kind == "unit" {
                    let [prefixes, definition, names @ ..] = rest else {
                        return Err(at_line("expected prefixes and a definition".into()));
                    };
                    (
                        parse_prefixes(prefixes).map_err(at_line)?,
                        definition,
                        None,
                        names,
                    )
                } else {
                    let [definition, offset, names @ ..] = rest else {
                        return Err(at_line("expected a definition and an offset".into()));
                    };
                    let offset = parse_number(offset)
                        .ok_or_else(|| at_line(format!("invalid offset '{offset}'")))?;
                    (Prefixes::None, definition, Some(offset), names)
                };
                let symbol = names
                    .first()
                    .ok_or_else(|| at_line("expected a symbol".to_string()))?;
                let (dimension, factor, expansion) =
                    database.evaluate_definition(definition).map_err(at_line)?;
                database
                    .add(UnitDef {
                        symbol: symbol.clone(),
                        aliases: names.to_vec(),
                        dimension,
                        factor,
                        offset,
                        expansion,
                        prefixes,
                    })
                    .map_err(at_line)?;
            }
            "named" => {
                for symbol in rest {
                    let index = *database
                        .exact
                        .get(symbol)
                        .ok_or_else(|| at_line(format!("unknown unit '{symbol}'")))?;
                    database.named.push(index);
                }
            }
            "reserved" => database.reserved.extend(rest.iter().cloned()),
            other => return Err(at_line(format!("unknown link kind '{other}'"))),
        }
    }
    Ok(database)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factor(name: &str) -> f64 {
        lookup_unit(name).unwrap().factor.to_f64()
    }

    fn symbol(name: &str) -> String {
        lookup_unit(name).unwrap().symbol
    }

    #[test]
    fn test_database_loads() {
        assert!(units().units.len() > 100);
    }

    #[test]
    fn test_si_prefixes_including_micro_spellings() {
        for name in ["µm", "μm", "um", "micrometre", "micrometers"] {
            assert_eq!(symbol(name), "µm", "{name}");
            assert!((factor(name) - 1e-6).abs() < 1e-18, "{name}");
        }
        assert_eq!(symbol("kilometres"), "km");
        assert_eq!(symbol("dam"), "dam");
        assert_eq!(symbol("mL"), "mL");
        assert_eq!(symbol("ml"), "mL");
        assert_eq!(symbol("kΩ"), "kΩ");
        assert_eq!(symbol("Mt"), "Mt");
    }

    #[test]
    fn test_binary_prefixes() {
        assert_eq!(symbol("KiB"), "KiB");
        assert!((factor("KiB") - 8192.0).abs() < 1e-9);
        assert_eq!(symbol("mebibits"), "Mibit");
    }

    #[test]
    fn test_exact_names_win_over_prefixes() {
        assert_eq!(symbol("ft"), "ft");
        assert_eq!(symbol("min"), "min");
        assert_eq!(symbol("Pa"), "Pa");
        assert_eq!(symbol("cd"), "cd");
        assert_eq!(symbol("pt"), "pt");
        assert!(lookup_unit("am").is_none());
        assert!(lookup_unit("as").is_none());
        assert!(lookup_unit("ft2").is_none());
    }

    #[test]
    fn test_aliases_and_case() {
        assert_eq!(symbol("feet"), "ft");
        assert_eq!(symbol("Feet"), "ft");
        assert_eq!(symbol("metres"), "m");
        assert_eq!(symbol("Celsius"), "°C");
        assert_eq!(symbol("Ω"), "Ω");
        assert!(lookup_unit("CUP").is_none());
    }

    #[test]
    fn test_definitions() {
        assert!((factor("ft") - 0.3048).abs() < 1e-15);
        assert!((factor("mi") - 1609.344).abs() < 1e-9);
        assert!((factor("ly") - 9_460_730_472_580_800.0).abs() < 1.0);
        assert!((factor("mph") - 0.44704).abs() < 1e-12);
        assert!((factor("J") - 1.0).abs() < 1e-15);
        assert_eq!(lookup_unit("J").unwrap().dimension(), {
            let m = Dimension::base("length").unwrap();
            let kg = Dimension::base("mass").unwrap();
            let s = Dimension::base("time").unwrap();
            kg.mul(&m.pow(2)).div(&s.pow(2))
        });
        let fahrenheit = lookup_unit("°F").unwrap();
        assert!((fahrenheit.factor.to_f64() - 5.0 / 9.0).abs() < 1e-15);
        assert_eq!(fahrenheit.offset().unwrap().to_f64(), 459.67);
    }

    #[test]
    fn test_named_units() {
        let volt = lookup_unit("V").unwrap().dimension();
        assert_eq!(named_unit_for(&volt), Some("V"));
        let hertz = lookup_unit("Hz").unwrap().dimension();
        assert_eq!(named_unit_for(&hertz), None);
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("5/9"), Some(Rational::new(5, 9)));
        assert_eq!(parse_number("0.3048"), Some(Rational::new(3048, 10000)));
        assert_eq!(parse_number("1.5e3"), Some(Rational::from_integer(1500)));
        assert_eq!(parse_number("m"), None);
    }
}
