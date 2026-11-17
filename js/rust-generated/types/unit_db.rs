// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ca92ebc060be70f8582fb36734a14ac6916eb87345ffdf21e0df74b0aef6b492 bytes=20461

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * The physical unit database (issue #235).
// |  *
// |  * Units live in `data/units/units.lino` (one link per unit, see the format
// |  * notes at the top of that file), so adding a unit means editing data, not
// |  * code. Each unit has a dimension, a factor to the coherent SI unit of that
// |  * dimension and, for `°C`/`°F`, an offset.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=0a9500644716d80634c79e584b74a9a656f7a5d977f113951bf8327746a663c3
// | import { eq, err, expectResult, ok, tryResult } from '../rust/core.js';
use crate::rust::core_::{eq, err, expectResult, ok, tryResult};
// meta-language:translated JavaScript import_statement items=1 sha256=f794ec57e30a3721cfa7a9110f6cb1529861e227164553059f1def65f32a079e
// | import { I32 } from '../rust/num.js';
use crate::rust::num::I32;
// meta-language:translated JavaScript import_statement items=1 sha256=23cd99e17b11292f68329c7ee3e2c8337284beb01e3192398ec635c14dabb13b
// | import { bigintFromStr } from '../rust/ratio.js';
use crate::rust::ratio::bigintFromStr;
// meta-language:translated JavaScript import_statement items=1 sha256=2699a6e714b1b381878d061164d692b89a1bb78a41240d141441ee48cdac521a
// | import {
// |   isAlphabetic,
// |   isAsciiDigit,
// |   isUppercase,
// |   isWhitespace,
// |   lines,
// |   splitOnce,
// |   splitWhitespace,
// |   startsWith,
// |   stripPrefix,
// |   stripSuffix,
// |   trim,
// |   utf8Len,
// | } from '../rust/str.js';
use crate::rust::str_::{isAlphabetic, isAsciiDigit, isUppercase, isWhitespace, lines, splitOnce, splitWhitespace, startsWith, stripPrefix, stripSuffix, trim, utf8Len};
// meta-language:translated JavaScript import_statement items=1 sha256=a62927985f048836f224e77cfe4542331052b78a96b69c4e2bf285bcf25a86f0
// | import { UNITS_LINO } from '../generated/units-lino.js';
use crate::generated::units_lino::UNITS_LINO;
// meta-language:translated JavaScript import_statement items=1 sha256=7be64f87657035d0db20585dae0d588ad3f7226d5e25d33a3e0f2f4543387ac3
// | import { Dimension } from './dimension.js';
use crate::types::dimension::Dimension;
// meta-language:translated JavaScript import_statement items=1 sha256=58ef51ca7fd34d8fcac0fe77879a14dab404c6d6f9e30d9a4eadb2bbaa150a7c
// | import { Rational } from './rational.js';
use crate::types::rational::Rational;

// meta-language:carried JavaScript lexical_declaration (type)
// | /**
// |  * SI prefixes: symbol, word, power of ten. `u` and Greek `μ` are accepted
// |  * spellings of the micro sign `µ`.
// |  * @type {ReadonlyArray<readonly [string, string, number]>}
// |  */
// | const SI_PREFIXES = Object.freeze([
// |   ['Q', 'quetta', 30],
// |   ['R', 'ronna', 27],
// |   ['Y', 'yotta', 24],
// |   ['Z', 'zetta', 21],
// |   ['E', 'exa', 18],
// |   ['P', 'peta', 15],
// |   ['T', 'tera', 12],
// |   ['G', 'giga', 9],
// |   ['M', 'mega', 6],
// |   ['k', 'kilo', 3],
// |   ['h', 'hecto', 2],
// |   ['da', 'deca', 1],
// |   ['d', 'deci', -1],
// |   ['c', 'centi', -2],
// |   ['m', 'milli', -3],
// |   ['µ', 'micro', -6],
// |   ['n', 'nano', -9],
// |   ['p', 'pico', -12],
// |   ['f', 'femto', -15],
// |   ['a', 'atto', -18],
// |   ['z', 'zepto', -21],
// |   ['y', 'yocto', -24],
// |   ['r', 'ronto', -27],
// |   ['q', 'quecto', -30],
// | ]);

// meta-language:carried JavaScript lexical_declaration (type)
// | /**
// |  * Binary prefixes (IEC 80000-13): symbol, word, power of 1024.
// |  * @type {ReadonlyArray<readonly [string, string, number]>}
// |  */
// | const BINARY_PREFIXES = Object.freeze([
// |   ['Ki', 'kibi', 1],
// |   ['Mi', 'mebi', 2],
// |   ['Gi', 'gibi', 3],
// |   ['Ti', 'tebi', 4],
// |   ['Pi', 'pebi', 5],
// |   ['Ei', 'exbi', 6],
// |   ['Zi', 'zebi', 7],
// |   ['Yi', 'yobi', 8],
// | ]);

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** Which prefixes a unit accepts. */
// | class Prefixes {
// |   /** @param {string} tag @param {object} [fields] */
// |   constructor(tag, fields) {
// |     /** @type {'None' | 'Si' | 'SiBinary'} */
// |     this.tag = /** @type {any} */ (tag);
// |     /** `Si`: the allowed symbols, or `null` for all. @type {string[] | null} */
// |     this._0 = fields === undefined ? null : /** @type {{ _0: string[] | null }} */ (fields)._0;
// |     Object.freeze(this);
// |   }
// |
// |   static None = new Prefixes('None');
// |
// |   /**
// |    * SI prefixes, optionally restricted to the listed symbols.
// |    * @param {string[] | null} allowed
// |    */
// |   static Si(allowed) {
// |     return new Prefixes('Si', { _0: allowed });
// |   }
// |
// |   /** All SI prefixes plus binary prefixes. */
// |   static SiBinary = new Prefixes('SiBinary');
// |
// |   /**
// |    * @param {string} symbol
// |    * @returns {boolean}
// |    */
// |   allowsSi(symbol) {
// |     switch (this.tag) {
// |       case 'None':
// |         return false;
// |       case 'SiBinary':
// |         return true;
// |       default:
// |         return this._0 === null || this._0.some((allowed) => allowed === symbol);
// |     }
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** One unit of the table. */
// | export class UnitDef {
// |   /**
// |    * @param {{ symbol: string, aliases: string[], dimension: Dimension, factor: Rational,
// |    *   offset: Rational | null, expansion: Array<[string, number]> | null, prefixes: Prefixes }} fields
// |    */
// |   constructor({ symbol, aliases, dimension, factor, offset, expansion, prefixes }) {
// |     /** Canonical display symbol. */
// |     this.symbol = symbol;
// |     /** Every accepted spelling, including the symbol. */
// |     this.aliases = aliases;
// |     /** Physical dimension. */
// |     this.dimension = dimension;
// |     /** Factor to the coherent SI unit of the dimension. */
// |     this.factor = factor;
// |     /** Offset of an affine unit (`°C`, `°F`): base = (value + offset) × factor. */
// |     this.offset = offset;
// |     /**
// |      * Canonical unit terms of a definition made only of other units
// |      * (`mph` = `mi h^-1`), used to simplify products such as `mph·h`.
// |      */
// |     this.expansion = expansion;
// |     /** @internal */
// |     this.prefixes = prefixes;
// |   }
// |
// |   /**
// |    * Whether this is an affine (offset) unit such as `°C`.
// |    * @returns {boolean}
// |    */
// |   isAffine() {
// |     return this.offset !== null;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A unit name resolved against the table, including any prefix. */
// | export class ResolvedUnit {
// |   /** @param {{ def: UnitDef, symbol: string, factor: Rational }} fields */
// |   constructor({ def, symbol, factor }) {
// |     /** The unit definition. */
// |     this.def = def;
// |     /** Canonical symbol including the prefix (`km`, `µs`, `KiB`). */
// |     this.symbol = symbol;
// |     /** Factor to the coherent SI unit, including the prefix. */
// |     this.factor = factor;
// |   }
// |
// |   /**
// |    * The dimension of the unit.
// |    * @returns {Dimension}
// |    */
// |   dimension() {
// |     return this.def.dimension;
// |   }
// |
// |   /**
// |    * The affine offset, for unprefixed affine units.
// |    * @returns {Rational | null}
// |    */
// |   offset() {
// |     return this.def.offset;
// |   }
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {[number, string, Rational]} Resolution (unit index, canonical symbol, factor) */

// meta-language:carried JavaScript export_statement (unsupported)
// | /** The parsed unit table. */
// | export class UnitDatabase {
// |   constructor() {
// |     /** @internal @type {UnitDef[]} */
// |     this.units = [];
// |     /** Case-sensitive spellings to unit index. @type {Map<string, number>} */
// |     this.exact = new Map();
// |     /** Lowercase word aliases to unit index. @type {Map<string, number>} */
// |     this.words = new Map();
// |     /** Coherent named units products collapse into. @type {number[]} */
// |     this.named = [];
// |     /** Words that are never read as a prefixed unit. @type {string[]} */
// |     this.reserved = [];
// |   }
// |
// |   /**
// |    * `#[derive(Default)]`.
// |    * @returns {UnitDatabase}
// |    */
// |   static default() {
// |     return new UnitDatabase();
// |   }
// |
// |   /**
// |    * Resolves a name to (unit index, canonical symbol, factor).
// |    * @param {string} name
// |    * @returns {Resolution | null}
// |    */
// |   resolve(name) {
// |     const index = this.exact.get(name);
// |     if (index !== undefined) {
// |       const def = this.units[index];
// |       return [index, def.symbol, def.factor];
// |     }
// |     if (this.reserved.some((reserved) => reserved === name)) {
// |       return null;
// |     }
// |     return this.resolveUnreserved(name);
// |   }
// |
// |   /**
// |    * Resolves a name without the reserved-word check, so the canonical
// |    * symbol `pm` of `picometres` still resolves.
// |    * @param {string} name
// |    * @returns {Resolution | null}
// |    */
// |   resolveUnreserved(name) {
// |     const index = this.exact.get(name);
// |     if (index !== undefined) {
// |       const def = this.units[index];
// |       return [index, def.symbol, def.factor];
// |     }
// |     return (
// |       this.resolveSymbolPrefix(name) ??
// |       this.resolveWordPrefix(name.toLowerCase()) ??
// |       (() => {
// |         const allUpper = [...name].every(isUppercase);
// |         const lower = name.toLowerCase();
// |         const word = this.words.get(lower);
// |         if (word === undefined || allUpper) {
// |           return null;
// |         }
// |         const def = this.units[word];
// |         return /** @type {Resolution} */ ([word, def.symbol, def.factor]);
// |       })()
// |     );
// |   }
// |
// |   /**
// |    * Prefix symbol + unit symbol (`km`, `µs`, `uF`, `KiB`).
// |    * @param {string} name
// |    * @returns {Resolution | null}
// |    */
// |   resolveSymbolPrefix(name) {
// |     /** @type {Array<[string, number]>} */
// |     const microSpellings = [
// |       ['u', -6],
// |       ['μ', -6],
// |     ];
// |     const si = [...SI_PREFIXES.map(([symbol, , power]) => /** @type {[string, number]} */ ([symbol, power])), ...microSpellings];
// |     for (const [prefix, power] of si) {
// |       const rest = stripPrefix(name, prefix);
// |       if (rest === null) {
// |         continue;
// |       }
// |       const index = this.exact.get(rest);
// |       if (index === undefined) {
// |         continue;
// |       }
// |       const def = this.units[index];
// |       const canonical = normalizePrefix(prefix);
// |       if (!isWordAlias(rest, def.symbol) && !def.isAffine() && def.prefixes.allowsSi(canonical)) {
// |         const symbol = `${canonical}${def.symbol}`;
// |         return [index, symbol, def.factor.mul(tenPow(power))];
// |       }
// |     }
// |     for (const [prefix, , power] of BINARY_PREFIXES) {
// |       const rest = stripPrefix(name, prefix);
// |       if (rest === null) {
// |         continue;
// |       }
// |       const index = this.exact.get(rest);
// |       if (index === undefined) {
// |         continue;
// |       }
// |       const def = this.units[index];
// |       if (def.prefixes.tag === 'SiBinary' && !isWordAlias(rest, def.symbol)) {
// |         const factor = def.factor.mul(Rational.fromInteger(1024n ** BigInt(power)));
// |         return [index, `${prefix}${def.symbol}`, factor];
// |       }
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * Prefix word + word alias (`kilometres`, `microseconds`, `kibibits`).
// |    * @param {string} lower
// |    * @returns {Resolution | null}
// |    */
// |   resolveWordPrefix(lower) {
// |     for (const [symbol, word, power] of SI_PREFIXES) {
// |       const rest = stripPrefix(lower, word);
// |       if (rest === null) {
// |         continue;
// |       }
// |       const index = this.words.get(rest);
// |       if (index === undefined) {
// |         continue;
// |       }
// |       const def = this.units[index];
// |       if (def.prefixes.allowsSi(symbol) && !def.isAffine()) {
// |         const factor = def.factor.mul(tenPow(power));
// |         return [index, `${symbol}${def.symbol}`, factor];
// |       }
// |     }
// |     for (const [symbol, word, power] of BINARY_PREFIXES) {
// |       const rest = stripPrefix(lower, word);
// |       if (rest === null) {
// |         continue;
// |       }
// |       const index = this.words.get(rest);
// |       if (index === undefined) {
// |         continue;
// |       }
// |       const def = this.units[index];
// |       if (def.prefixes.tag === 'SiBinary') {
// |         const factor = def.factor.mul(Rational.fromInteger(1024n ** BigInt(power)));
// |         return [index, `${symbol}${def.symbol}`, factor];
// |       }
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * Throws `Err(String)` on a duplicate name.
// |    * @param {UnitDef} def
// |    */
// |   add(def) {
// |     const index = this.units.length;
// |     for (const alias of def.aliases) {
// |       const previous = this.exact.has(alias);
// |       this.exact.set(alias, index);
// |       if (previous) {
// |         err(`duplicate unit name '${alias}'`);
// |       }
// |       if (isWordAlias(alias, def.symbol)) {
// |         const key = alias.toLowerCase();
// |         if (!this.words.has(key)) {
// |           this.words.set(key, index);
// |         }
// |       }
// |     }
// |     this.units.push(def);
// |   }
// |
// |   /**
// |    * Evaluates a definition such as `kg m s^-2` or `1/1000 m^3`. Throws
// |    * `Err(String)`.
// |    * @param {string} definition
// |    * @returns {Evaluated}
// |    */
// |   evaluateDefinition(definition) {
// |     let dimension = Dimension.NONE;
// |     let factor = Rational.one();
// |     let sign = 1;
// |     /** @type {Array<[string, number]>} */
// |     const terms = [];
// |     let pure = true;
// |     for (const part of splitWhitespace(definition)) {
// |       if (part === '/') {
// |         sign = -1;
// |         continue;
// |       }
// |       const number = parseNumber(part);
// |       if (number !== null) {
// |         factor = factor.mul(number.powI32(sign));
// |         pure = false;
// |         continue;
// |       }
// |       const inner = stripPrefix(part, '[');
// |       const baseName = inner === null ? null : stripSuffix(inner, ']');
// |       if (baseName !== null) {
// |         const base = Dimension.base(baseName);
// |         if (base === null) {
// |           err(`unknown dimension '${baseName}'`);
// |         }
// |         dimension = dimension.mul(/** @type {Dimension} */ (base).pow(sign));
// |         pure = false;
// |         continue;
// |       }
// |       const split = splitOnce(part, '^');
// |       /** @type {string} */
// |       let name;
// |       /** @type {number} */
// |       let power;
// |       if (split !== null) {
// |         name = split[0];
// |         const parsed = ok(() => I32.parse(split[1]));
// |         if (parsed === null) {
// |           err(`invalid power in '${part}'`);
// |         }
// |         power = Number(parsed);
// |       } else {
// |         name = part;
// |         power = 1;
// |       }
// |       const resolved = this.resolve(name);
// |       if (resolved === null) {
// |         err(`unknown unit '${name}'`);
// |       }
// |       const [index, symbol, unitFactor] = /** @type {Resolution} */ (resolved);
// |       power = Math.imul(power, sign);
// |       dimension = dimension.mul(this.units[index].dimension.pow(power));
// |       factor = factor.mul(unitFactor.powI32(power));
// |       terms.push([symbol, power]);
// |     }
// |     const expansion = pure && !(terms.length === 1 && terms[0][1] === 1) ? terms : null;
// |     return [dimension, factor, expansion];
// |   }
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {UnitDatabase | null} */
// | let UNITS = null;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Returns the bundled unit database.
// |  * @returns {UnitDatabase}
// |  */
// | export function units() {
// |   UNITS ??= expectResult(() => parseDatabase(UNITS_LINO), 'data/units/units.lino is valid');
// |   return UNITS;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Looks up a unit name in the bundled database.
// |  * @param {string} name
// |  * @returns {ResolvedUnit | null}
// |  */
// | export function lookupUnit(name) {
// |   const database = units();
// |   const resolved = database.resolve(name);
// |   if (resolved === null) {
// |     return null;
// |   }
// |   const [index, symbol, factor] = resolved;
// |   return new ResolvedUnit({ def: database.units[index], symbol, factor });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Looks up a canonical symbol produced by [`lookup_unit`], including
// |  * prefixed symbols that are reserved as typed words (`pm`, `am`).
// |  * @param {string} symbol
// |  * @returns {ResolvedUnit | null}
// |  */
// | export function lookupSymbol(symbol) {
// |   const database = units();
// |   const resolved = database.resolveUnreserved(symbol);
// |   if (resolved === null) {
// |     return null;
// |   }
// |   const [index, canonical, factor] = resolved;
// |   return new ResolvedUnit({ def: database.units[index], symbol: canonical, factor });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Returns the coherent named unit (`N`, `J`, `V`, …) of a dimension.
// |  * @param {Dimension} dimension
// |  * @returns {string | null}
// |  */
// | export function namedUnitFor(dimension) {
// |   const database = units();
// |   const def = database.named.map((index) => database.units[index]).find((def) => eq(def.dimension, dimension));
// |   return def === undefined ? null : def.symbol;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Whether an alias is a word (`metre`, `feet`) rather than a symbol (`m`, `ft`).
// |  * @param {string} alias
// |  * @param {string} symbol
// |  * @returns {boolean}
// |  */
// | function isWordAlias(alias, symbol) {
// |   const chars = [...alias];
// |   return alias !== symbol && chars.length >= 3 && chars.every(isAlphabetic);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} prefix
// |  * @returns {string}
// |  */
// | function normalizePrefix(prefix) {
// |   switch (prefix) {
// |     case 'u':
// |     case 'μ':
// |       return 'µ';
// |     default:
// |       return prefix;
// |   }
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {number} exponent
// |  * @returns {Rational}
// |  */
// | function tenPow(exponent) {
// |   return Rational.fromInteger(10n).powI32(exponent);
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /** @typedef {[Dimension, Rational, Array<[string, number]> | null]} Evaluated Dimension, factor and optional expansion of a definition. */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses `1000`, `0.3048`, `1.602176634e-19` or `5/9` exactly.
// |  * @internal
// |  * @param {string} text
// |  * @returns {Rational | null}
// |  */
// | export function parseNumber(text) {
// |   const fractionSplit = splitOnce(text, '/');
// |   if (fractionSplit !== null) {
// |     const [numerText, denomText] = fractionSplit;
// |     const denom = parseNumber(denomText);
// |     if (denom === null || denom.isZero()) {
// |       return null;
// |     }
// |     const numer = parseNumber(numerText);
// |     return numer === null ? null : numer.div(denom);
// |   }
// |   /** @type {string} */
// |   let mantissa;
// |   /** @type {number} */
// |   let exponent;
// |   const exponentSplit = splitOnce(text, ['e', 'E']);
// |   if (exponentSplit !== null) {
// |     mantissa = exponentSplit[0];
// |     const parsed = ok(() => I32.parse(exponentSplit[1]));
// |     if (parsed === null) {
// |       return null;
// |     }
// |     exponent = Number(parsed);
// |   } else {
// |     mantissa = text;
// |     exponent = 0;
// |   }
// |   const [whole, fraction] = splitOnce(mantissa, '.') ?? [mantissa, ''];
// |   const digits = `${whole}${fraction}`;
// |   if (digits === '' || ![...digits].every((c) => c.length === 1 && isAsciiDigit(c))) {
// |     return null;
// |   }
// |   const numer = ok(() => bigintFromStr(digits));
// |   if (numer === null) {
// |     return null;
// |   }
// |   const scale = utf8Len(fraction);
// |   return Rational.fromBigint(numer).mul(tenPow(exponent - scale));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Splits a link `(kind a 'b c' "d")` into its parts. Throws `Err(String)`.
// |  * @param {string} line
// |  * @returns {string[]}
// |  */
// | function splitLink(line) {
// |   const open = stripPrefix(line, '(');
// |   const inner = open === null ? null : stripSuffix(open, ')');
// |   if (inner === null) {
// |     return err('expected a link in parentheses');
// |   }
// |   const parts = [];
// |   const chars = [...inner];
// |   let position = 0;
// |   while (position < chars.length) {
// |     const ch = chars[position];
// |     if (isWhitespace(ch)) {
// |       position += 1;
// |     } else if (ch === "'" || ch === '"') {
// |       position += 1;
// |       let part = '';
// |       while (true) {
// |         if (position >= chars.length) {
// |           return err('unterminated quote');
// |         }
// |         const c = chars[position];
// |         position += 1;
// |         if (c === ch) {
// |           break;
// |         }
// |         part += c;
// |       }
// |       parts.push(part);
// |     } else {
// |       let part = '';
// |       while (position < chars.length) {
// |         const c = chars[position];
// |         if (isWhitespace(c)) {
// |           break;
// |         }
// |         part += c;
// |         position += 1;
// |       }
// |       parts.push(part);
// |     }
// |   }
// |   return parts;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Throws `Err(String)`.
// |  * @param {string} text
// |  * @returns {Prefixes}
// |  */
// | function parsePrefixes(text) {
// |   switch (text) {
// |     case 'none':
// |       return Prefixes.None;
// |     case 'si':
// |       return Prefixes.Si(null);
// |     case 'si-binary':
// |       return Prefixes.SiBinary;
// |     default: {
// |       const list = stripPrefix(text, 'si:');
// |       if (list === null) {
// |         return err(`unknown prefix class '${text}'`);
// |       }
// |       return Prefixes.Si(list.split(','));
// |     }
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses the unit table. Throws `Err(String)`.
// |  * @internal
// |  * @param {string} content
// |  * @returns {UnitDatabase}
// |  */
// | export function parseDatabase(content) {
// |   const database = UnitDatabase.default();
// |   const allLines = lines(content);
// |   for (let lineIndex = 0; lineIndex < allLines.length; lineIndex += 1) {
// |     const line = trim(allLines[lineIndex]);
// |     if (line === '' || startsWith(line, '//')) {
// |       continue;
// |     }
// |     /** @param {string} message */
// |     const atLine = (message) => `line ${lineIndex + 1}: ${message}`;
// |     /**
// |      * `.map_err(at_line)?`
// |      * @template T
// |      * @param {() => T} fn
// |      * @returns {T}
// |      */
// |     const mapErr = (fn) => {
// |       const result = tryResult(fn);
// |       if (!result.ok) {
// |         return err(atLine(result.error.error));
// |       }
// |       return /** @type {T} */ (result.value);
// |     };
// |     const parts = mapErr(() => splitLink(line));
// |     if (parts.length === 0) {
// |       return err(atLine('empty link'));
// |     }
// |     const [kind, ...rest] = parts;
// |     switch (kind) {
// |       case 'unit':
// |       case 'affine': {
// |         /** @type {Prefixes} */
// |         let prefixes;
// |         /** @type {string} */
// |         let definition;
// |         /** @type {Rational | null} */
// |         let offset;
// |         /** @type {string[]} */
// |         let names;
// |         if (kind === 'unit') {
// |           if (rest.length < 2) {
// |             return err(atLine('expected prefixes and a definition'));
// |           }
// |           prefixes = mapErr(() => parsePrefixes(rest[0]));
// |           definition = rest[1];
// |           offset = null;
// |           names = rest.slice(2);
// |         } else {
// |           if (rest.length < 2) {
// |             return err(atLine('expected a definition and an offset'));
// |           }
// |           definition = rest[0];
// |           offset = parseNumber(rest[1]);
// |           if (offset === null) {
// |             return err(atLine(`invalid offset '${rest[1]}'`));
// |           }
// |           prefixes = Prefixes.None;
// |           names = rest.slice(2);
// |         }
// |         if (names.length === 0) {
// |           return err(atLine('expected a symbol'));
// |         }
// |         const symbol = names[0];
// |         const [dimension, factor, expansion] = mapErr(() => database.evaluateDefinition(definition));
// |         mapErr(() =>
// |           database.add(
// |             new UnitDef({
// |               symbol,
// |               aliases: [...names],
// |               dimension,
// |               factor,
// |               offset,
// |               expansion,
// |               prefixes,
// |             }),
// |           ),
// |         );
// |         break;
// |       }
// |       case 'named':
// |         for (const symbol of rest) {
// |           const index = database.exact.get(symbol);
// |           if (index === undefined) {
// |             return err(atLine(`unknown unit '${symbol}'`));
// |           }
// |           database.named.push(index);
// |         }
// |         break;
// |       case 'reserved':
// |         database.reserved.push(...rest);
// |         break;
// |       default:
// |         return err(atLine(`unknown link kind '${kind}'`));
// |     }
// |   }
// |   return database;
// | }
