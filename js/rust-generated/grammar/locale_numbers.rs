// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9be458c5b1df2a95bc24b7ac80674f8ebaed655c26c48d3bc6e80ab1a607f0e5 bytes=11292

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Deterministic numeric conventions shared by every lexer entry point.
// |  *
// |  * A single comma before three digits groups thousands; otherwise it is a
// |  * decimal comma. With both punctuation marks the rightmost one is decimal.
// |  * The documented fend outlier `1,1` means 11 in the primary convention.
// |  * Function-call commas remain argument separators.
// |  *
// |  * `input: &[char]` is an array of one-code-point strings (`[...text]`), so
// |  * `start`, `end` and the "at position N" of the errors are char indexes, as
// |  * in Rust.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=9d4d12629187d31e09abd32a6699425b3566f07f2bf2c5e630590e1b79971629
// | import { err } from '../rust/core.js';
use crate::rust::core_::err;
// meta-language:translated JavaScript import_statement items=1 sha256=4f608d4d30ce1787e18ec0f859e34b54048f346bf71695e5c54fa77d4ced5f7d
// | import { U128, U32 } from '../rust/num.js';
use crate::rust::num::{U128, U32};
// meta-language:translated JavaScript import_statement items=1 sha256=b57a3da121c7de1eb30b2eebac86054aaceec35dcda49484bd465534c99eaf27
// | import {
// |   contains,
// |   countMatches,
// |   fromDigit,
// |   isAsciiDigit,
// |   isDigit,
// |   replace,
// |   rfind,
// |   rsplitOnce,
// |   split,
// |   toDigit,
// |   utf8Len,
// | } from '../rust/str.js';
use crate::rust::str_::{contains, countMatches, fromDigit, isAsciiDigit, isDigit, replace, rfind, rsplitOnce, split, toDigit, utf8Len};

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `input.get(index).is_some_and(predicate)`.
// |  * @param {string[]} input
// |  * @param {number} index
// |  * @param {(ch: string) => boolean} predicate
// |  */
// | function getIsSomeAnd(input, index, predicate) {
// |   const ch = input[index];
// |   return ch !== undefined && predicate(ch);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * `chars.iter().take_while(predicate).count()`.
// |  * @param {string[]} chars
// |  * @param {(ch: string) => boolean} predicate
// |  */
// | function takeWhileCount(chars, predicate) {
// |   let count = 0;
// |   while (count < chars.length && predicate(chars[count])) {
// |     count += 1;
// |   }
// |   return count;
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `pub(super) fn scan`: the normalized number text that starts at `start`
// |  * and the char index after it.
// |  * @param {string[]} input
// |  * @param {number} start
// |  * @param {boolean} allowComma
// |  * @param {boolean} decimalComma
// |  * @returns {[string, number]}
// |  */
// | export function scan(input, start, allowComma, decimalComma) {
// |   const number = scanRadix(input, start);
// |   if (number !== null) {
// |     return number;
// |   }
// |   let end = start;
// |   let firstGroupDigits = 0;
// |   let hasGroupSpace = false;
// |   let canGroupSpace = true;
// |   while (end < input.length) {
// |     const ch = input[end];
// |     if (isAsciiDigit(ch) || ch === '_') {
// |       if (ch === '_') {
// |         firstGroupDigits = 0;
// |       }
// |       if (isAsciiDigit(ch) && !hasGroupSpace) {
// |         firstGroupDigits += 1;
// |       }
// |     } else if ((ch === '.' || ch === "'" || ch === '’' || (ch === ',' && allowComma) || isSpace(ch))
// |       && getIsSomeAnd(input, end + 1, isAsciiDigit)
// |     ) {
// |       // `'` groups exactly three digits; otherwise it is the foot mark
// |       // of `6'2"` (issue #235).
// |       if ((ch === "'" || ch === '’') && !isThreeDigitGroup(input, end + 1)) {
// |         break;
// |       }
// |       if (isSpace(ch)) {
// |         // Bound lookahead to four digits. Invalid groups leave a token
// |         // boundary, also preserving date parts such as `Jan 17 2027`.
// |         const digits = takeWhileCount(input.slice(end + 1).slice(0, 4), isAsciiDigit);
// |         if (!canGroupSpace || firstGroupDigits > 3 || digits !== 3) {
// |           break;
// |         }
// |         hasGroupSpace = true;
// |       } else {
// |         canGroupSpace = ch === '.';
// |         firstGroupDigits = 0;
// |         hasGroupSpace = false;
// |       }
// |     } else {
// |       break;
// |     }
// |     end += 1;
// |   }
// |   const candidate = input.slice(start, end).join('');
// |   const normalized = normalize(candidate, decimalComma)
// |     ?? err(CalculatorError.parse(`Invalid number separators in '${candidate}' at position ${start}`));
// |   return [normalized, end];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string[]} input
// |  * @param {number} start
// |  */
// | function isThreeDigitGroup(input, start) {
// |   return takeWhileCount(input.slice(start).slice(0, 4), isAsciiDigit) === 3;
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=27b8d699c210a57538f660be79b5c919b7d4758d6b13cf0c541d2aa89bf0f961
// | /** @param {string} ch */
// | function isSpace(ch) {
// |   return ch === ' ' || ch === '\u{00a0}' || ch === '\u{2007}' || ch === '\u{2009}' || ch === '\u{202f}';
// | }
pub fn is_space(ch: String) -> bool {
    (((((ch == " ") || (ch == " ")) || (ch == " ")) || (ch == " ")) || (ch == " "))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} candidate
