// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=60711b0dc5d69dd41a169f4d2bb9ee9a51f668a5400fcc74bde4a20c8a34129d bytes=9095

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * chrono 0.4.45 format items (src/format/mod.rs `Item`, `Numeric`, `Fixed`,
// |  * `Pad`) and the `StrftimeItems` tokenizer (src/format/strftime.rs), without
// |  * the `unstable-locales` feature.
// |  */

// meta-language:carried JavaScript import_statement (unsupported)
// | import { isWhitespace } from '../str.js';

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * A format item. `Numeric` items carry the field name and the padding;
// |  * `Fixed` items carry the `Fixed` (or `InternalInternal`) variant name.
// |  * @typedef {{ tag: 'Literal', value: string }
// |  *   | { tag: 'Space', value: string }
// |  *   | { tag: 'Numeric', numeric: string, pad: 'None' | 'Zero' | 'Space' }
// |  *   | { tag: 'Fixed', fixed: string }
// |  *   | { tag: 'Error' }} Item
// |  */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} value
// |  * @returns {Item}
// |  */
// | function literal(value) {
// |   return Object.freeze({ tag: 'Literal', value });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} value
// |  * @returns {Item}
// |  */
// | function space(value) {
// |   return Object.freeze({ tag: 'Space', value });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `num`: a numeric item without padding.
// |  * @param {string} numeric
// |  * @returns {Item}
// |  */
// | function num(numeric) {
// |   return Object.freeze({ tag: 'Numeric', numeric, pad: 'None' });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `num0`: a zero-padded numeric item.
// |  * @param {string} numeric
// |  * @returns {Item}
// |  */
// | function num0(numeric) {
// |   return Object.freeze({ tag: 'Numeric', numeric, pad: 'Zero' });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `nums`: a space-padded numeric item.
// |  * @param {string} numeric
// |  * @returns {Item}
// |  */
// | function nums(numeric) {
// |   return Object.freeze({ tag: 'Numeric', numeric, pad: 'Space' });
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `fixed` / `internal_fixed`.
// |  * @param {string} name
// |  * @returns {Item}
// |  */
// | function fixed(name) {
// |   return Object.freeze({ tag: 'Fixed', fixed: name });
// | }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `Item::Error`. */
// | const ERROR = /** @type {Item} */ (Object.freeze({ tag: 'Error' }));

// meta-language:carried JavaScript lexical_declaration (type)
// | /** `D_FMT` (`%x`). */
// | const D_FMT = [num0('Month'), literal('/'), num0('Day'), literal('/'), num0('YearMod100')];
// meta-language:carried JavaScript lexical_declaration (type)
// | /** `D_T_FMT` (`%c`). */
// | const D_T_FMT = [
// |   fixed('ShortWeekdayName'), space(' '), fixed('ShortMonthName'), space(' '), nums('Day'),
// |   space(' '), num0('Hour'), literal(':'), num0('Minute'), literal(':'), num0('Second'),
// |   space(' '), num0('Year'),
// | ];
// meta-language:carried JavaScript lexical_declaration (type)
// | /** `T_FMT` (`%X`). */
// | const T_FMT = [num0('Hour'), literal(':'), num0('Minute'), literal(':'), num0('Second')];
// meta-language:carried JavaScript lexical_declaration (type)
// | /** `T_FMT_AMPM` (`%r`). */
// | const T_FMT_AMPM = [
// |   num0('Hour12'), literal(':'), num0('Minute'), literal(':'), num0('Second'), space(' '),
// |   fixed('UpperAmPm'),
// | ];

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** Single-item specifiers: `%A`, `%B`, ... */
// | const SIMPLE = new Map([
// |   ['A', fixed('LongWeekdayName')],
// |   ['B', fixed('LongMonthName')],
// |   ['C', num0('YearDiv100')],
// |   ['G', num0('IsoYear')],
// |   ['H', num0('Hour')],
// |   ['I', num0('Hour12')],
// |   ['M', num0('Minute')],
// |   ['P', fixed('LowerAmPm')],
// |   ['S', num0('Second')],
// |   ['U', num0('WeekFromSun')],
// |   ['V', num0('IsoWeek')],
// |   ['W', num0('WeekFromMon')],
// |   ['Y', num0('Year')],
// |   ['Z', fixed('TimezoneName')],
// |   ['a', fixed('ShortWeekdayName')],
// |   ['b', fixed('ShortMonthName')],
// |   ['h', fixed('ShortMonthName')],
// |   ['d', num0('Day')],
// |   ['e', nums('Day')],
// |   ['f', num0('Nanosecond')],
// |   ['g', num0('IsoYearMod100')],
// |   ['j', num0('Ordinal')],
// |   ['k', nums('Hour')],
// |   ['l', nums('Hour12')],
// |   ['m', num0('Month')],
// |   ['n', space('\n')],
// |   ['p', fixed('UpperAmPm')],
// |   ['q', num('Quarter')],
// |   ['s', num('Timestamp')],
// |   ['t', space('\t')],
// |   ['u', num('WeekdayFromMon')],
// |   ['w', num('NumDaysFromSun')],
// |   ['y', num0('YearMod100')],
// |   ['+', fixed('RFC3339')],
// |   ['%', literal('%')],
// | ]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** Specifiers that expand to several items (`queue!` / `queue_from_slice!`). */
// | const QUEUED = new Map([
// |   ['D', [num0('Month'), literal('/'), num0('Day'), literal('/'), num0('YearMod100')]],
// |   ['F', [num0('Year'), literal('-'), num0('Month'), literal('-'), num0('Day')]],
// |   ['R', [num0('Hour'), literal(':'), num0('Minute')]],
// |   ['T', [num0('Hour'), literal(':'), num0('Minute'), literal(':'), num0('Second')]],
// |   ['X', T_FMT],
// |   ['c', D_T_FMT],
// |   ['r', T_FMT_AMPM],
// |   ['v', [nums('Day'), literal('-'), fixed('ShortMonthName'), literal('-'), num0('Year')]],
// |   ['x', D_FMT],
// | ]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** `Fixed` items selected by `%.Nf` and the internal `%Nf`. */
// | const DOT_NANOS = new Map([['3', 'Nanosecond3'], ['6', 'Nanosecond6'], ['9', 'Nanosecond9']]);
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const NO_DOT_NANOS = new Map([
// |   ['3', 'Nanosecond3NoDot'], ['6', 'Nanosecond6NoDot'], ['9', 'Nanosecond9NoDot'],
// | ]);

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The first character (code point) of `s`, or `null` when it is empty.
// |  * @param {string} s
// |  */
// | function firstChar(s) {
// |   const code = s.codePointAt(0);
// |   return code === undefined ? null : String.fromCodePoint(code);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The UTF-16 index of the first character of `s` matching `predicate`, or
// |  * `s.length`.
// |  * @param {string} s
// |  * @param {(c: string) => boolean} predicate
// |  */
// | function findChar(s, predicate) {
// |   let index = 0;
// |   for (const c of s) {
// |     if (predicate(c)) {
// |       return index;
// |     }
// |     index += c.length;
// |   }
// |   return s.length;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * chrono `StrftimeItems`: a lazy iterator of format `Item`s parsed from a
// |  * strftime-like format string. Errors show up as `Item::Error` items, so a
// |  * consumer sees them only when it gets that far.
// |  * @implements {Iterable<Item>}
// |  */
// | export class StrftimeItems {
// |   /** @param {string} s the format string */
// |   constructor(s) {
// |     /** The unparsed rest of the format string. */
// |     this.remainder = s;
// |     /**
// |      * Items of an expanded specifier still to be returned.
// |      * @type {Item[]}
// |      */
// |     this.queue = [];
// |   }
// |
// |   /**
// |    * `StrftimeItems::new`.
// |    * @param {string} s
// |    */
// |   static new(s) {
// |     return new StrftimeItems(s);
// |   }
// |
// |   /**
// |    * `Iterator::next`.
// |    * @returns {Item | null}
// |    */
// |   next() {
// |     if (this.queue.length > 0) {
// |       return /** @type {Item} */ (this.queue.shift());
// |     }
// |     const result = this.parseNextItem(this.remainder);
// |     if (result === null) {
// |       return null;
// |     }
// |     this.remainder = result[0];
// |     return result[1];
// |   }
// |
// |   /** @returns {Iterator<Item>} */
// |   *[Symbol.iterator]() {
// |     for (let item = this.next(); item !== null; item = this.next()) {
// |       yield item;
// |     }
// |   }
// |
// |   /**
// |    * `parse_next_item`: the remainder after the next item and the item.
// |    * @param {string} input
// |    * @returns {[string, Item] | null}
// |    */
// |   parseNextItem(input) {
// |     const first = firstChar(input);
// |     if (first === null) {
// |       return null;
// |     }
// |     if (first !== '%') {
// |       if (isWhitespace(first)) {
// |         const nextspec = findChar(input, (c) => !isWhitespace(c));
// |         return [input.slice(nextspec), space(input.slice(0, nextspec))];
// |       }
// |       const nextspec = findChar(input, (c) => isWhitespace(c) || c === '%');
// |       return [input.slice(nextspec), literal(input.slice(0, nextspec))];
// |     }
// |     let remainder = input.slice(1);
// |     /** `next!()`: the next character, or `null` at the premature end. */
// |     const next = () => {
// |       const c = firstChar(remainder);
// |       if (c !== null) {
// |         remainder = remainder.slice(c.length);
// |       }
// |       return c;
// |     };
// |     let spec = next();
// |     if (spec === null) {
// |       return [remainder, ERROR];
// |     }
// |     /** @type {'None' | 'Zero' | 'Space' | null} */
// |     const padOverride = spec === '-' ? 'None' : spec === '0' ? 'Zero' : spec === '_' ? 'Space' : null;
// |     const isAlternate = spec === '#';
// |     if (padOverride !== null || isAlternate) {
// |       spec = next();
// |       if (spec === null) {
// |         return [remainder, ERROR];
// |       }
// |     }
// |     if (isAlternate && spec !== 'z') {
// |       return [remainder, ERROR];
// |     }
// |     /** @type {Item} */
// |     let item;
// |     const queued = QUEUED.get(spec);
// |     if (queued !== undefined) {
// |       this.queue = queued.slice(1);
// |       item = queued[0];
// |     } else if (spec === 'z') {
// |       item = fixed(isAlternate ? 'TimezoneOffsetPermissive' : 'TimezoneOffset');
// |     } else if (spec === ':') {
// |       if (remainder.startsWith('::z')) {
// |         remainder = remainder.slice(3);
// |         item = fixed('TimezoneOffsetTripleColon');
// |       } else if (remainder.startsWith(':z')) {
// |         remainder = remainder.slice(2);
// |         item = fixed('TimezoneOffsetDoubleColon');
// |       } else if (remainder.startsWith('z')) {
// |         remainder = remainder.slice(1);
// |         item = fixed('TimezoneOffsetColon');
// |       } else {
// |         item = ERROR;
// |       }
// |     } else if (spec === '.') {
// |       const c = next();
// |       if (c === null) {
// |         return [remainder, ERROR];
// |       }
// |       const name = DOT_NANOS.get(c);
// |       if (name !== undefined) {
// |         const f = next();
// |         if (f === null) {
// |           return [remainder, ERROR];
// |         }
// |         item = f === 'f' ? fixed(name) : ERROR;
// |       } else {
// |         item = c === 'f' ? fixed('Nanosecond') : ERROR;
// |       }
// |     } else if (NO_DOT_NANOS.has(spec)) {
// |       const f = next();
// |       if (f === null) {
// |         return [remainder, ERROR];
// |       }
// |       item = f === 'f' ? fixed(/** @type {string} */ (NO_DOT_NANOS.get(spec))) : ERROR;
// |     } else {
// |       item = SIMPLE.get(spec) ?? ERROR;
// |     }
// |     if (padOverride !== null) {
// |       if (item.tag === 'Numeric' && this.queue.length === 0) {
// |         return [remainder, Object.freeze({ tag: 'Numeric', numeric: item.numeric, pad: padOverride })];
// |       }
// |       return [remainder, ERROR];
// |     }
// |     return [remainder, item];
// |   }
// | }
