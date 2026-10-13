// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=190d597f7a5625b0600b57c4c85d4487911a1e7cd06a832283aba143bb817202 bytes=31799

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Unit types for values (port of `src/types/unit.rs`).
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { contains } from '../rust/str.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { intToF64 } from '../rust/num.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { eq } from '../rust/core.js';
// meta-language:carried JavaScript import_statement (unsupported)
// | import { PhysicalUnit } from './physical.js';

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Builds the lookup map of a `match s { "a" | "b" => Some(X), ... }`.
// |  * The first arm that lists a string wins, as in a Rust `match`.
// |  * @template T
// |  * @param {Array<[string[], T]>} arms
// |  * @returns {Map<string, T>}
// |  */
// | function matchTable(arms) {
// |   /** @type {Map<string, T>} */
// |   const table = new Map();
// |   for (const [patterns, value] of arms) {
// |     for (const pattern of patterns) {
// |       if (!table.has(pattern)) {
// |         table.set(pattern, value);
// |       }
// |     }
// |   }
// |   return table;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Represents a unit of measurement.
// |  *
// |  * Variants: `None` (default, dimensionless), `Currency(String)` (e.g. USD,
// |  * EUR, TON crypto), `Duration(DurationUnit)`, `DataSize(DataSizeUnit)` (e.g.
// |  * KB, MiB, GB), `Mass(MassUnit)` (e.g. kg, ton, lb), `Timezone(String)` for
// |  * datetime conversion (e.g. MSK, EST, GMT), `Custom(String)` and
// |  * `Physical(PhysicalUnit)`, a product of database units with integer powers
// |  * (`m`, `km/h`, `kg m/s^2`).
// |  */
// | export class Unit {
// |   /** @param {string} tag @param {object} [fields] */
// |   constructor(tag, fields) {
// |     /** @type {'None' | 'Currency' | 'Duration' | 'DataSize' | 'Mass' | 'Timezone' | 'Custom' | 'Physical'} */
// |     this.tag = /** @type {any} */ (tag);
// |     /** The payload of a tuple variant; `null` for `None`. @type {any} */
// |     this._0 = fields === undefined ? null : /** @type {{ _0: unknown }} */ (fields)._0;
// |     Object.freeze(this);
// |   }
// |
// |   /** No unit (dimensionless). */
// |   static None = new Unit('None');
// |
// |   /**
// |    * Currency unit (e.g., USD, EUR, TON crypto).
// |    * @param {string} code
// |    */
// |   static Currency(code) {
// |     return new Unit('Currency', { _0: code });
// |   }
// |
// |   /**
// |    * Time duration unit.
// |    * @param {DurationUnit} unit
// |    */
// |   static Duration(unit) {
// |     return new Unit('Duration', { _0: unit });
// |   }
// |
// |   /**
// |    * Data size unit (e.g., KB, MiB, GB).
// |    * @param {DataSizeUnit} unit
// |    */
// |   static DataSize(unit) {
// |     return new Unit('DataSize', { _0: unit });
// |   }
// |
// |   /**
// |    * Mass/weight unit (e.g., kg, ton, lb).
// |    * @param {MassUnit} unit
// |    */
// |   static Mass(unit) {
// |     return new Unit('Mass', { _0: unit });
// |   }
// |
// |   /**
// |    * Timezone for datetime conversion (e.g., MSK, EST, GMT).
// |    * @param {string} tz
// |    */
// |   static Timezone(tz) {
// |     return new Unit('Timezone', { _0: tz });
// |   }
// |
// |   /**
// |    * Custom unit.
// |    * @param {string} name
// |    */
// |   static Custom(name) {
// |     return new Unit('Custom', { _0: name });
// |   }
// |
// |   /**
// |    * Product of database units with integer powers (`m`, `km/h`, `kg m/s^2`).
// |    * @param {PhysicalUnit} unit
// |    */
// |   static Physical(unit) {
// |     return new Unit('Physical', { _0: unit });
// |   }
// |
// |   /**
// |    * `#[derive(Default)]`: `Unit::None`.
// |    * @returns {Unit}
// |    */
// |   static default() {
// |     return Unit.None;
// |   }
// |
// |   /**
// |    * Whether this is an opaque compound unit without dimensional algebra.
// |    * @returns {boolean}
// |    */
// |   isOpaqueCompound() {
// |     return this.tag === 'Custom' && contains(this._0, '/');
// |   }
// |
// |   /**
// |    * Creates a currency unit.
// |    * @param {string} code
// |    * @returns {Unit}
// |    */
// |   static currency(code) {
// |     return Unit.Currency(code.toUpperCase());
// |   }
// |
// |   /**
// |    * Creates a data size unit.
// |    * @param {DataSizeUnit} unit
// |    * @returns {Unit}
// |    */
// |   static dataSize(unit) {
// |     return Unit.DataSize(unit);
// |   }
// |
// |   /**
// |    * Creates a mass unit.
// |    * @param {MassUnit} unit
// |    * @returns {Unit}
// |    */
// |   static mass(unit) {
// |     return Unit.Mass(unit);
// |   }
// |
// |   /**
// |    * Checks if the unit is a currency.
// |    * @returns {boolean}
// |    */
// |   isCurrency() {
// |     return this.tag === 'Currency';
// |   }
// |
// |   /**
// |    * Checks if the unit is a duration.
// |    * @returns {boolean}
// |    */
// |   isDuration() {
// |     return this.tag === 'Duration';
// |   }
// |
// |   /**
// |    * Checks if the unit is a data size unit.
// |    * @returns {boolean}
// |    */
// |   isDataSize() {
// |     return this.tag === 'DataSize';
// |   }
// |
// |   /**
// |    * Checks if the unit is a mass unit.
// |    * @returns {boolean}
// |    */
// |   isMass() {
// |     return this.tag === 'Mass';
// |   }
// |
// |   /**
// |    * Checks if the unit is a timezone.
// |    * @returns {boolean}
// |    */
// |   isTimezone() {
// |     return this.tag === 'Timezone';
// |   }
// |
// |   /**
// |    * Checks if two units are in the same category (both currencies, both mass, etc.).
// |    *
// |    * `Unit::None` is treated as compatible with any category.
// |    * @param {Unit} other
// |    * @returns {boolean}
// |    */
// |   isSameCategory(other) {
// |     if (this.tag === 'Physical' || other.tag === 'Physical') {
// |       return this.sameDimension(other);
// |     }
// |     return (
// |       this.tag === 'None' ||
// |       other.tag === 'None' ||
// |       (this.tag === other.tag &&
// |         (this.tag === 'Currency' ||
// |           this.tag === 'Duration' ||
// |           this.tag === 'DataSize' ||
// |           this.tag === 'Mass' ||
// |           this.tag === 'Timezone' ||
// |           this.tag === 'Custom'))
// |     );
// |   }
// |
// |   /**
// |    * Whether both units are physical and share a dimension (`m` and `ft`).
// |    * @param {Unit} other
// |    * @returns {boolean}
// |    */
// |   sameDimension(other) {
// |     const a = PhysicalUnit.fromUnit(this);
// |     const b = PhysicalUnit.fromUnit(other);
// |     if (a !== null && b !== null) {
// |       return eq(a.dimension(), b.dimension());
// |     }
// |     return false;
// |   }
// |
// |   /**
// |    * Checks if two units are compatible for the given operation.
// |    * @param {Unit} other
// |    * @param {string} op
// |    * @returns {boolean}
// |    */
// |   isCompatibleForOperation(other, op) {
// |     if (this.tag === 'None' || other.tag === 'None') {
// |       return true;
// |     }
// |     if (this.tag === 'Currency' && other.tag === 'Currency') {
// |       // Currencies can be added/subtracted (with conversion)
// |       // but not multiplied/divided together
// |       switch (op) {
// |         case '+':
// |         case '-':
// |           return true;
// |         case '*':
// |         case '/':
// |           return this._0 === other._0;
// |         default:
// |           return false;
// |       }
// |     }
// |     if (this.tag === 'Duration' && other.tag === 'Duration') {
// |       // Durations can be added/subtracted if same unit
// |       // For different units, we'd need conversion
// |       switch (op) {
// |         case '+':
// |         case '-':
// |           return this._0 === other._0;
// |         default:
// |           return false;
// |       }
// |     }
// |     if (this.tag === 'DataSize' && other.tag === 'DataSize') {
// |       // Data sizes can be added/subtracted (with conversion between units)
// |       return op === '+' || op === '-';
// |     }
// |     if (this.tag === 'Mass' && other.tag === 'Mass') {
// |       // Mass units can be added/subtracted (with conversion between units)
// |       return op === '+' || op === '-';
// |     }
// |     return false;
// |   }
// |
// |   /**
// |    * Returns the display name of the unit.
// |    * @returns {string}
// |    */
// |   displayName() {
// |     switch (this.tag) {
// |       case 'None':
// |         return '';
// |       case 'DataSize':
// |       case 'Mass':
// |         return this._0.abbreviation();
// |       default:
// |         return String(this._0);
// |     }
// |   }
// |
// |   /**
// |    * Returns the display name for a unit conversion target.
// |    * @returns {string}
// |    */
// |   conversionTargetName() {
// |     return this.tag === 'None' ? 'number' : this.displayName();
// |   }
// |
// |   /**
// |    * `impl Display`.
// |    * @returns {string}
// |    */
// |   toString() {
// |     return this.tag === 'None' ? '' : String(this._0);
// |   }
// |
// |   /**
// |    * serde's externally tagged enum representation.
// |    * @returns {unknown}
// |    */
// |   toJSON() {
// |     if (this.tag === 'None') {
// |       return 'None';
// |     }
// |     const value = this._0;
// |     return { [this.tag]: typeof value === 'string' ? value : value.toJSON() };
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Duration units for time calculations. */
// | export class DurationUnit {
// |   /** @param {string} tag */
// |   constructor(tag) {
// |     /** @type {'Milliseconds' | 'Seconds' | 'Minutes' | 'Hours' | 'Days' | 'Weeks' | 'Months' | 'Years'} */
// |     this.tag = /** @type {any} */ (tag);
// |     Object.freeze(this);
// |   }
// |
// |   static Milliseconds = new DurationUnit('Milliseconds');
// |   static Seconds = new DurationUnit('Seconds');
// |   static Minutes = new DurationUnit('Minutes');
// |   static Hours = new DurationUnit('Hours');
// |   static Days = new DurationUnit('Days');
// |   static Weeks = new DurationUnit('Weeks');
// |   static Months = new DurationUnit('Months');
// |   static Years = new DurationUnit('Years');
// |
// |   /**
// |    * `impl Display`.
// |    * @returns {string}
// |    */
// |   toString() {
// |     switch (this.tag) {
// |       case 'Milliseconds':
// |         return 'milliseconds';
// |       case 'Seconds':
// |         return 'seconds';
// |       case 'Minutes':
// |         return 'minutes';
// |       case 'Hours':
// |         return 'hours';
// |       case 'Days':
// |         return 'days';
// |       case 'Weeks':
// |         return 'weeks';
// |       case 'Months':
// |         return 'months';
// |       default:
// |         return 'years';
// |     }
// |   }
// |
// |   /** serde: the variant name. */
// |   toJSON() {
// |     return this.tag;
// |   }
// |
// |   /**
// |    * Parses a string into a `DurationUnit`, returning `None` if not recognized.
// |    *
// |    * Supports English and Russian duration unit names (all grammatical cases).
// |    * @param {string} s
// |    * @returns {DurationUnit | null}
// |    */
// |   static parse(s) {
// |     return durationTable().get(s.toLowerCase()) ?? null;
// |   }
// |
// |   /**
// |    * Converts a duration to seconds.
// |    * @param {number} value
// |    * @returns {number}
// |    */
// |   toSecs(value) {
// |     switch (this.tag) {
// |       case 'Milliseconds':
// |         return value / 1000.0;
// |       case 'Seconds':
// |         return value;
// |       case 'Minutes':
// |         return value * 60.0;
// |       case 'Hours':
// |         return value * 3600.0;
// |       case 'Days':
// |         return value * 86400.0;
// |       case 'Weeks':
// |         return value * 604_800.0;
// |       case 'Months':
// |         return value * 2_592_000.0; // 30 days approximation
// |       default:
// |         return value * 31_536_000.0; // 365 days approximation
// |     }
// |   }
// |
// |   /**
// |    * Converts seconds to this duration unit.
// |    * @param {number} seconds
// |    * @returns {number}
// |    */
// |   secsToUnit(seconds) {
// |     switch (this.tag) {
// |       case 'Milliseconds':
// |         return seconds * 1000.0;
// |       case 'Seconds':
// |         return seconds;
// |       case 'Minutes':
// |         return seconds / 60.0;
// |       case 'Hours':
// |         return seconds / 3600.0;
// |       case 'Days':
// |         return seconds / 86400.0;
// |       case 'Weeks':
// |         return seconds / 604_800.0;
// |       case 'Months':
// |         return seconds / 2_592_000.0;
// |       default:
// |         return seconds / 31_536_000.0;
// |     }
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Data size units — both decimal (SI, powers of 1000) and binary (IEC, powers of 1024).
// |  *
// |  * SI units follow IEC 80000-13 / the International System of Units.
// |  * Binary units follow IEC 80000-13 (published January 29, 1999).
// |  */
// | export class DataSizeUnit {
// |   /** @param {string} tag @param {bigint} bits @param {string} abbreviation */
// |   constructor(tag, bits, abbreviation) {
// |     /** @type {string} */
// |     this.tag = tag;
// |     /** @type {bigint} */
// |     this._bits = bits;
// |     /** @type {string} */
// |     this._abbreviation = abbreviation;
// |     Object.freeze(this);
// |   }
// |
// |   // ---- Bits (smallest common unit) ----
// |   /** 1 bit */
// |   static Bit = new DataSizeUnit('Bit', 1n, 'bit');
// |   // ---- Decimal (SI) bit units ----
// |   /** 1 kilobit = 1,000 bits */
// |   static Kilobit = new DataSizeUnit('Kilobit', 1_000n, 'Kb');
// |   /** 1 megabit = 1,000,000 bits */
// |   static Megabit = new DataSizeUnit('Megabit', 1_000_000n, 'Mb');
// |   /** 1 gigabit = 1,000,000,000 bits */
// |   static Gigabit = new DataSizeUnit('Gigabit', 1_000_000_000n, 'Gb');
// |   /** 1 terabit = 1,000,000,000,000 bits */
// |   static Terabit = new DataSizeUnit('Terabit', 1_000_000_000_000n, 'Tb');
// |   /** 1 petabit = 1,000,000,000,000,000 bits */
// |   static Petabit = new DataSizeUnit('Petabit', 1_000_000_000_000_000n, 'Pb');
// |   // ---- Binary (IEC) bit units ----
// |   /** 1 kibibit = 1,024 bits */
// |   static Kibibit = new DataSizeUnit('Kibibit', 1_024n, 'Kib');
// |   /** 1 mebibit = 1,048,576 bits */
// |   static Mebibit = new DataSizeUnit('Mebibit', 1_048_576n, 'Mib');
// |   /** 1 gibibit = 1,073,741,824 bits */
// |   static Gibibit = new DataSizeUnit('Gibibit', 1_073_741_824n, 'Gib');
// |   /** 1 tebibit = 1,099,511,627,776 bits */
// |   static Tebibit = new DataSizeUnit('Tebibit', 1_099_511_627_776n, 'Tib');
// |   /** 1 pebibit = 1,125,899,906,842,624 bits */
// |   static Pebibit = new DataSizeUnit('Pebibit', 1_125_899_906_842_624n, 'Pib');
// |   // ---- Bytes ----
// |   /** 1 byte = 8 bits */
// |   static Byte = new DataSizeUnit('Byte', 8n, 'B');
// |   // ---- Decimal (SI) byte units ----
// |   /** 1 kilobyte = 1,000 bytes (SI) */
// |   static Kilobyte = new DataSizeUnit('Kilobyte', 8_000n, 'KB');
// |   /** 1 megabyte = 1,000,000 bytes (SI) */
// |   static Megabyte = new DataSizeUnit('Megabyte', 8_000_000n, 'MB');
// |   /** 1 gigabyte = 1,000,000,000 bytes (SI) */
// |   static Gigabyte = new DataSizeUnit('Gigabyte', 8_000_000_000n, 'GB');
// |   /** 1 terabyte = 1,000,000,000,000 bytes (SI) */
// |   static Terabyte = new DataSizeUnit('Terabyte', 8_000_000_000_000n, 'TB');
// |   /** 1 petabyte = 1,000,000,000,000,000 bytes (SI) */
// |   static Petabyte = new DataSizeUnit('Petabyte', 8_000_000_000_000_000n, 'PB');
// |   // ---- Binary (IEC) byte units ----
// |   /** 1 kibibyte = 1,024 bytes (IEC) */
// |   static Kibibyte = new DataSizeUnit('Kibibyte', 8_192n, 'KiB');
// |   /** 1 mebibyte = 1,048,576 bytes (IEC) */
// |   static Mebibyte = new DataSizeUnit('Mebibyte', 8_388_608n, 'MiB');
// |   /** 1 gibibyte = 1,073,741,824 bytes (IEC) */
// |   static Gibibyte = new DataSizeUnit('Gibibyte', 8_589_934_592n, 'GiB');
// |   /** 1 tebibyte = 1,099,511,627,776 bytes (IEC) */
// |   static Tebibyte = new DataSizeUnit('Tebibyte', 8_796_093_022_208n, 'TiB');
// |   /** 1 pebibyte = 1,125,899,906,842,624 bytes (IEC) */
// |   static Pebibyte = new DataSizeUnit('Pebibyte', 9_007_199_254_740_992n, 'PiB');
// |
// |   /**
// |    * Returns the number of bits this unit represents.
// |    * Uses exact integer arithmetic via u128 to avoid floating-point precision errors.
// |    * @returns {bigint}
// |    */
// |   bits() {
// |     return this._bits;
// |   }
// |
// |   /**
// |    * Converts a value from this unit to another data size unit.
// |    *
// |    * Conversion goes through bits as the canonical base unit.
// |    * @param {number} value
// |    * @param {DataSizeUnit} to
// |    * @returns {number}
// |    */
// |   convert(value, to) {
// |     const fromBits = intToF64(this.bits());
// |     const toBits = intToF64(to.bits());
// |     return (value * fromBits) / toBits;
// |   }
// |
// |   /**
// |    * Returns the standard abbreviation for this unit.
// |    * @returns {string}
// |    */
// |   abbreviation() {
// |     return this._abbreviation;
// |   }
// |
// |   /**
// |    * Parses a string into a `DataSizeUnit`, returning `None` if not recognized.
// |    * @param {string} s
// |    * @returns {DataSizeUnit | null}
// |    */
// |   static parse(s) {
// |     return dataSizeTable().get(s) ?? null;
// |   }
// |
// |   /**
// |    * `impl Display`.
// |    * @returns {string}
// |    */
// |   toString() {
// |     return this.abbreviation();
// |   }
// |
// |   /** serde: the variant name. */
// |   toJSON() {
// |     return this.tag;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Mass/weight units. */
// | export class MassUnit {
// |   /** @param {string} tag @param {number} grams @param {string} abbreviation */
// |   constructor(tag, grams, abbreviation) {
// |     /** @type {string} */
// |     this.tag = tag;
// |     /** @type {number} */
// |     this._grams = grams;
// |     /** @type {string} */
// |     this._abbreviation = abbreviation;
// |     Object.freeze(this);
// |   }
// |
// |   /** 1 milligram = 0.001 grams */
// |   static Milligram = new MassUnit('Milligram', 0.001, 'mg');
// |   /** 1 gram */
// |   static Gram = new MassUnit('Gram', 1.0, 'g');
// |   /** 1 kilogram = 1000 grams */
// |   static Kilogram = new MassUnit('Kilogram', 1000.0, 'kg');
// |   /** 1 metric ton = 1000 kilograms */
// |   static MetricTon = new MassUnit('MetricTon', 1_000_000.0, 't');
// |   /** 1 pound ≈ 453.592 grams */
// |   static Pound = new MassUnit('Pound', 453.592_37, 'lb');
// |   /** 1 ounce ≈ 28.3495 grams */
// |   static Ounce = new MassUnit('Ounce', 28.349_523, 'oz');
// |
// |   /**
// |    * Returns the number of grams this unit represents (as f64).
// |    * @returns {number}
// |    */
// |   grams() {
// |     return this._grams;
// |   }
// |
// |   /**
// |    * Converts a value from this unit to another mass unit.
// |    * @param {number} value
// |    * @param {MassUnit} to
// |    * @returns {number}
// |    */
// |   convert(value, to) {
// |     return (value * this.grams()) / to.grams();
// |   }
// |
// |   /**
// |    * Returns the standard abbreviation for this unit.
// |    * @returns {string}
// |    */
// |   abbreviation() {
// |     return this._abbreviation;
// |   }
// |
// |   /**
// |    * Parses a string into a `MassUnit`, returning `None` if not recognized.
// |    *
// |    * Supports English and multilingual aliases for all 7 app languages:
// |    * English (en), Russian (ru), Chinese (zh), Hindi (hi), Arabic (ar),
// |    * German (de), and French (fr).
// |    *
// |    * Note: Some words like Russian "фунт" (pound) are intentionally NOT included
// |    * here because they map to currencies (GBP) in `CurrencyDatabase::parse_currency`.
// |    * The ambiguity between mass-pound and currency-pound is resolved at a higher
// |    * level via `parse_unit_with_alternatives`.
// |    * @param {string} s
// |    * @returns {MassUnit | null}
// |    */
// |   static parse(s) {
// |     return massTable().get(s) ?? null;
// |   }
// |
// |   /**
// |    * `impl Display`.
// |    * @returns {string}
// |    */
// |   toString() {
// |     return this.abbreviation();
// |   }
// |
// |   /** serde: the variant name. */
// |   toJSON() {
// |     return this.tag;
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Map<string, DataSizeUnit> | null} */
// | let DATA_SIZE_TABLE = null;

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** The arms of `DataSizeUnit::parse`. */
// | function dataSizeTable() {
// |   DATA_SIZE_TABLE ??= matchTable([
// |     // Bits
// |     [['b', 'bit', 'bits'], DataSizeUnit.Bit],
// |     // Decimal bit units
// |     [['Kb', 'kbit', 'kilobit', 'kilobits'], DataSizeUnit.Kilobit],
// |     [['Mb', 'mbit', 'megabit', 'megabits'], DataSizeUnit.Megabit],
// |     [['Gb', 'gbit', 'gigabit', 'gigabits'], DataSizeUnit.Gigabit],
// |     [['Tb', 'tbit', 'terabit', 'terabits'], DataSizeUnit.Terabit],
// |     [['Pb', 'pbit', 'petabit', 'petabits'], DataSizeUnit.Petabit],
// |     // Binary bit units
// |     [['Kib', 'kibit', 'kibibit', 'kibibits'], DataSizeUnit.Kibibit],
// |     [['Mib', 'mibit', 'mebibit', 'mebibits'], DataSizeUnit.Mebibit],
// |     [['Gib', 'gibit', 'gibibit', 'gibibits'], DataSizeUnit.Gibibit],
// |     [['Tib', 'tibit', 'tebibit', 'tebibits'], DataSizeUnit.Tebibit],
// |     [['Pib', 'pibit', 'pebibit', 'pebibits'], DataSizeUnit.Pebibit],
// |     // Bytes
// |     [['B', 'byte', 'bytes'], DataSizeUnit.Byte],
// |     // Decimal byte units
// |     [['KB', 'kB', 'kilobyte', 'kilobytes'], DataSizeUnit.Kilobyte],
// |     [['MB', 'megabyte', 'megabytes'], DataSizeUnit.Megabyte],
// |     [['GB', 'gigabyte', 'gigabytes'], DataSizeUnit.Gigabyte],
// |     [['TB', 'terabyte', 'terabytes'], DataSizeUnit.Terabyte],
// |     [['PB', 'petabyte', 'petabytes'], DataSizeUnit.Petabyte],
// |     // Binary byte units
// |     [['KiB', 'kibibyte', 'kibibytes'], DataSizeUnit.Kibibyte],
// |     [['MiB', 'mebibyte', 'mebibytes'], DataSizeUnit.Mebibyte],
// |     [['GiB', 'gibibyte', 'gibibytes'], DataSizeUnit.Gibibyte],
// |     [['TiB', 'tebibyte', 'tebibytes'], DataSizeUnit.Tebibyte],
// |     [['PiB', 'pebibyte', 'pebibytes'], DataSizeUnit.Pebibyte],
// |   ]);
// |   return DATA_SIZE_TABLE;
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Map<string, MassUnit> | null} */
// | let MASS_TABLE = null;

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** The arms of `MassUnit::parse`. */
// | function massTable() {
// |   MASS_TABLE ??= matchTable([
// |     // ── English ──────────────────────────────────────────────────────
// |     [['mg', 'milligram', 'milligrams'], MassUnit.Milligram],
// |     [['g', 'gram', 'grams'], MassUnit.Gram],
// |     [['kg', 'kgs', 'kilogram', 'kilograms'], MassUnit.Kilogram],
// |     // Metric ton: includes "ton" (singular) which is ambiguous with TON cryptocurrency.
// |     // The ambiguity is resolved at a higher level (NumberGrammar::parse_unit_with_alternatives)
// |     // which detects when both mass and currency interpretations are valid and surfaces
// |     // them as alternative interpretations to the user.
// |     [['t', 'ton', 'tons', 'tonne', 'tonnes', 'metric_ton', 'metric_tons'], MassUnit.MetricTon],
// |     [['lb', 'lbs', 'pound', 'pounds'], MassUnit.Pound],
// |     [['oz', 'ounce', 'ounces'], MassUnit.Ounce],
// |
// |     // ── Russian (ru) ─────────────────────────────────────────────────
// |     // Milligram: миллиграмм (all grammatical cases)
// |     [
// |       [
// |         'мг',
// |         'миллиграмм',
// |         'миллиграмма',
// |         'миллиграммов',
// |         'миллиграмме',
// |         'миллиграмму',
// |         'миллиграммом',
// |         'миллиграммы',
// |         'миллиграммам',
// |         'миллиграммами',
// |         'миллиграммах',
// |       ],
// |       MassUnit.Milligram,
// |     ],
// |     // Gram: грамм (all grammatical cases)
// |     [
// |       ['г', 'гр', 'грамм', 'грамма', 'граммов', 'грамме', 'грамму', 'граммом', 'граммы', 'граммам', 'граммами', 'граммах'],
// |       MassUnit.Gram,
// |     ],
// |     // Kilogram: килограмм (all grammatical cases)
// |     [
// |       [
// |         'кг',
// |         'килограмм',
// |         'килограмма',
// |         'килограммов',
// |         'килограмме',
// |         'килограмму',
// |         'килограммом',
// |         'килограммы',
// |         'килограммам',
// |         'килограммами',
// |         'килограммах',
// |       ],
// |       MassUnit.Kilogram,
// |     ],
// |     // Metric ton: тонна (all grammatical cases)
// |     [['тонна', 'тонны', 'тонн', 'тонне', 'тонну', 'тонной', 'тоннам', 'тоннами', 'тоннах'], MassUnit.MetricTon],
// |     // Ounce: унция (all grammatical cases)
// |     [['унция', 'унции', 'унций', 'унцию', 'унцией', 'унциям', 'унциями', 'унциях'], MassUnit.Ounce],
// |
// |     // ── Chinese Simplified (zh) ──────────────────────────────────────
// |     // Milligram: 毫克 (háo kè)
// |     [['毫克'], MassUnit.Milligram],
// |     // Gram: 克 (kè)
// |     [['克'], MassUnit.Gram],
// |     // Kilogram: 千克 (qiān kè) formal; 公斤 (gōng jīn) common
// |     [['千克', '公斤'], MassUnit.Kilogram],
// |     // Metric ton: 吨 (dūn)
// |     [['吨'], MassUnit.MetricTon],
// |     // Pound: 磅 (bàng)
// |     [['磅'], MassUnit.Pound],
// |     // Ounce: 盎司 (àng sī)
// |     [['盎司'], MassUnit.Ounce],
// |
// |     // ── Hindi (hi) ───────────────────────────────────────────────────
// |     // Milligram: मिलीग्राम (milīgrām)
// |     [['मिलीग्राम'], MassUnit.Milligram],
// |     // Gram: ग्राम (grām)
// |     [['ग्राम'], MassUnit.Gram],
// |     // Kilogram: किलोग्राम (kilōgrām); किलो (kilo) short form
// |     [['किलोग्राम', 'किलो'], MassUnit.Kilogram],
// |     // Metric ton: टन (ṭan)
// |     [['टन'], MassUnit.MetricTon],
// |     // Ounce: औंस (auṃs)
// |     [['औंस'], MassUnit.Ounce],
// |
// |     // ── Arabic (ar) ──────────────────────────────────────────────────
// |     // Milligram: ميليغرام (mīlīġrām) / ملغ abbreviation
// |     [['ملغ', 'ميليغرام', 'ملغم'], MassUnit.Milligram],
// |     // Gram: غرام (ġrām) / جرام (jarām) Egyptian variant
// |     [['غرام', 'جرام'], MassUnit.Gram],
// |     // Kilogram: كيلوغرام (kīlūġrām) / كيلوجرام Egyptian / كغ / كيلو short
// |     [['كغ', 'كيلوغرام', 'كيلوجرام', 'كيلو'], MassUnit.Kilogram],
// |     // Metric ton: طن (ṭun) / أطنان (aṭnān) plural
// |     [['طن', 'أطنان'], MassUnit.MetricTon],
// |     // Pound: رطل (raṭl) / أرطال (arṭāl) plural — distinct from جنيه (GBP currency)
// |     [['رطل', 'أرطال'], MassUnit.Pound],
// |     // Ounce: أونصة (ūnṣa) / أوقية (ūqiyya) traditional
// |     [['أونصة', 'أوقية'], MassUnit.Ounce],
// |
// |     // ── German (de) ──────────────────────────────────────────────────
// |     // German uses Latin-script abbreviations (mg, g, kg, t, lb, oz) already handled above.
// |     // Full German words for mass units (nouns are capitalized in German):
// |     // "Gramm" is both singular and plural in German.
// |     [['Milligramm', 'milligramm'], MassUnit.Milligram],
// |     [['Gramm', 'gramm'], MassUnit.Gram],
// |     [['Kilogramm', 'kilogramm', 'Kilo', 'kilo'], MassUnit.Kilogram],
// |     // "tonne"/"Tonne" already covered by English section; "tonnen"/"Tonnen" is German plural
// |     [['Tonnen', 'tonnen'], MassUnit.MetricTon],
// |     [['Unze', 'unze', 'Unzen', 'unzen'], MassUnit.Ounce],
// |
// |     // ── French (fr) ──────────────────────────────────────────────────
// |     // French uses Latin-script abbreviations already handled above.
// |     // Full French words:
// |     [['milligramme', 'milligrammes'], MassUnit.Milligram],
// |     [['gramme', 'grammes'], MassUnit.Gram],
// |     [['kilogramme', 'kilogrammes'], MassUnit.Kilogram],
// |     // "tonne"/"tonnes" already covered by English
// |     [['once', 'onces'], MassUnit.Ounce],
// |   ]);
// |   return MASS_TABLE;
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Map<string, DurationUnit> | null} */
// | let DURATION_TABLE = null;

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** The arms of `DurationUnit::parse` (matched against the lowercase input). */
// | function durationTable() {
// |   DURATION_TABLE ??= matchTable([
// |     // ── English ──────────────────────────────────────────────────────
// |     [['ms', 'millisecond', 'milliseconds'], DurationUnit.Milliseconds],
// |     [['s', 'sec', 'secs', 'second', 'seconds'], DurationUnit.Seconds],
// |     [['min', 'mins', 'minute', 'minutes'], DurationUnit.Minutes],
// |     [['h', 'hr', 'hrs', 'hour', 'hours'], DurationUnit.Hours],
// |     [['d', 'day', 'days'], DurationUnit.Days],
// |     [['w', 'week', 'weeks'], DurationUnit.Weeks],
// |     [['mo', 'month', 'months'], DurationUnit.Months],
// |     [['y', 'yr', 'yrs', 'year', 'years'], DurationUnit.Years],
// |     // ── Russian (ru) ─────────────────────────────────────────────────
// |     // Millisecond: миллисекунда (all grammatical cases)
// |     [
// |       [
// |         'мс',
// |         'миллисекунда',
// |         'миллисекунды',
// |         'миллисекунд',
// |         'миллисекунде',
// |         'миллисекунду',
// |         'миллисекундой',
// |         'миллисекундам',
// |         'миллисекундами',
// |         'миллисекундах',
// |       ],
// |       DurationUnit.Milliseconds,
// |     ],
// |     // Second: секунда (all grammatical cases)
// |     [
// |       ['с', 'сек', 'секунда', 'секунды', 'секунд', 'секунде', 'секунду', 'секундой', 'секундам', 'секундами', 'секундах'],
// |       DurationUnit.Seconds,
// |     ],
// |     // Minute: минута (all grammatical cases)
// |     [
// |       ['мин', 'минута', 'минуты', 'минут', 'минуте', 'минуту', 'минутой', 'минутам', 'минутами', 'минутах'],
// |       DurationUnit.Minutes,
// |     ],
// |     // Hour: час (all grammatical cases)
// |     [['ч', 'час', 'часа', 'часов', 'часе', 'часу', 'часом', 'часам', 'часами', 'часах'], DurationUnit.Hours],
// |     // Day: день (all grammatical cases)
// |     [['д', 'день', 'дня', 'дней', 'дне', 'дню', 'днём', 'дням', 'днями', 'днях'], DurationUnit.Days],
// |     // Week: неделя (all grammatical cases)
// |     [
// |       ['нед', 'неделя', 'недели', 'недель', 'неделе', 'неделю', 'неделей', 'неделям', 'неделями', 'неделях'],
// |       DurationUnit.Weeks,
// |     ],
// |     // Month: месяц (all grammatical cases)
// |     [
// |       ['мес', 'месяц', 'месяца', 'месяцев', 'месяце', 'месяцу', 'месяцем', 'месяцам', 'месяцами', 'месяцах'],
// |       DurationUnit.Months,
// |     ],
// |     // Year: год (all grammatical cases)
// |     [['г', 'год', 'года', 'лет', 'году', 'годом', 'годам', 'годами', 'годах'], DurationUnit.Years],
// |     // ── German (de) ──────────────────────────────────────────────────
// |     [['millisekunde', 'millisekunden'], DurationUnit.Milliseconds],
// |     [['sekunde', 'sekunden', 'sek'], DurationUnit.Seconds],
// |     [['minuten'], DurationUnit.Minutes], // "minute"/"min" already covered by English
// |     [['stunde', 'stunden', 'std'], DurationUnit.Hours],
// |     [['tag', 'tage', 'tagen'], DurationUnit.Days],
// |     [['woche', 'wochen'], DurationUnit.Weeks],
// |     [['monat', 'monate', 'monaten', 'monats'], DurationUnit.Months],
// |     [['jahr', 'jahre', 'jahren', 'jahres'], DurationUnit.Years],
// |     // ── French (fr) ──────────────────────────────────────────────────
// |     [['milliseconde', 'millisecondes'], DurationUnit.Milliseconds],
// |     [['seconde', 'secondes'], DurationUnit.Seconds],
// |     // "minute"/"minutes" already covered by English
// |     [['heure', 'heures'], DurationUnit.Hours],
// |     [['jour', 'jours'], DurationUnit.Days],
// |     [['semaine', 'semaines'], DurationUnit.Weeks],
// |     [['mois'], DurationUnit.Months],
// |     [['an', 'ans', 'année', 'années', 'annee', 'annees'], DurationUnit.Years],
// |     // ── Chinese Simplified (zh) ───────────────────────────────────────
// |     [['毫秒'], DurationUnit.Milliseconds],
// |     [['秒'], DurationUnit.Seconds],
// |     [['分钟', '分'], DurationUnit.Minutes],
// |     [['小时', '时', '钟'], DurationUnit.Hours],
// |     [['天', '日'], DurationUnit.Days],
// |     [['周', '星期', '礼拜'], DurationUnit.Weeks],
// |     [['月', '个月'], DurationUnit.Months],
// |     [['年'], DurationUnit.Years],
// |     // ── Hindi (hi) ───────────────────────────────────────────────────
// |     [['मिलीसेकंड'], DurationUnit.Milliseconds],
// |     [['सेकंड', 'सेकेंड'], DurationUnit.Seconds],
// |     [['मिनट', 'मिनटों'], DurationUnit.Minutes],
// |     [['घंटा', 'घंटे', 'घंटों'], DurationUnit.Hours],
// |     [['दिन', 'दिनों'], DurationUnit.Days],
// |     [['सप्ताह', 'हफ्ता', 'हफ्ते', 'हफ्तों'], DurationUnit.Weeks],
// |     [['महीना', 'महीने', 'महीनों', 'माह', 'मास'], DurationUnit.Months],
// |     [['साल', 'वर्ष', 'वर्षों'], DurationUnit.Years],
// |     // ── Arabic (ar) ──────────────────────────────────────────────────
// |     [['مللي', 'ميلي'], DurationUnit.Milliseconds], // millisecond abbreviations
// |     [['ثانية', 'ثواني', 'ثوان'], DurationUnit.Seconds],
// |     [['دقيقة', 'دقائق'], DurationUnit.Minutes],
// |     [['ساعة', 'ساعات'], DurationUnit.Hours],
// |     [['يوم', 'أيام', 'ايام'], DurationUnit.Days],
// |     [['أسبوع', 'اسبوع', 'أسابيع', 'اسابيع'], DurationUnit.Weeks],
// |     [['شهر', 'أشهر', 'اشهر', 'شهور'], DurationUnit.Months],
// |     [['سنة', 'سنوات', 'عام', 'أعوام', 'اعوام'], DurationUnit.Years],
// |   ]);
// |   return DURATION_TABLE;
// | }
