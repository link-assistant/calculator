// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4dcfe272f372a983984a3600bf0fe746afaf5d16fc258bf6ef316e9721861b48 bytes=18188

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Rust `str` and `char` semantics that JavaScript strings do not share:
// |  * Unicode `White_Space` trimming, the `char::is_*` classes, UTF-8 byte
// |  * offsets, `lines`, literal `replace` and the `Debug` escaping of strings.
// |  *
// |  * A Rust `char` is a one-code-point JavaScript string. See js/PORTING.md.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=01fef6aeb8c3f94e87fffc4e5d4b41c777da316211307cdc3b699dfa20824354
// | import { panic, unreachable } from './core.js';
use crate::rust::core_::{panic, unreachable};

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const WHITE_SPACE = /^\p{White_Space}$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const ALPHABETIC = /^\p{Alphabetic}$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const NUMERIC = /^\p{N}$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const LOWERCASE = /^\p{Lowercase}$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const UPPERCASE = /^\p{Uppercase}$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /**
// |  * The chars `{:?}` escapes as `\u{..}` (`char::escape_debug_ext`): control,
// |  * private use, `White_Space`, `Grapheme_Extend`, `Default_Ignorable_Code_Point`,
// |  * format (`Cf`) and unassigned chars.
// |  */
// | const ESCAPED_IN_DEBUG = /^[\p{Cc}\p{Co}\p{White_Space}\p{Grapheme_Extend}\p{Default_Ignorable_Code_Point}\p{Cf}\p{Cn}]$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const LEADING_WHITE_SPACE = /^\p{White_Space}+/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const TRAILING_WHITE_SPACE = /\p{White_Space}+$/u;
// meta-language:carried JavaScript lexical_declaration (unsupported)
// | const WHITE_SPACE_RUN = /\p{White_Space}+/u;

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_whitespace` (Unicode `White_Space`).
// |  * @param {string} c
// |  */
// | export function isWhitespace(c) {
// |   return WHITE_SPACE.test(c);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_alphabetic` (Unicode `Alphabetic`).
// |  * @param {string} c
// |  */
// | export function isAlphabetic(c) {
// |   return ALPHABETIC.test(c);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_numeric` (general categories `Nd`, `Nl` and `No`).
// |  * @param {string} c
// |  */
// | export function isNumeric(c) {
// |   return NUMERIC.test(c);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_alphanumeric`.
// |  * @param {string} c
// |  */
// | export function isAlphanumeric(c) {
// |   return isAlphabetic(c) || isNumeric(c);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_lowercase` (Unicode `Lowercase`).
// |  * @param {string} c
// |  */
// | export function isLowercase(c) {
// |   return LOWERCASE.test(c);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_uppercase` (Unicode `Uppercase`).
// |  * @param {string} c
// |  */
// | export function isUppercase(c) {
// |   return UPPERCASE.test(c);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::is_ascii_digit`.
// |  * @param {string} c
// |  */
// | export function isAsciiDigit(c) {
// |   return c.length === 1 && c >= '0' && c <= '9';
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::is_ascii_alphabetic`.
// |  * @param {string} c
// |  */
// | export function isAsciiAlphabetic(c) {
// |   return c.length === 1 && ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::is_ascii_uppercase`.
// |  * @param {string} c
// |  */
// | export function isAsciiUppercase(c) {
// |   return c.length === 1 && c >= 'A' && c <= 'Z';
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::is_ascii_lowercase`.
// |  * @param {string} c
// |  */
// | export function isAsciiLowercase(c) {
// |   return c.length === 1 && c >= 'a' && c <= 'z';
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `char::is_ascii_alphanumeric`.
// |  * @param {string} c
// |  */
// | export function isAsciiAlphanumeric(c) {
// |   return isAsciiDigit(c) || isAsciiAlphabetic(c);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::to_digit(radix)`: ASCII digits and letters only.
// |  * @param {string} c
// |  * @param {number} radix 2..=36
// |  * @returns {number | null}
// |  */
// | export function toDigit(c, radix) {
// |   if (c.length !== 1) {
// |     return null;
// |   }
// |   const code = c.charCodeAt(0);
// |   let digit = 99;
// |   if (code >= 48 && code <= 57) {
// |     digit = code - 48;
// |   } else if (code >= 97 && code <= 122) {
// |     digit = code - 87;
// |   } else if (code >= 65 && code <= 90) {
// |     digit = code - 55;
// |   }
// |   return digit < radix ? digit : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::is_digit(radix)`.
// |  * @param {string} c
// |  * @param {number} radix
// |  */
// | export function isDigit(c, radix) {
// |   return toDigit(c, radix) !== null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::from_digit(num, radix)`: lowercase letters above 9.
// |  * @param {number} num
// |  * @param {number} radix
// |  * @returns {string | null}
// |  */
// | export function fromDigit(num, radix) {
// |   return num >= 0 && num < radix ? num.toString(radix) : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::to_ascii_lowercase` / `char::to_ascii_lowercase`.
// |  * @param {string} s
// |  */
// | export function toAsciiLowercase(s) {
// |   return s.replace(/[A-Z]+/g, (run) => run.toLowerCase());
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::to_ascii_uppercase` / `char::to_ascii_uppercase`.
// |  * @param {string} s
// |  */
// | export function toAsciiUppercase(s) {
// |   return s.replace(/[a-z]+/g, (run) => run.toUpperCase());
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::eq_ignore_ascii_case`.
// |  * @param {string} a
// |  * @param {string} b
// |  */
// | export function eqIgnoreAsciiCase(a, b) {
// |   return a.length === b.length && toAsciiLowercase(a) === toAsciiLowercase(b);
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `str::trim` (Unicode `White_Space`; JavaScript's `trim` differs on U+0085
// |  * and U+FEFF).
// |  * @param {string} s
// |  */
// | export function trim(s) {
// |   return trimEnd(trimStart(s));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::trim_start`.
// |  * @param {string} s
// |  */
// | export function trimStart(s) {
// |   return s.replace(LEADING_WHITE_SPACE, '');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::trim_end`.
// |  * @param {string} s
// |  */
// | export function trimEnd(s) {
// |   return s.replace(TRAILING_WHITE_SPACE, '');
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * A Rust string pattern: a `&str`, a `char` predicate (`char::is_numeric`,
// |  * `|c| ..`) or a slice of chars (`&['a', 'b']`). A one-character string is a
// |  * `char` pattern, which matches the same text as the equal `&str`.
// |  * @typedef {string | ((c: string) => boolean) | string[]} Pattern
// |  */

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The predicate of a char pattern, or null for a `&str` pattern.
// |  * @param {Pattern} pattern
// |  * @returns {((c: string) => boolean) | null}
// |  */
// | function charPredicate(pattern) {
// |   if (typeof pattern === 'function') {
// |     return pattern;
// |   }
// |   if (Array.isArray(pattern)) {
// |     return (c) => pattern.includes(c);
// |   }
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::trim_start_matches(pattern)`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function trimStartMatches(s, pattern) {
// |   const predicate = charPredicate(pattern);
// |   if (predicate === null) {
// |     const text = /** @type {string} */ (pattern);
// |     if (text === '') {
// |       return s;
// |     }
// |     let start = 0;
// |     while (s.startsWith(text, start)) {
// |       start += text.length;
// |     }
// |     return s.slice(start);
// |   }
// |   let start = 0;
// |   for (const c of s) {
// |     if (!predicate(c)) {
// |       break;
// |     }
// |     start += c.length;
// |   }
// |   return s.slice(start);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::trim_end_matches(pattern)`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function trimEndMatches(s, pattern) {
// |   const predicate = charPredicate(pattern);
// |   if (predicate === null) {
// |     const text = /** @type {string} */ (pattern);
// |     if (text === '') {
// |       return s;
// |     }
// |     let end = s.length;
// |     while (end >= text.length && s.startsWith(text, end - text.length)) {
// |       end -= text.length;
// |     }
// |     return s.slice(0, end);
// |   }
// |   const chars = [...s];
// |   let end = chars.length;
// |   while (end > 0 && predicate(chars[end - 1])) {
// |     end -= 1;
// |   }
// |   return chars.slice(0, end).join('');
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `str::trim_matches(pattern)` for char patterns.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function trimMatches(s, pattern) {
// |   return trimEndMatches(trimStartMatches(s, pattern), pattern);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::strip_prefix`.
// |  * @param {string} s
// |  * @param {Pattern} prefix
// |  * @returns {string | null}
// |  */
// | export function stripPrefix(s, prefix) {
// |   const predicate = charPredicate(prefix);
// |   if (predicate === null) {
// |     const text = /** @type {string} */ (prefix);
// |     return s.startsWith(text) ? s.slice(text.length) : null;
// |   }
// |   const first = s.codePointAt(0);
// |   if (first === undefined) {
// |     return null;
// |   }
// |   const c = String.fromCodePoint(first);
// |   return predicate(c) ? s.slice(c.length) : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::strip_suffix`.
// |  * @param {string} s
// |  * @param {Pattern} suffix
// |  * @returns {string | null}
// |  */
// | export function stripSuffix(s, suffix) {
// |   const predicate = charPredicate(suffix);
// |   if (predicate === null) {
// |     const text = /** @type {string} */ (suffix);
// |     return s.endsWith(text) ? s.slice(0, s.length - text.length) : null;
// |   }
// |   const chars = [...s];
// |   if (chars.length === 0 || !predicate(chars[chars.length - 1])) {
// |     return null;
// |   }
// |   return chars.slice(0, -1).join('');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::starts_with` for any pattern.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function startsWith(s, pattern) {
// |   return stripPrefix(s, pattern) !== null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::ends_with` for any pattern.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function endsWith(s, pattern) {
// |   return stripSuffix(s, pattern) !== null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::find(pattern)`: the UTF-16 index of the first match, or null. Use
// |  * `indexToByte` when Rust prints the position.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {number | null}
// |  */
// | export function find(s, pattern) {
// |   const predicate = charPredicate(pattern);
// |   if (predicate === null) {
// |     const index = s.indexOf(/** @type {string} */ (pattern));
// |     return index === -1 ? null : index;
// |   }
// |   let index = 0;
// |   for (const c of s) {
// |     if (predicate(c)) {
// |       return index;
// |     }
// |     index += c.length;
// |   }
// |   return null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::rfind(pattern)`: the UTF-16 index of the last match, or null.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {number | null}
// |  */
// | export function rfind(s, pattern) {
// |   const predicate = charPredicate(pattern);
// |   if (predicate === null) {
// |     const index = s.lastIndexOf(/** @type {string} */ (pattern));
// |     return index === -1 ? null : index;
// |   }
// |   let found = null;
// |   let index = 0;
// |   for (const c of s) {
// |     if (predicate(c)) {
// |       found = index;
// |     }
// |     index += c.length;
// |   }
// |   return found;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::contains` for any pattern.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function contains(s, pattern) {
// |   return find(s, pattern) !== null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The match boundaries of a pattern: `[start, end]` UTF-16 ranges, left to
// |  * right, non-overlapping, as Rust's `match_indices`. An empty `&str` matches
// |  * at every char boundary.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {Array<[number, number]>}
// |  */
// | function matchRanges(s, pattern) {
// |   /** @type {Array<[number, number]>} */
// |   const ranges = [];
// |   const predicate = charPredicate(pattern);
// |   if (predicate === null) {
// |     const text = /** @type {string} */ (pattern);
// |     if (text === '') {
// |       let index = 0;
// |       for (const c of s) {
// |         ranges.push([index, index]);
// |         index += c.length;
// |       }
// |       ranges.push([s.length, s.length]);
// |       return ranges;
// |     }
// |     let index = s.indexOf(text);
// |     while (index !== -1) {
// |       ranges.push([index, index + text.length]);
// |       index = s.indexOf(text, index + text.length);
// |     }
// |     return ranges;
// |   }
// |   let index = 0;
// |   for (const c of s) {
// |     if (predicate(c)) {
// |       ranges.push([index, index + c.length]);
// |     }
// |     index += c.length;
// |   }
// |   return ranges;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::split(pattern)`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {string[]}
// |  */
// | export function split(s, pattern) {
// |   const parts = [];
// |   let start = 0;
// |   for (const [from, to] of matchRanges(s, pattern)) {
// |     parts.push(s.slice(start, from));
// |     start = to;
// |   }
// |   parts.push(s.slice(start));
// |   return parts;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::splitn(n, pattern)`.
// |  * @param {string} s
// |  * @param {number} n
// |  * @param {Pattern} pattern
// |  * @returns {string[]}
// |  */
// | export function splitn(s, n, pattern) {
// |   if (n === 0) {
// |     return [];
// |   }
// |   const parts = [];
// |   let start = 0;
// |   for (const [from, to] of matchRanges(s, pattern)) {
// |     if (parts.length === n - 1) {
// |       break;
// |     }
// |     parts.push(s.slice(start, from));
// |     start = to;
// |   }
// |   parts.push(s.slice(start));
// |   return parts;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::split_once(pattern)`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {[string, string] | null}
// |  */
// | export function splitOnce(s, pattern) {
// |   const [range] = matchRanges(s, pattern);
// |   return range === undefined ? null : [s.slice(0, range[0]), s.slice(range[1])];
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::rsplit_once(pattern)`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @returns {[string, string] | null}
// |  */
// | export function rsplitOnce(s, pattern) {
// |   const ranges = matchRanges(s, pattern);
// |   const range = ranges[ranges.length - 1];
// |   return range === undefined ? null : [s.slice(0, range[0]), s.slice(range[1])];
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `str::matches(pattern).count()`.
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  */
// | export function countMatches(s, pattern) {
// |   return matchRanges(s, pattern).length;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::replace(pattern, to)`: replaces every match, literally (JavaScript's
// |  * `replace` replaces the first match and expands `$&` in `to`).
// |  * @param {string} s
// |  * @param {Pattern} pattern
// |  * @param {string} to
// |  */
// | export function replace(s, pattern, to) {
// |   return split(s, pattern).join(to);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::split_whitespace`.
// |  * @param {string} s
// |  * @returns {string[]}
// |  */
// | export function splitWhitespace(s) {
// |   return s.split(WHITE_SPACE_RUN).filter((part) => part !== '');
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::lines`: split at `\n`, drop one `\r` before it, no final empty line.
// |  * @param {string} s
// |  * @returns {string[]}
// |  */
// | export function lines(s) {
// |   if (s === '') {
// |     return [];
// |   }
// |   const parts = s.split('\n');
// |   if (parts[parts.length - 1] === '') {
// |     parts.pop();
// |   }
// |   const last = s.endsWith('\n') ? parts.length : parts.length - 1;
// |   return parts.map((line, index) => (index < last && line.endsWith('\r') ? line.slice(0, -1) : line));
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `.chars().count()`.
// |  * @param {string} s
// |  */
// | export function charCount(s) {
// |   let count = 0;
// |   for (let index = 0; index < s.length; index += 1) {
// |     const code = s.charCodeAt(index);
// |     if (code < 0xdc00 || code > 0xdfff || index === 0 || !isHighSurrogate(s.charCodeAt(index - 1))) {
// |       count += 1;
// |     }
// |   }
// |   return count;
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=875bc14f0c5f195c621cfb011b5abb165ca8f09ebcb6be546e0ad7b0c16ec1f4
// | /** @param {number} code */
// | function isHighSurrogate(code) {
// |   return code >= 0xd800 && code <= 0xdbff;
// | }
pub fn is_high_surrogate(code: f64) -> bool {
    ((code >= (55296f64)) && (code <= (56319f64)))
}

// meta-language:translated JavaScript function_declaration items=1 sha256=f9b5312e87bb48fa8c389ea6d7c483adcfa19b939251303b9a8e798be7bc8a22
// | /**
// |  * The UTF-8 length of one code point.
// |  * @param {number} codePoint
// |  */
// | function utf8Width(codePoint) {
// |   if (codePoint < 0x80) {
// |     return 1;
// |   }
// |   if (codePoint < 0x800) {
// |     return 2;
// |   }
// |   return codePoint < 0x10000 ? 3 : 4;
// | }
pub fn utf8_width(code_point: f64) -> f64 {
    if (code_point < (128f64)) {
        1f64
    } else {
        if (code_point < (2048f64)) {
            2f64
        } else {
            if (code_point < (65536f64)) {
                3f64
            } else {
                4f64
            }
        }
    }
}

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `str::len()`: the UTF-8 byte length.
// |  * @param {string} s
// |  */
// | export function utf8Len(s) {
// |   return indexToByte(s, s.length);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The UTF-8 byte offset of a UTF-16 index of `s`.
// |  * @param {string} s
// |  * @param {number} index
// |  */
// | export function indexToByte(s, index) {
// |   let bytes = 0;
// |   for (let position = 0; position < index;) {
// |     const codePoint = /** @type {number} */ (s.codePointAt(position));
// |     bytes += utf8Width(codePoint);
// |     position += codePoint > 0xffff ? 2 : 1;
// |   }
// |   return bytes;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The UTF-16 index of a UTF-8 byte offset of `s`, or null when the offset is
// |  * out of bounds or inside a char.
// |  * @param {string} s
// |  * @param {number} byte
// |  * @returns {number | null}
// |  */
// | function boundaryIndex(s, byte) {
// |   let bytes = 0;
// |   let position = 0;
// |   while (bytes < byte && position < s.length) {
// |     const codePoint = /** @type {number} */ (s.codePointAt(position));
// |     bytes += utf8Width(codePoint);
// |     position += codePoint > 0xffff ? 2 : 1;
// |   }
// |   return bytes === byte ? position : null;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * The UTF-16 index of a UTF-8 byte offset of `s` that Rust has computed (a
// |  * `find` result, a `char_indices` offset). Panics when it is not a char
// |  * boundary; slice with `sliceBytes`, which panics as Rust does.
// |  * @param {string} s
// |  * @param {number} byte
// |  */
// | export function byteToIndex(s, byte) {
// |   const index = boundaryIndex(s, byte);
// |   return index ?? panic(`byte index ${byte} is not a char boundary of ${debugStr(s)}`);
// | }

// meta-language:carried JavaScript export_statement (syntax)
// | /**
// |  * `&s[begin..end]` (or `&s[begin..]` without `end`) with UTF-8 byte offsets.
// |  * Panics with Rust's messages, checked in Rust's order.
// |  * @param {string} s
// |  * @param {number} begin
// |  * @param {number} [end]
// |  */
// | export function sliceBytes(s, begin, end) {
// |   const len = utf8Len(s);
// |   const stop = end ?? len;
// |   if (begin > len) {
// |     return panic(`start byte index ${begin} is out of bounds for string of length ${len}`);
// |   }
// |   if (stop > len) {
// |     return panic(`end byte index ${stop} is out of bounds for string of length ${len}`);
// |   }
// |   if (begin > stop) {
// |     return panic(`byte range starts at ${begin} but ends at ${stop}`);
// |   }
// |   const from = boundaryIndex(s, begin);
// |   if (from === null) {
// |     return panic(`start byte index ${begin} ${insideChar(s, begin)}`);
// |   }
// |   const to = boundaryIndex(s, stop);
// |   if (to === null) {
// |     return panic(`end byte index ${stop} ${insideChar(s, stop)}`);
// |   }
// |   return s.slice(from, to);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * The end of a slicing panic for a byte offset inside a char.
// |  * @param {string} s
// |  * @param {number} byte
// |  */
// | function insideChar(s, byte) {
// |   let start = 0;
// |   for (const c of s) {
// |     const width = utf8Width(/** @type {number} */ (c.codePointAt(0)));
// |     if (byte < start + width) {
// |       return `is not a char boundary; it is inside ${debugChar(c)} (bytes ${start}..${start + width} of string)`;
// |     }
// |     start += width;
// |   }
// |   return unreachable(`byte ${byte} is not inside ${debugStr(s)}`);
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `str::char_indices()`: `[byteOffset, char]` pairs.
// |  * @param {string} s
// |  * @returns {Array<[number, string]>}
// |  */
// | export function charIndices(s) {
// |   /** @type {Array<[number, string]>} */
// |   const result = [];
// |   let bytes = 0;
// |   for (const c of s) {
// |     result.push([bytes, c]);
// |     bytes += utf8Width(/** @type {number} */ (c.codePointAt(0)));
// |   }
// |   return result;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `char::len_utf8`.
// |  * @param {string} c
// |  */
// | export function lenUtf8(c) {
// |   return utf8Width(/** @type {number} */ (c.codePointAt(0)));
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `char::escape_debug_ext` as `<str as Debug>` uses it.
// |  * @param {string} c
// |  * @param {boolean} inStr whether the char is inside a string literal
// |  */
// | function escapeDebug(c, inStr) {
// |   switch (c) {
// |     case '\0':
// |       return '\\0';
// |     case '\t':
// |       return '\\t';
// |     case '\r':
// |       return '\\r';
// |     case '\n':
// |       return '\\n';
// |     case '\\':
// |       return '\\\\';
// |     case '"':
// |       return inStr ? '\\"' : c;
// |     case "'":
// |       return inStr ? c : "\\'";
// |     default:
// |       break;
// |   }
// |   // Printable ASCII, and two grapheme extenders Rust prints as they are.
// |   if ((c >= ' ' && c <= '~') || c === '\uff9e' || c === '\uff9f') {
// |     return c;
// |   }
// |   if (ESCAPED_IN_DEBUG.test(c)) {
// |     return `\\u{${/** @type {number} */ (c.codePointAt(0)).toString(16)}}`;
// |   }
// |   return c;
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `{:?}` of a `&str`/`String`: `"text"` with Rust's escapes.
// |  * @param {string} s
// |  */
// | export function debugStr(s) {
// |   let result = '"';
// |   for (const c of s) {
// |     result += escapeDebug(c, true);
// |   }
// |   return `${result}"`;
// | }

// meta-language:carried JavaScript export_statement (type)
// | /**
// |  * `{:?}` of a `char`: `'c'` with Rust's escapes.
// |  * @param {string} c
// |  */
// | export function debugChar(c) {
// |   return `'${escapeDebug(c, false)}'`;
// | }