// |  * @param {boolean} decimalComma
// |  * @returns {string | null}
// |  */
// | function normalize(candidate, decimalComma) {
// |   // Underscores may separate digits, including fractional digits, but cannot
// |   // hide malformed punctuation or occur consecutively.
// |   const chars = [...candidate];
// |   for (const [i, ch] of chars.entries()) {
// |     if (ch === '_'
// |       && (i === 0
// |         || !isAsciiDigit(chars[i - 1])
// |         || !getIsSomeAnd(chars, i + 1, isAsciiDigit))
// |     ) {
// |       return null;
// |     }
// |   }
// |   // Space grouping may also continue an underscore-grouped integer.
// |   const number = [...candidate].some(isSpace) && !contains(candidate, ['.', ','])
// |     ? replace(candidate, '_', ' ')
// |     : replace(candidate, '_', '');
// |   // Issue #233 explicitly requires this fend compatibility outlier alongside
// |   // decimal commas. Keep it narrow; `1,2` and `12,3` remain decimal fractions.
// |   if (number === '1,1' && !decimalComma) {
// |     return '11';
// |   }
// |   const commas = countMatches(number, ',');
// |   const dots = countMatches(number, '.');
// |   /** @type {string | null} */
// |   let decimal;
// |   if (commas === 0 && dots === 0) {
// |     decimal = null;
// |   } else if (commas === 0 && dots === 1) {
// |     decimal = '.';
// |   } else if (commas === 1 && dots === 0 && (decimalComma || !validGroups(number, ','))) {
// |     decimal = ',';
// |   } else if (dots === 0) {
// |     decimal = null;
// |   } else if (commas === 0) {
// |     return null;
// |   } else if (rfind(number, ',') > rfind(number, '.')) {
// |     // Both are `Some` here: there is at least one comma and one dot.
// |     decimal = ',';
// |   } else {
// |     decimal = '.';
// |   }
// |   let integer;
// |   /** @type {string | null} */
// |   let fraction;
// |   if (decimal !== null) {
// |     const parts = rsplitOnce(number, decimal);
// |     if (parts === null) {
// |       return null;
// |     }
// |     const [integerPart, fractionPart] = parts;
// |     if (contains(integerPart, decimal)
// |       || fractionPart === ''
// |       || !([...fractionPart].every(isAsciiDigit)
// |         || ([...fractionPart].some(isSpace) && validGroups(fractionPart, ' ')))
// |     ) {
// |       return null;
// |     }
// |     integer = integerPart;
// |     fraction = fractionPart;
// |   } else {
// |     integer = number;
// |     fraction = null;
// |   }
// |   const grouping = [...integer].filter((ch) => !isAsciiDigit(ch));
// |   const separator = grouping[0] ?? null;
// |   if (separator !== null) {
// |     if (grouping.some((ch) => !sameGroup(ch, separator))
// |       || !validGroups(integer, separator)
// |     ) {
// |       return null;
// |     }
// |     integer = [...integer].filter(isAsciiDigit).join('');
// |   } else if (integer === '') {
// |     // Preserve the conventional leading-dot decimal spelling `.5`.
// |     if (decimal !== '.') {
// |       return null;
// |     }
// |     integer = '0';
// |   }
// |   return fraction === null
// |     ? integer
// |     : `${integer}.${[...fraction].filter(isAsciiDigit).join('')}`;
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=5db11ab5d436f6694353885a1a3bbc173ed865bf2ef009c6a134167e87e5e87f
// | /**
// |  * @param {string} left
// |  * @param {string} right
// |  */
// | function sameGroup(left, right) {
// |   return left === right
// |     || (isSpace(left) && isSpace(right))
// |     || ((left === "'" || left === '’') && (right === "'" || right === '’'));
// | }
pub fn same_group(left: String, right: String) -> bool {
    (((left == right) || (is_space(left.clone()) && is_space(right.clone()))) || (((left == "'") || (left == "’")) && ((right == "'") || (right == "’"))))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} number
