//! Products of database units with integer powers (issue #235).
//!
//! A [`PhysicalUnit`] such as `kg m s^-2` keeps the unit symbols the user
//! wrote, so results display in familiar units, while its dimension and
//! scale come from the unit database in `data/units/units.lino`.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::unit_db::{lookup_unit, named_unit_for, ResolvedUnit};
use super::{DataSizeUnit, Dimension, DurationUnit, MassUnit, Rational, Unit};

/// One `symbol^power` factor of a unit product.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitTerm {
    /// Canonical unit symbol, including any prefix (`km`, `µF`).
    pub symbol: String,
    /// Non-zero integer power.
    pub power: i32,
}

/// A product of database units, such as `m`, `km/h` or `kg m/s^2`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PhysicalUnit {
    terms: Vec<UnitTerm>,
}

impl UnitTerm {
    fn resolved(&self) -> ResolvedUnit {
        lookup_unit(&self.symbol).expect("unit terms hold canonical database symbols")
    }

    fn dimension(&self) -> Dimension {
        self.resolved().dimension().pow(self.power)
    }

    fn factor(&self) -> Rational {
        self.resolved().factor.pow_i32(self.power)
    }
}

impl PhysicalUnit {
    /// Resolves a single unit name (`metres`, `km`, `µF`) from the database.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        let resolved = lookup_unit(name)?;
        Some(Self::from_terms(vec![(resolved.symbol, 1)]))
    }

    /// Builds a unit from `(canonical symbol, power)` pairs.
    #[must_use]
    pub fn from_terms(terms: Vec<(String, i32)>) -> Self {
        Self {
            terms: terms
                .into_iter()
                .filter(|(_, power)| *power != 0)
                .map(|(symbol, power)| UnitTerm { symbol, power })
                .collect(),
        }
    }

    /// The `symbol^power` factors.
    #[must_use]
    pub fn terms(&self) -> &[UnitTerm] {
        &self.terms
    }

    /// Whether the product has no factors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// The combined dimension.
    #[must_use]
    pub fn dimension(&self) -> Dimension {
        self.terms
            .iter()
            .fold(Dimension::NONE, |acc, term| acc.mul(&term.dimension()))
    }

    /// The factor to the coherent SI unit of the dimension.
    #[must_use]
    pub fn factor(&self) -> Rational {
        self.terms
            .iter()
            .fold(Rational::one(), |acc, term| acc * term.factor())
    }

    /// The offset of a lone affine unit such as `°C`.
    #[must_use]
    pub fn affine_offset(&self) -> Option<Rational> {
        match self.terms.as_slice() {
            [term] if term.power == 1 => term.resolved().offset().cloned(),
            _ => None,
        }
    }

    /// Whether any factor is an affine unit (`°C`, `°F`).
    #[must_use]
    pub fn has_affine(&self) -> bool {
        self.terms
            .iter()
            .any(|term| term.resolved().def.is_affine())
    }

    /// Raises every factor to an integer power.
    #[must_use]
    pub fn powi(&self, power: i32) -> Self {
        Self::from_terms(
            self.terms
                .iter()
                .map(|term| (term.symbol.clone(), term.power * power))
                .collect(),
        )
    }

    /// Takes the `n`th root, if every power is divisible by `n`.
    #[must_use]
    pub fn root(&self, n: i32) -> Option<Self> {
        if n == 0 || self.terms.iter().any(|term| term.power % n != 0) {
            return None;
        }
        Some(self.powi(1).map_powers(|power| power / n))
    }

    fn map_powers(self, f: impl Fn(i32) -> i32) -> Self {
        Self::from_terms(
            self.terms
                .into_iter()
                .map(|term| (term.symbol, f(term.power)))
                .collect(),
        )
    }

    /// The unsimplified product of two units.
    #[must_use]
    pub fn product(&self, other: &Self) -> Self {
        let mut terms = self.terms.clone();
        terms.extend(other.terms.iter().cloned());
        Self { terms }
    }

    /// Simplifies a product, returning the new unit and the factor the
    /// numeric value must be multiplied by.
    ///
    /// Equal symbols merge (`m·m` → `m^2`); terms of the same dimension merge
    /// into the earlier one (`cm·m` → `cm^2`); compound units expand when that
    /// shortens the product (`mph·h` → `mi`); and a product of coherent SI
    /// units with a named derived unit collapses to the named unit (`V/Ω` →
    /// `A`). A dimensionless product folds into the number.
    #[must_use]
    pub fn simplify(&self) -> (Self, Rational) {
        if self.dimension().is_none() && self.terms.iter().all(|t| !t.dimension().is_none()) {
            return (Self::default(), self.factor());
        }
        let (mut unit, mut scale) = self.merge();
        while let Some((expanded, expanded_scale)) = unit.shorter_expansion() {
            unit = expanded;
            scale = scale * expanded_scale;
        }
        if let Some(named) = unit.named_collapse() {
            unit = named;
        }
        (unit, scale)
    }

    /// Merges equal symbols and equal dimensions into the earliest term.
    fn merge(&self) -> (Self, Rational) {
        let mut scale = Rational::one();
        let mut merged: Vec<UnitTerm> = Vec::new();
        for term in &self.terms {
            let resolved = term.resolved();
            let base_dimension = resolved.dimension();
            let target = merged.iter_mut().find(|existing| {
                existing.symbol == term.symbol
                    || (!base_dimension.is_none()
                        && !resolved.def.is_affine()
                        && existing.resolved().dimension() == base_dimension)
            });
            match target {
                Some(existing) => {
                    if existing.symbol != term.symbol {
                        let ratio = resolved.factor / existing.resolved().factor;
                        scale = scale * ratio.pow_i32(term.power);
                    }
                    existing.power += term.power;
                }
                None => merged.push(term.clone()),
            }
        }
        merged.retain(|term| term.power != 0);
        (Self { terms: merged }, scale)
    }

    /// Replaces one compound term by its definition when that shortens the product.
    fn shorter_expansion(&self) -> Option<(Self, Rational)> {
        for (index, term) in self.terms.iter().enumerate() {
            let resolved = term.resolved();
            let Some(expansion) = &resolved.def.expansion else {
                continue;
            };
            if resolved.symbol != resolved.def.symbol {
                continue;
            }
            let mut terms: Vec<UnitTerm> = self.terms.clone();
            terms.remove(index);
            terms.splice(
                index..index,
                expansion.iter().map(|(symbol, power)| UnitTerm {
                    symbol: symbol.clone(),
                    power: power * term.power,
                }),
            );
            let (candidate, scale) = Self { terms }.merge();
            if candidate.terms.len() < self.terms.len() {
                return Some((candidate, scale));
            }
        }
        None
    }

    /// Collapses a coherent product containing a named derived unit (`V/Ω`).
    fn named_collapse(&self) -> Option<Self> {
        if self.terms.len() < 2 {
            return None;
        }
        let coherent = self.terms.iter().all(|term| {
            let resolved = term.resolved();
            resolved.factor == Rational::one() && !resolved.def.is_affine()
        });
        let has_derived = self.terms.iter().any(|term| {
            named_unit_for(&term.resolved().dimension()) == Some(term.symbol.as_str())
                && !is_base_dimension(&term.resolved().dimension())
        });
        if !(coherent && has_derived) {
            return None;
        }
        let named = named_unit_for(&self.dimension())?;
        Some(Self::from_terms(vec![(named.to_string(), 1)]))
    }

    /// The unit as a legacy duration, data-size or mass unit, when it is one.
    #[must_use]
    pub fn to_legacy(&self) -> Option<Unit> {
        let [term] = self.terms.as_slice() else {
            return None;
        };
        if term.power != 1 {
            return None;
        }
        legacy_unit(&term.symbol)
    }

    /// Converts a legacy unit or `Unit::Physical` into a unit product.
    ///
    /// Returns `None` for units outside dimensional algebra: currencies,
    /// timezones and custom names.
    #[must_use]
    pub fn from_unit(unit: &Unit) -> Option<Self> {
        let symbol = match unit {
            Unit::None => return Some(Self::default()),
            Unit::Physical(physical) => return Some(physical.clone()),
            Unit::Mass(mass) => mass_symbol(*mass),
            Unit::Duration(duration) => duration_symbol(*duration),
            Unit::DataSize(size) => data_size_symbol(*size),
            Unit::Currency(_) | Unit::Timezone(_) | Unit::Custom(_) => return None,
        };
        Some(Self::from_terms(vec![(symbol.to_string(), 1)]))
    }

    /// Wraps the product as a [`Unit`], preferring legacy variants.
    #[must_use]
    pub fn into_unit(self) -> Unit {
        if self.is_empty() {
            return Unit::None;
        }
        self.to_legacy().unwrap_or(Unit::Physical(self))
    }
}

