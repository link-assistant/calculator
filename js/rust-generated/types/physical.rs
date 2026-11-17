// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=eccad0dc663f84cd824697315cd841cfa73e570634ef8a5c6b0359099b029bb5 bytes=13732

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Products of database units with integer powers (issue #235).
// |  *
// |  * A [`PhysicalUnit`] such as `kg m s^-2` keeps the unit symbols the user
// |  * wrote, so results display in familiar units, while its dimension and
// |  * scale come from the unit database in `data/units/units.lino`.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=adaeb34ef928777b7353507f66ef1ed575a99af493895ffbd4b7c2ccb1dfb824
// | import { eq, expect } from '../rust/core.js';
use crate::rust::core_::{eq, expect};
// meta-language:translated JavaScript import_statement items=1 sha256=7be64f87657035d0db20585dae0d588ad3f7226d5e25d33a3e0f2f4543387ac3
// | import { Dimension } from './dimension.js';
use crate::types::dimension::Dimension;
// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from './rational.js';
use crate::types::rational::Rational;
// meta-language:translated JavaScript import_statement items=1 sha256=f9078407a5b92441aba8935468e2f2955da7f1d51ca4b28e98063b489be301af
// | import { lookupSymbol, lookupUnit, namedUnitFor } from './unit_db.js';
use crate::types::unit_db::{lookupSymbol, lookupUnit, namedUnitFor};
// meta-language:translated JavaScript import_statement items=1 sha256=833d93e30036ffd3cd3844991b2876bad22900b080d3705a5b4d9b2c03b59c29
// | import { DataSizeUnit, DurationUnit, MassUnit, Unit } from './unit.js';
use crate::types::unit::{DataSizeUnit, DurationUnit, MassUnit, Unit};

// meta-language:carried JavaScript export_statement (unsupported)
// | /** One `symbol^power` factor of a unit product. */
// | export class UnitTerm {
// |   /** @param {{ symbol: string, power: number }} fields */
// |   constructor({ symbol, power }) {
// |     /** Canonical unit symbol, including any prefix (`km`, `µF`). */
// |     this.symbol = symbol;
// |     /** Non-zero integer power (`i32`). */
// |     this.power = power;
// |   }
// |
// |   /** @returns {import('./unit_db.js').ResolvedUnit} */
// |   resolved() {
// |     return expect(lookupSymbol(this.symbol), 'unit terms hold canonical database symbols');
// |   }
// |
// |   /** @returns {Dimension} */
// |   dimension() {
// |     return this.resolved().dimension().pow(this.power);
// |   }
// |
// |   /** @returns {Rational} */
// |   factor() {
// |     return this.resolved().factor.powI32(this.power);
// |   }
// |
// |   /** serde: `{ "symbol": ..., "power": ... }`. */
// |   toJSON() {
// |     return { symbol: this.symbol, power: this.power };
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A product of database units, such as `m`, `km/h` or `kg m/s^2`. */
// | export class PhysicalUnit {
// |   /** @param {{ terms: UnitTerm[] }} fields */
// |   constructor({ terms }) {
// |     /** @internal */
// |     this._terms = terms;
// |   }
// |
// |   /**
// |    * `#[derive(Default)]`: the empty product.
// |    * @returns {PhysicalUnit}
// |    */
// |   static default() {
// |     return new PhysicalUnit({ terms: [] });
// |   }
// |
// |   /**
// |    * Resolves a single unit name (`metres`, `km`, `µF`) from the database.
// |    * @param {string} name
// |    * @returns {PhysicalUnit | null}
// |    */
// |   static parse(name) {
// |     const resolved = lookupUnit(name);
// |     if (resolved === null) {
// |       return null;
// |     }
// |     return PhysicalUnit.fromTerms([[resolved.symbol, 1]]);
// |   }
// |
// |   /**
// |    * Builds a unit from `(canonical symbol, power)` pairs.
// |    * @param {Array<[string, number]>} terms
// |    * @returns {PhysicalUnit}
// |    */
// |   static fromTerms(terms) {
// |     return new PhysicalUnit({
// |       terms: terms.filter(([, power]) => power !== 0).map(([symbol, power]) => new UnitTerm({ symbol, power })),
// |     });
// |   }
// |
// |   /**
// |    * The `symbol^power` factors.
// |    * @returns {readonly UnitTerm[]}
// |    */
// |   terms() {
// |     return this._terms;
// |   }
// |
// |   /**
// |    * Whether the product has no factors.
// |    * @returns {boolean}
// |    */
// |   isEmpty() {
// |     return this._terms.length === 0;
// |   }
// |
// |   /**
// |    * The combined dimension.
// |    * @returns {Dimension}
// |    */
// |   dimension() {
// |     return this._terms.reduce((acc, term) => acc.mul(term.dimension()), Dimension.NONE);
// |   }
// |
// |   /**
// |    * The factor to the coherent SI unit of the dimension.
// |    * @returns {Rational}
// |    */
// |   factor() {
// |     return this._terms.reduce((acc, term) => acc.mul(term.factor()), Rational.one());
// |   }
// |
// |   /**
// |    * The offset of a lone affine unit such as `°C`.
// |    * @returns {Rational | null}
// |    */
// |   affineOffset() {
// |     if (this._terms.length === 1 && this._terms[0].power === 1) {
// |       return this._terms[0].resolved().offset();
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * Whether any factor is an affine unit (`°C`, `°F`).
// |    * @returns {boolean}
// |    */
// |   hasAffine() {
// |     return this._terms.some((term) => term.resolved().def.isAffine());
// |   }
// |
// |   /**
// |    * Raises every factor to an integer power.
// |    * @param {number} power
// |    * @returns {PhysicalUnit}
// |    */
// |   powi(power) {
// |     return PhysicalUnit.fromTerms(this._terms.map((term) => [term.symbol, Math.imul(term.power, power)]));
// |   }
// |
// |   /**
// |    * Takes the `n`th root, if every power is divisible by `n`.
// |    * @param {number} n
// |    * @returns {PhysicalUnit | null}
// |    */
// |   root(n) {
// |     if (n === 0 || this._terms.some((term) => term.power % n !== 0)) {
// |       return null;
// |     }
// |     return this.powi(1).mapPowers((power) => (power / n) | 0);
// |   }
// |
// |   /**
// |    * @param {(power: number) => number} f
// |    * @returns {PhysicalUnit}
// |    */
// |   mapPowers(f) {
// |     return PhysicalUnit.fromTerms(this._terms.map((term) => [term.symbol, f(term.power)]));
// |   }
// |
// |   /**
// |    * The unsimplified product of two units.
// |    * @param {PhysicalUnit} other
// |    * @returns {PhysicalUnit}
// |    */
// |   product(other) {
// |     const terms = this._terms.map((term) => new UnitTerm(term));
// |     terms.push(...other._terms.map((term) => new UnitTerm(term)));
// |     return new PhysicalUnit({ terms });
// |   }
// |
// |   /**
// |    * Simplifies a product, returning the new unit and the factor the
// |    * numeric value must be multiplied by.
// |    *
// |    * Equal symbols merge (`m·m` → `m^2`); terms of the same dimension merge
// |    * into the earlier one (`cm·m` → `cm^2`); compound units expand when that
// |    * shortens the product (`mph·h` → `mi`); and a product of coherent SI
// |    * units with a named derived unit collapses to the named unit (`V/Ω` →
// |    * `A`). A dimensionless product folds into the number.
// |    * @returns {[PhysicalUnit, Rational]}
// |    */
// |   simplify() {
// |     if (this.dimension().isNone() && this._terms.every((t) => !t.dimension().isNone())) {
// |       return [PhysicalUnit.default(), this.factor()];
// |     }
// |     let [unit, scale] = this.merge();
// |     while (true) {
// |       const expanded = unit.shorterExpansion();
// |       if (expanded === null) {
// |         break;
// |       }
// |       unit = expanded[0];
// |       scale = scale.mul(expanded[1]);
// |     }
// |     const named = unit.namedCollapse();
// |     if (named !== null) {
// |       unit = named[0];
// |       scale = scale.mul(named[1]);
// |     }
// |     return [unit, scale];
// |   }
// |
// |   /**
// |    * Merges equal symbols and equal dimensions into the earliest term.
// |    * @returns {[PhysicalUnit, Rational]}
// |    */
// |   merge() {
// |     let scale = Rational.one();
// |     /** @type {UnitTerm[]} */
// |     const merged = [];
// |     for (const term of this._terms) {
// |       const resolved = term.resolved();
// |       const baseDimension = resolved.dimension();
// |       const target = merged.find(
// |         (existing) =>
// |           existing.symbol === term.symbol ||
// |           (!baseDimension.isNone() && !resolved.def.isAffine() && eq(existing.resolved().dimension(), baseDimension)),
// |       );
// |       if (target !== undefined) {
// |         if (target.symbol !== term.symbol) {
// |           const ratio = resolved.factor.div(target.resolved().factor);
// |           scale = scale.mul(ratio.powI32(term.power));
// |         }
// |         target.power = (target.power + term.power) | 0;
// |       } else {
// |         merged.push(new UnitTerm(term));
// |       }
// |     }
// |     return [new PhysicalUnit({ terms: merged.filter((term) => term.power !== 0) }), scale];
// |   }
// |
// |   /**
// |    * Replaces one compound term by its definition when that shortens the product.
// |    * @returns {[PhysicalUnit, Rational] | null}
// |    */
// |   shorterExpansion() {
// |     for (let index = 0; index < this._terms.length; index += 1) {
// |       const term = this._terms[index];
// |       const resolved = term.resolved();
// |       const expansion = resolved.def.expansion;
// |       if (expansion === null) {
// |         continue;
// |       }
// |       if (resolved.symbol !== resolved.def.symbol) {
// |         continue;
// |       }
// |       const terms = this._terms.map((t) => new UnitTerm(t));
// |       terms.splice(
// |         index,
// |         1,
// |         ...expansion.map(([symbol, power]) => new UnitTerm({ symbol, power: Math.imul(power, term.power) })),
// |       );
// |       const [candidate, scale] = new PhysicalUnit({ terms }).merge();
// |       if (candidate._terms.length < this._terms.length) {
// |         return [candidate, scale];
// |       }
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * Collapses a product of SI-prefixed coherent units containing a named
// |    * derived unit (`V/Ω` → `A`, `mA·kΩ` → `V`), folding prefixes into the
// |    * returned scale.
// |    * @returns {[PhysicalUnit, Rational] | null}
// |    */
// |   namedCollapse() {
// |     if (this._terms.length < 2) {
// |       return null;
// |     }
// |     const coherent = this._terms.every((term) => {
// |       const resolved = term.resolved();
// |       return (
// |         (eq(resolved.factor, Rational.one()) || eq(resolved.def.factor, Rational.one())) && !resolved.def.isAffine()
// |       );
// |     });
// |     const hasDerived = this._terms.some((term) => {
// |       const resolved = term.resolved();
// |       return (
// |         namedUnitFor(resolved.dimension()) === resolved.def.symbol && !isBaseDimension(resolved.dimension())
// |       );
// |     });
// |     if (!(coherent && hasDerived)) {
// |       return null;
// |     }
// |     const named = namedUnitFor(this.dimension());
// |     if (named === null) {
// |       return null;
// |     }
// |     return [PhysicalUnit.fromTerms([[named, 1]]), this.factor()];
// |   }
// |
// |   /**
// |    * The unit as a legacy duration, data-size or mass unit, when it is one.
// |    * @returns {Unit | null}
// |    */
// |   toLegacy() {
// |     if (this._terms.length !== 1) {
// |       return null;
// |     }
// |     const term = this._terms[0];
// |     if (term.power !== 1) {
// |       return null;
// |     }
// |     return legacyUnit(term.symbol);
// |   }
// |
// |   /**
// |    * Converts a legacy unit or `Unit::Physical` into a unit product.
// |    *
// |    * Returns `None` for units outside dimensional algebra: currencies,
// |    * timezones and custom names.
// |    * @param {Unit} unit
// |    * @returns {PhysicalUnit | null}
// |    */
// |   static fromUnit(unit) {
// |     /** @type {string} */
// |     let symbol;
// |     switch (unit.tag) {
// |       case 'None':
// |         return PhysicalUnit.default();
// |       case 'Physical':
// |         return unit._0.clone();
// |       case 'Mass':
// |         symbol = massSymbol(unit._0);
// |         break;
// |       case 'Duration':
// |         symbol = durationSymbol(unit._0);
// |         break;
// |       case 'DataSize':
// |         symbol = dataSizeSymbol(unit._0);
// |         break;
// |       default:
// |         return null;
// |     }
// |     return PhysicalUnit.fromTerms([[symbol, 1]]);
// |   }
// |
// |   /**
// |    * Wraps the product as a [`Unit`], preferring legacy variants.
// |    * @returns {Unit}
// |    */
// |   intoUnit() {
// |     if (this.isEmpty()) {
// |       return Unit.None;
// |     }
// |     return this.toLegacy() ?? Unit.Physical(this);
// |   }
// |
// |   /**
// |    * `#[derive(Clone)]`.
// |    * @returns {PhysicalUnit}
// |    */
// |   clone() {
// |     return new PhysicalUnit({ terms: this._terms.map((term) => new UnitTerm(term)) });
// |   }
// |
// |   /**
// |    * `#[derive(PartialEq)]` (the private field is named `_terms` here).
// |    * @param {unknown} other
// |    * @returns {boolean}
// |    */
// |   equals(other) {
// |     return other instanceof PhysicalUnit && eq(this._terms, other._terms);
// |   }
// |
// |   /** serde: `{ "terms": [...] }`. */
// |   toJSON() {
// |     return { terms: this._terms.map((term) => term.toJSON()) };
// |   }
// |
// |   /**
// |    * `impl Display`: formats as `kg m/s^2`, `m s^-1 kg^-1` or `kg^-1` (ASCII powers).
// |    * @returns {string}
// |    */
// |   toString() {
// |     /** @param {string} symbol @param {number} power */
// |     const formatTerm = (symbol, power) => (power === 1 ? symbol : `${symbol}^${power}`);
// |     const positive = this._terms.filter((term) => term.power > 0).map((term) => formatTerm(term.symbol, term.power));
// |     const negative = this._terms.filter((term) => term.power < 0);
// |     if (negative.length === 1 && positive.length > 0) {
// |       const single = negative[0];
// |       return `${positive.join(' ')}/${formatTerm(single.symbol, -single.power)}`;
// |     }
// |     return this._terms.map((term) => formatTerm(term.symbol, term.power)).join(' ');
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {Dimension} dimension
// |  * @returns {boolean}
// |  */
// | function isBaseDimension(dimension) {
// |   const nonzero = dimension._0.filter((power) => power !== 0);
// |   return nonzero.length === 1 && nonzero[0] === 1;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {MassUnit} unit
// |  * @returns {string}
// |  */
// | function massSymbol(unit) {
// |   switch (unit) {
// |     case MassUnit.Milligram:
// |       return 'mg';
// |     case MassUnit.Gram:
// |       return 'g';
// |     case MassUnit.Kilogram:
// |       return 'kg';
// |     case MassUnit.MetricTon:
// |       return 't';
// |     case MassUnit.Pound:
// |       return 'lb';
// |     default:
// |       return 'oz';
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {DurationUnit} unit
// |  * @returns {string}
// |  */
// | function durationSymbol(unit) {
// |   switch (unit.tag) {
// |     case 'Milliseconds':
// |       return 'ms';
// |     case 'Seconds':
// |       return 's';
// |     case 'Minutes':
// |       return 'min';
// |     case 'Hours':
// |       return 'h';
// |     case 'Days':
// |       return 'day';
// |     case 'Weeks':
// |       return 'week';
// |     case 'Months':
// |       return 'month';
// |     default:
// |       return 'year';
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Record<string, string>} */
// | const DATA_SIZE_SYMBOLS = Object.freeze({
// |   Bit: 'bit',
// |   Kilobit: 'kbit',
// |   Megabit: 'Mbit',
// |   Gigabit: 'Gbit',
// |   Terabit: 'Tbit',
// |   Petabit: 'Pbit',
// |   Kibibit: 'Kibit',
// |   Mebibit: 'Mibit',
// |   Gibibit: 'Gibit',
// |   Tebibit: 'Tibit',
// |   Pebibit: 'Pibit',
// |   Byte: 'B',
// |   Kilobyte: 'kB',
// |   Megabyte: 'MB',
// |   Gigabyte: 'GB',
// |   Terabyte: 'TB',
// |   Petabyte: 'PB',
// |   Kibibyte: 'KiB',
// |   Mebibyte: 'MiB',
// |   Gibibyte: 'GiB',
// |   Tebibyte: 'TiB',
// |   Pebibyte: 'PiB',
// | });

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {DataSizeUnit} unit
// |  * @returns {string}
// |  */
// | function dataSizeSymbol(unit) {
// |   return DATA_SIZE_SYMBOLS[unit.tag];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} symbol
// |  * @returns {Unit | null}
// |  */
// | function legacyUnit(symbol) {
// |   const MASSES = [
// |     MassUnit.Milligram,
// |     MassUnit.Gram,
// |     MassUnit.Kilogram,
// |     MassUnit.MetricTon,
// |     MassUnit.Pound,
// |     MassUnit.Ounce,
// |   ];
// |   const DURATIONS = [
// |     DurationUnit.Milliseconds,
// |     DurationUnit.Seconds,
// |     DurationUnit.Minutes,
// |     DurationUnit.Hours,
// |     DurationUnit.Days,
// |     DurationUnit.Weeks,
// |     DurationUnit.Months,
// |     DurationUnit.Years,
// |   ];
// |   const mass = MASSES.find((m) => massSymbol(m) === symbol);
// |   if (mass !== undefined) {
// |     return Unit.Mass(mass);
// |   }
// |   const duration = DURATIONS.find((d) => durationSymbol(d) === symbol);
// |   if (duration !== undefined) {
// |     return Unit.Duration(duration);
// |   }
// |   const size = DataSizeUnit.parse(symbol);
// |   return size !== null && dataSizeSymbol(size) === symbol ? Unit.DataSize(size) : null;
// | }