// |  * @param {string} separator
// |  */
// | function validGroups(number, separator) {
// |   const groups = split(number, (ch) => sameGroup(ch, separator));
// |   const first = groups[0] ?? null;
// |   if (first === null) {
// |     return false;
// |   }
// |   if (first === ''
// |     || utf8Len(first) > 3
// |     || groups.some((part) => ![...part].every(isAsciiDigit))
// |   ) {
// |     return false;
// |   }
// |   const western = groups.slice(1).every((part) => utf8Len(part) === 3);
// |   const indian = separator === ','
// |     && groups.length >= 3
// |     && utf8Len(first) <= 2
// |     && utf8Len(groups[groups.length - 1]) === 3
// |     && groups.slice(1, groups.length - 1).every((part) => utf8Len(part) === 2);
// |   return western || indian;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Parse radix fractions exactly within the decimal grammar's 28-place limit.
// |  * Accumulation and expansion are bounded, and overflow is recoverable.
// |  *
// |  * The `u128` numerator and denominator are bigints.
// |  * @param {string[]} input
// |  * @param {number} start
// |  * @returns {[string, number] | null}
// |  */
// | function scanRadix(input, start) {
// |   if (input[start] !== '0') {
// |     return null;
// |   }
// |   let radix;
// |   switch (input[start + 1]) {
// |     case 'b':
// |     case 'B':
// |       radix = 2;
// |       break;
// |     case 'o':
// |     case 'O':
// |       radix = 8;
// |       break;
// |     case 'x':
// |     case 'X':
// |       radix = 16;
// |       break;
// |     default:
// |       return null;
// |   }
// |   if (!getIsSomeAnd(input, start + 2, (ch) => isDigit(ch, radix))) {
// |     return null;
// |   }
// |   let end = start + 2;
// |   let numerator = 0n;
// |   let denominator = 1n;
// |   let fraction = false;
// |   while (end < input.length) {
// |     const ch = input[end];
// |     const digit = toDigit(ch, radix);
// |     if (digit !== null) {
// |       const shifted = U128.checkedMul(numerator, BigInt(radix));
// |       numerator = (shifted === null ? null : U128.checkedAdd(shifted, BigInt(digit)))
// |         ?? err(CalculatorError.Overflow);
// |       if (fraction) {
// |         denominator = U128.checkedMul(denominator, BigInt(radix))
// |           ?? err(CalculatorError.Overflow);
// |       }
// |     } else if (ch === '_'
// |       && isDigit(input[end - 1], radix)
// |       && getIsSomeAnd(input, end + 1, (next) => isDigit(next, radix))
// |     ) {
// |       // Digit separator, preserving the existing underscore convention.
// |     } else if (ch === '.' && !fraction) {
// |       fraction = true;
// |     } else {
// |       break;
// |     }
// |     end += 1;
// |   }
// |   if (input[end] === 'e' || input[end] === 'E') {
// |     const [negative, exponent, exponentEnd] = scanRadixExponent(input, end + 1, radix);
// |     end = exponentEnd;
// |     const factor = U128.checkedPow(BigInt(radix), exponent)
// |       ?? err(CalculatorError.Overflow);
// |     if (negative) {
// |       denominator = U128.checkedMul(denominator, factor)
// |         ?? err(CalculatorError.Overflow);
// |     } else {
// |       numerator = U128.checkedMul(numerator, factor)
// |         ?? err(CalculatorError.Overflow);
// |     }
// |   }
// |   let number = String(numerator / denominator);
// |   let remainder = numerator % denominator;
// |   if (remainder !== 0n) {
// |     number += '.';
// |     for (let place = 0; place < 28; place += 1) {
// |       remainder = U128.checkedMul(remainder, 10n) ?? err(CalculatorError.Overflow);
// |       const digit = U32.tryFrom(remainder / denominator) ?? err(CalculatorError.Overflow);
// |       number += fromDigit(Number(digit), 10) ?? err(CalculatorError.Overflow);
// |       remainder %= denominator;
// |       if (remainder === 0n) {
// |         break;
// |       }
// |     }
// |     if (remainder !== 0n) {
// |       return err(CalculatorError.Overflow);
// |     }
// |   }
// |   return [number, end];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * fend uses exponent digits and powers in the literal's radix, e.g. `0b1e10 = 4`.
// |  * @param {string[]} input
// |  * @param {number} end
// |  * @param {number} radix
// |  * @returns {[boolean, number, number]} whether the exponent is negative, the
// |  *   `u32` exponent, and the char index after it
// |  */
// | function scanRadixExponent(input, end, radix) {
// |   const negative = input[end] === '-';
// |   if (input[end] === '+' || input[end] === '-') {
// |     end += 1;
// |   }
// |   if (!getIsSomeAnd(input, end, (ch) => isDigit(ch, radix))) {
// |     return err(CalculatorError.parse('Invalid radix exponent'));
// |   }
// |   let exponent = 0;
// |   while (end < input.length) {
// |     const ch = input[end];
// |     const digit = toDigit(ch, radix);
// |     if (digit !== null) {
// |       const shifted = U32.checkedMul(BigInt(exponent), BigInt(radix));
// |       exponent = Number((shifted === null ? null : U32.checkedAdd(shifted, BigInt(digit)))
// |         ?? err(CalculatorError.Overflow));
// |     } else if (ch !== '_'
// |       || !isDigit(input[end - 1], radix)
// |       || !getIsSomeAnd(input, end + 1, (next) => isDigit(next, radix))
// |     ) {
// |       break;
// |     }
// |     end += 1;
// |   }
// |   return [negative, exponent, end];
// | }