fn is_base_dimension(dimension: &Dimension) -> bool {
    let mut nonzero = dimension.0.iter().filter(|power| **power != 0);
    matches!((nonzero.next(), nonzero.next()), (Some(1), None))
}

const fn mass_symbol(unit: MassUnit) -> &'static str {
    match unit {
        MassUnit::Milligram => "mg",
        MassUnit::Gram => "g",
        MassUnit::Kilogram => "kg",
        MassUnit::MetricTon => "t",
        MassUnit::Pound => "lb",
        MassUnit::Ounce => "oz",
    }
}

const fn duration_symbol(unit: DurationUnit) -> &'static str {
    match unit {
        DurationUnit::Milliseconds => "ms",
        DurationUnit::Seconds => "s",
        DurationUnit::Minutes => "min",
        DurationUnit::Hours => "h",
        DurationUnit::Days => "day",
        DurationUnit::Weeks => "week",
        DurationUnit::Months => "month",
        DurationUnit::Years => "year",
    }
}

const fn data_size_symbol(unit: DataSizeUnit) -> &'static str {
    match unit {
        DataSizeUnit::Bit => "bit",
        DataSizeUnit::Kilobit => "kbit",
        DataSizeUnit::Megabit => "Mbit",
        DataSizeUnit::Gigabit => "Gbit",
        DataSizeUnit::Terabit => "Tbit",
        DataSizeUnit::Petabit => "Pbit",
        DataSizeUnit::Kibibit => "Kibit",
        DataSizeUnit::Mebibit => "Mibit",
        DataSizeUnit::Gibibit => "Gibit",
        DataSizeUnit::Tebibit => "Tibit",
        DataSizeUnit::Pebibit => "Pibit",
        DataSizeUnit::Byte => "B",
        DataSizeUnit::Kilobyte => "kB",
        DataSizeUnit::Megabyte => "MB",
        DataSizeUnit::Gigabyte => "GB",
        DataSizeUnit::Terabyte => "TB",
        DataSizeUnit::Petabyte => "PB",
        DataSizeUnit::Kibibyte => "KiB",
        DataSizeUnit::Mebibyte => "MiB",
        DataSizeUnit::Gibibyte => "GiB",
        DataSizeUnit::Tebibyte => "TiB",
        DataSizeUnit::Pebibyte => "PiB",
    }
}

fn legacy_unit(symbol: &str) -> Option<Unit> {
    const MASSES: [MassUnit; 6] = [
        MassUnit::Milligram,
        MassUnit::Gram,
        MassUnit::Kilogram,
        MassUnit::MetricTon,
        MassUnit::Pound,
        MassUnit::Ounce,
    ];
    const DURATIONS: [DurationUnit; 8] = [
        DurationUnit::Milliseconds,
        DurationUnit::Seconds,
        DurationUnit::Minutes,
        DurationUnit::Hours,
        DurationUnit::Days,
        DurationUnit::Weeks,
        DurationUnit::Months,
        DurationUnit::Years,
    ];
    if let Some(mass) = MASSES.into_iter().find(|m| mass_symbol(*m) == symbol) {
        return Some(Unit::Mass(mass));
    }
    if let Some(duration) = DURATIONS
        .into_iter()
        .find(|d| duration_symbol(*d) == symbol)
    {
        return Some(Unit::Duration(duration));
    }
    DataSizeUnit::parse(symbol)
        .filter(|size| data_size_symbol(*size) == symbol)
        .map(Unit::DataSize)
}

impl fmt::Display for PhysicalUnit {
    /// Formats as `kg m/s^2`, `m s^-1 kg^-1` or `kg^-1` (ASCII powers).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format_term = |symbol: &str, power: i32| {
            if power == 1 {
                symbol.to_string()
            } else {
                format!("{symbol}^{power}")
            }
        };
        let positive: Vec<String> = self
            .terms
            .iter()
            .filter(|term| term.power > 0)
            .map(|term| format_term(&term.symbol, term.power))
            .collect();
        let negative: Vec<&UnitTerm> = self.terms.iter().filter(|term| term.power < 0).collect();
        match negative.as_slice() {
            [single] if !positive.is_empty() => write!(
                f,
                "{}/{}",
                positive.join(" "),
                format_term(&single.symbol, -single.power)
            ),
            _ => {
                let all: Vec<String> = self
                    .terms
                    .iter()
                    .map(|term| format_term(&term.symbol, term.power))
                    .collect();
                write!(f, "{}", all.join(" "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(terms: &[(&str, i32)]) -> PhysicalUnit {
        PhysicalUnit::from_terms(terms.iter().map(|(s, p)| ((*s).to_string(), *p)).collect())
    }

    fn simplified(terms: &[(&str, i32)]) -> (String, f64) {
        let (unit, scale) = unit(terms).simplify();
        (unit.to_string(), scale.to_f64())
    }

    #[test]
    fn display_uses_ascii_powers() {
        assert_eq!(
            unit(&[("kg", 1), ("m", 1), ("s", -2)]).to_string(),
            "kg m/s^2"
        );
        assert_eq!(unit(&[("kg", -1)]).to_string(), "kg^-1");
        assert_eq!(
            unit(&[("m", 1), ("s", -1), ("kg", -1)]).to_string(),
            "m s^-1 kg^-1"
        );
        assert_eq!(unit(&[("km", 1), ("day", -1)]).to_string(), "km/day");
    }

    #[test]
    fn simplify_merges_symbols_and_dimensions() {
        assert_eq!(simplified(&[("m", 1), ("m", 1)]), ("m^2".to_string(), 1.0));
        assert_eq!(
            simplified(&[("cm", 1), ("m", 1)]),
            ("cm^2".to_string(), 100.0)
        );
        assert_eq!(simplified(&[("m", 1), ("cm", -1)]), (String::new(), 100.0));
        let (text, scale) = simplified(&[("lb", -1), ("kg", -1)]);
        assert_eq!(text, "lb^-2");
        assert!((scale - 0.453_592_37).abs() < 1e-12);
    }

    #[test]
    fn simplify_expands_and_collapses() {
        assert_eq!(simplified(&[("mph", 1), ("h", 1)]), ("mi".to_string(), 1.0));
        assert_eq!(simplified(&[("V", 1), ("Ω", -1)]), ("A".to_string(), 1.0));
        assert_eq!(simplified(&[("A", 1), ("Ω", 1)]), ("V".to_string(), 1.0));
        assert_eq!(
            simplified(&[("kg", 1), ("m", 1), ("s", -2)]),
            ("kg m/s^2".to_string(), 1.0)
        );
        assert_eq!(
            simplified(&[("kW", 1), ("h", 1)]),
            ("kW h".to_string(), 1.0)
        );
    }

    #[test]
    fn legacy_bridge_round_trips() {
        for unit in [
            Unit::Mass(MassUnit::Pound),
            Unit::Duration(DurationUnit::Days),
            Unit::DataSize(DataSizeUnit::Kibibyte),
            Unit::DataSize(DataSizeUnit::Kilobit),
        ] {
            let physical = PhysicalUnit::from_unit(&unit).expect("legacy units are physical");
            assert_eq!(physical.into_unit(), unit);
        }
        assert_eq!(PhysicalUnit::from_unit(&Unit::currency("USD")), None);
    }
}
