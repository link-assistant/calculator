// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fce3ae010db273743c5a81950eb3cdf66f631bd35d2a5b5ebbf57d5965f70e4f bytes=17526

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// Unbounded integers for the portable core's naturals and integers.
pub mod ml {
    use std::cmp::Ordering;
    use std::fmt;

    /// Sign and magnitude; the magnitude is little-endian base 2^32 without
    /// leading zero limbs, and zero is never negative.
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Big {
        negative: bool,
        magnitude: Vec<u32>,
    }

    fn trim(mut magnitude: Vec<u32>) -> Vec<u32> {
        while magnitude.last() == Some(&0) {
            magnitude.pop();
        }
        magnitude
    }

    fn compare_magnitude(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    fn add_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for index in 0..a.len().max(b.len()) {
            let sum = u64::from(*a.get(index).unwrap_or(&0)) + u64::from(*b.get(index).unwrap_or(&0)) + carry;
            out.push(sum as u32);
            carry = sum >> 32;
        }
        if carry > 0 {
            out.push(carry as u32);
        }
        out
    }

    /// |a| - |b| where |a| >= |b|.
    fn sub_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (index, limb) in a.iter().enumerate() {
            let mut difference = i64::from(*limb) - i64::from(*b.get(index).unwrap_or(&0)) - borrow;
            borrow = 0;
            if difference < 0 {
                difference += 1 << 32;
                borrow = 1;
            }
            out.push(difference as u32);
        }
        trim(out)
    }

    fn mul_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, x) in a.iter().enumerate() {
            let mut carry = 0u64;
            for (j, y) in b.iter().enumerate() {
                let product = u64::from(out[i + j]) + u64::from(*x) * u64::from(*y) + carry;
                out[i + j] = product as u32;
                carry = product >> 32;
            }
            let mut index = i + b.len();
            while carry > 0 {
                let sum = u64::from(out[index]) + carry;
                out[index] = sum as u32;
                carry = sum >> 32;
                index += 1;
            }
        }
        trim(out)
    }

    /// Quotient and remainder of magnitudes; the divisor is not zero.
    fn divmod_magnitude(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        let mut quotient = vec![0u32; a.len()];
        if b.len() == 1 {
            let divisor = u64::from(b[0]);
            let mut remainder = 0u64;
            for index in (0..a.len()).rev() {
                let current = (remainder << 32) | u64::from(a[index]);
                quotient[index] = (current / divisor) as u32;
                remainder = current % divisor;
            }
            return (trim(quotient), trim(vec![remainder as u32]));
        }
        let mut remainder: Vec<u32> = Vec::new();
        for bit in (0..a.len() * 32).rev() {
            let mut carry = (a[bit / 32] >> (bit % 32)) & 1;
            for limb in remainder.iter_mut() {
                let next = *limb >> 31;
                *limb = (*limb << 1) | carry;
                carry = next;
            }
            if carry != 0 {
                remainder.push(carry);
            }
            if compare_magnitude(&remainder, b) != Ordering::Less {
                remainder = sub_magnitude(&remainder, b);
                quotient[bit / 32] |= 1 << (bit % 32);
            }
        }
        (trim(quotient), remainder)
    }

    impl Big {
        fn from_parts(negative: bool, magnitude: Vec<u32>) -> Self {
            let magnitude = trim(magnitude);
            Self { negative: negative && !magnitude.is_empty(), magnitude }
        }

        pub fn zero() -> Self {
            Self::from_parts(false, Vec::new())
        }

        pub fn from_i128(value: i128) -> Self {
            let mut big = Self::from_u128(value.unsigned_abs());
            big.negative = value < 0;
            big
        }

        pub fn from_u128(mut value: u128) -> Self {
            let mut magnitude = Vec::new();
            while value > 0 {
                magnitude.push(value as u32);
                value >>= 32;
            }
            Self::from_parts(false, magnitude)
        }

        /// A decimal literal with an optional leading minus sign.
        pub fn parse(text: &str) -> Self {
            let (negative, digits) = match text.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, text),
            };
            let ten = Self::from_i128(10);
            let mut value = Self::zero();
            for digit in digits.chars() {
                let digit = digit.to_digit(10).expect("decimal literal");
                value = value.mul(&ten).add(&Self::from_i128(i128::from(digit)));
            }
            if negative { value.neg() } else { value }
        }

        pub fn is_zero(&self) -> bool {
            self.magnitude.is_empty()
        }

        pub fn is_negative(&self) -> bool {
            self.negative
        }

        /// The array index this names, if it is one.
        pub fn to_index(&self) -> Option<usize> {
            if self.negative || self.magnitude.len() > 2 {
                return None;
            }
            let value = self.magnitude.iter().rev().fold(0u64, |acc, &digit| (acc << 32) | u64::from(digit));
            usize::try_from(value).ok()
        }

        pub fn neg(&self) -> Self {
            Self::from_parts(!self.negative, self.magnitude.clone())
        }

        pub fn add(&self, other: &Self) -> Self {
            if self.negative == other.negative {
                return Self::from_parts(self.negative, add_magnitude(&self.magnitude, &other.magnitude));
            }
            match compare_magnitude(&self.magnitude, &other.magnitude) {
                Ordering::Less => Self::from_parts(other.negative, sub_magnitude(&other.magnitude, &self.magnitude)),
                _ => Self::from_parts(self.negative, sub_magnitude(&self.magnitude, &other.magnitude)),
            }
        }

        pub fn sub(&self, other: &Self) -> Self {
            self.add(&other.neg())
        }

        pub fn mul(&self, other: &Self) -> Self {
            Self::from_parts(self.negative != other.negative, mul_magnitude(&self.magnitude, &other.magnitude))
        }

        /// Division rounding toward zero; the divisor is not zero.
        pub fn divmod_trunc(&self, other: &Self) -> (Self, Self) {
            let (quotient, remainder) = divmod_magnitude(&self.magnitude, &other.magnitude);
            (
                Self::from_parts(self.negative != other.negative, quotient),
                Self::from_parts(self.negative, remainder),
            )
        }
    }

    impl Ord for Big {
        fn cmp(&self, other: &Self) -> Ordering {
            match (self.negative, other.negative) {
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (false, false) => compare_magnitude(&self.magnitude, &other.magnitude),
                (true, true) => compare_magnitude(&other.magnitude, &self.magnitude),
            }
        }
    }

    impl PartialOrd for Big {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl fmt::Display for Big {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.is_zero() {
                return formatter.write_str("0");
            }
            let billion = [1_000_000_000u32];
            let mut chunks = Vec::new();
            let mut rest = self.magnitude.clone();
            while !rest.is_empty() {
                let (quotient, remainder) = divmod_magnitude(&rest, &billion);
                chunks.push(remainder.first().copied().unwrap_or(0));
                rest = quotient;
            }
            let mut text = String::new();
            if self.negative {
                text.push('-');
            }
            text.push_str(&chunks.pop().unwrap_or(0).to_string());
            for chunk in chunks.iter().rev() {
                text.push_str(&format!("{chunk:09}"));
            }
            formatter.write_str(&text)
        }
    }

    /// Natural subtraction: truncated at zero.
    pub fn nat_sub(a: &Big, b: &Big) -> Big {
        if a > b { a.sub(b) } else { Big::zero() }
    }

    pub fn pred(a: &Big) -> Big {
        a.sub(&Big::from_i128(1))
    }

    /// Integer division with the source's rounding ("trunc", "floor" or
    /// "euclid"); by zero it aborts with the source's message or is total
    /// (x / 0 = 0, x % 0 = x).
    pub fn divide(a: &Big, b: &Big, rounding: &str, by_zero: Option<&str>, remainder: bool) -> Big {
        if b.is_zero() {
            if let Some(message) = by_zero {
                panic!("{message}");
            }
            return if remainder { a.clone() } else { Big::zero() };
        }
        let one = Big::from_i128(1);
        let (mut quotient, rest) = a.divmod_trunc(b);
        if !rest.is_zero() {
            if rounding == "floor" && rest.is_negative() != b.is_negative() {
                quotient = quotient.sub(&one);
            }
            if rounding == "euclid" && rest.is_negative() {
                quotient = if b.is_negative() { quotient.add(&one) } else { quotient.sub(&one) };
            }
        }
        if remainder { a.sub(&quotient.mul(b)) } else { quotient }
    }

    pub fn to_nat_checked(value: Big, message: &str) -> Big {
        if value.is_negative() {
            panic!("{message}");
        }
        value
    }

    pub fn clamp_nat(value: Big) -> Big {
        if value.is_negative() { Big::zero() } else { value }
    }

    /// Bounded domains for executable theorem checks.
    pub fn range(low: i128, high: i128) -> Vec<Big> {
        (low..=high).map(Big::from_i128).collect()
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Spelled numbers and arithmetic operator words as an optional front end
// |  * (issue #222).
// |  *
// |  * The vocabulary lives in `data/words/arithmetic-words.lino` (one link per
// |  * surface form), so adding a language means editing data, not code. The lexer
// |  * calls [`scan`] at the start of every word: when the input there is a number
// |  * word ("twenty one", "сто двадцать", "二十三") or an operator phrase
// |  * ("multiplied by", "умножить на", "除以"), it receives ready-made `Number` or
// |  * operator tokens; otherwise it lexes an ordinary identifier.
// |  *
// |  * `input: &[char]` is an array of one-code-point strings (`[...text]`), so
// |  * every position is a char index, as in Rust.
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=eb4026bed2dcb2d46ec1d67a7f404022282f54a03353a5738d7fa2e8637b3fec
// | import { isWordChar, TokenKind } from './lexer.js';
use crate::grammar::lexer::{isWordChar, TokenKind};
// meta-language:translated JavaScript import_statement items=1 sha256=77223ff4065cabc97af188a10d83780709b13c3461405d42d32dbc3d21163a7b
// | import { CurrencyDatabase } from '../types/currency.js';
use crate::types::currency::CurrencyDatabase;
// meta-language:translated JavaScript import_statement items=1 sha256=0a1f60462c594ae56f2e14e8ec74e791ef163b0f7eb7f8e4671a456ae4a11901
// | import { ARITHMETIC_WORDS_LINO } from '../generated/arithmetic-words-lino.js';
use crate::generated::arithmetic_words_lino::ARITHMETIC_WORDS_LINO;
// meta-language:translated JavaScript import_statement items=1 sha256=5780b5fddec468273672cd8196e15bd2a156e5f695e945ce155a10b5601c0446
// | import { eq, ok, panic } from '../rust/core.js';
use crate::rust::core_::{eq, ok, panic};
// meta-language:translated JavaScript import_statement items=1 sha256=d890f767af793f85fe5abb1fa7820d1b5d74d836d6204059e074c09fc2f0039a
// | import { U128 } from '../rust/num.js';
use crate::rust::num::U128;
// meta-language:translated JavaScript import_statement items=1 sha256=a2332c91d411fc0924780fe01fe69f3322ea379c6f6355e32a60c565fa1cde56
// | import {
// |   charCount,
// |   debugStr,
// |   isAsciiAlphabetic,
// |   isUppercase,
// |   isWhitespace,
// |   lines,
// |   splitOnce,
// |   splitWhitespace,
// |   startsWith,
// |   stripPrefix,
// |   stripSuffix,
// |   trim,
// | } from '../rust/str.js';
use crate::rust::str_::{charCount, debugStr, isAsciiAlphabetic, isUppercase, isWhitespace, lines, splitOnce, splitWhitespace, startsWith, stripPrefix, stripSuffix, trim};

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** What a single vocabulary word means. */
// | class Meaning {
// |   /** @param {string} tag @param {object} [fields] */
// |   constructor(tag, fields) {
// |     /** @type {string} */
// |     this.tag = tag;
// |     Object.assign(this, fields);
// |     Object.freeze(this);
// |   }
// |
// |   /** A cardinal number ("seven", "сорок"). @param {bigint} n */
// |   static Number(n) { return new Meaning('Number', { _0: n }); }
// |
// |   /** A multiplier of what precedes it ("hundred", "тысяча", "十"). @param {bigint} s */
// |   static Scale(s) { return new Meaning('Scale', { _0: s }); }
// |
// |   /** An arithmetic operator. @param {TokenKind} op */
// |   static Operator(op) { return new Meaning('Operator', { _0: op }); }
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * @typedef {{ lang: string, meaning: Meaning & { _0: any } }} Word
// |  */

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * The parsed vocabulary.
// |  * @typedef {object} Vocabulary
// |  * @property {Map<string, Word>} words Single words (lowercase) to their meaning.
// |  * @property {Array<[string[], TokenKind]>} phrases Multi-word operator phrases (lowercase words), longest first.
// |  * @property {number} longestWord Length in characters of the longest single word, for CJK segmentation.
// |  */

// meta-language:carried JavaScript lexical_declaration (unsupported)
// | /** @type {Vocabulary | null} */
// | let VOCABULARY = null;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * @internal
// |  * @returns {Vocabulary}
// |  */
// | export function vocabulary() {
// |   if (VOCABULARY === null) {
// |     const result = parseVocabulary(ARITHMETIC_WORDS_LINO);
// |     if (typeof result === 'string') {
// |       panic(`data/words/arithmetic-words.lino is valid: ${debugStr(result)}`);
// |     }
// |     VOCABULARY = result;
// |   }
// |   return VOCABULARY;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string} symbol
// |  * @returns {TokenKind | null}
// |  */
// | function operatorKind(symbol) {
// |   switch (symbol) {
// |     case '+': return TokenKind.Plus;
// |     case '-': return TokenKind.Minus;
// |     case '*': return TokenKind.Star;
// |     case '/': return TokenKind.Slash;
// |     case '%': return TokenKind.Percent;
// |     case '^': return TokenKind.Caret;
// |     case 'percent': return TokenKind.PercentWord;
// |     case 'percent-prefix': return TokenKind.PercentPrefix;
// |     case 'portion': return TokenKind.Portion;
// |     case 'increase-from': return TokenKind.IncreaseFrom;
// |     case 'decrease-from': return TokenKind.DecreaseFrom;
// |     default: return null;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Parses lines of the form `(kind lang value 'surface form')`.
// |  *
// |  * `Result<Vocabulary, String>`: returns the vocabulary, or the error message
// |  * as a string.
// |  * @internal
// |  * @param {string} content
// |  * @returns {Vocabulary | string}
// |  */
// | export function parseVocabulary(content) {
// |   /** @type {Vocabulary} */
// |   const vocabulary = { words: new Map(), phrases: [], longestWord: 0 };
// |   const allLines = lines(content);
// |   for (let index = 0; index < allLines.length; index++) {
// |     const line = trim(allLines[index]);
// |     if (line === '' || startsWith(line, '//')) {
// |       continue;
// |     }
// |     const error = () => `arithmetic-words.lino:${index + 1}: malformed link ${debugStr(line)}`;
// |     const rest = stripPrefix(line, '(');
// |     const body = rest === null ? null : stripSuffix(rest, ')');
// |     if (body === null) {
// |       return error();
// |     }
// |     const split = splitOnce(body, '\'');
// |     if (split === null) {
// |       return error();
// |     }
// |     const [head, rawSurface] = split;
// |     const stripped = stripSuffix(rawSurface, '\'');
// |     if (stripped === null) {
// |       return error();
// |     }
// |     const surface = stripped.toLowerCase();
// |     const fields = splitWhitespace(head);
// |     if (fields.length !== 3) {
// |       return error();
// |     }
// |     const [kind, lang, value] = fields;
// |     let meaning;
// |     switch (kind) {
// |       case 'number': {
// |         const n = ok(() => U128.parse(value));
// |         if (n === null) {
// |           return error();
// |         }
// |         meaning = Meaning.Number(n);
// |         break;
// |       }
// |       case 'scale': {
// |         const n = ok(() => U128.parse(value));
// |         if (n === null) {
// |           return error();
// |         }
// |         meaning = Meaning.Scale(n);
// |         break;
// |       }
// |       case 'operator': {
// |         const op = operatorKind(value);
// |         if (op === null) {
// |           return error();
// |         }
// |         meaning = Meaning.Operator(op);
// |         break;
// |       }
// |       default:
// |         return error();
// |     }
// |
// |     const words = splitWhitespace(surface);
// |     if (words.length === 0) {
// |       return error();
// |     } else if (words.length === 1) {
// |       vocabulary.longestWord = Math.max(vocabulary.longestWord, charCount(surface));
// |       const word = { lang, meaning: /** @type {Meaning & { _0: any }} */ (meaning) };
// |       const previous = vocabulary.words.get(surface);
// |       vocabulary.words.set(surface, word);
// |       if (previous !== undefined) {
// |         if (!eq(previous.meaning, word.meaning)) {
// |           return `${error()}: ${debugStr(surface)} has two meanings`;
// |         }
// |       }
// |     } else if (meaning.tag === 'Operator') {
// |       vocabulary.phrases.push([words, /** @type {any} */ (meaning)._0]);
// |     } else {
// |       return `${error()}: only operators may span words`;
// |     }
// |   }
// |   vocabulary.phrases.sort((a, b) => b[0].length - a[0].length);
// |   return vocabulary;
// | }

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * A token produced from words: kind, start, end (char positions) and text.
// |  * @typedef {[TokenKind, number, number, string]} WordToken
// |  */

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Recognizes number words or an operator phrase starting at `pos`.
// |  *
// |  * Returns the tokens (at least one) in input order, or `None` when the word
// |  * at `pos` is not arithmetic vocabulary.
// |  * @param {string[]} input
// |  * @param {number} pos
// |  * @returns {WordToken[] | null}
// |  */
// | export function scan(input, pos) {
// |   const longest = Math.min(vocabulary().longestWord, input.length - pos);
// |   for (let len = longest; len >= 1; len--) {
// |     const word = lookup(input, pos, pos + len);
// |     if (word !== null && word.meaning.tag === 'Operator') {
// |       /** @type {TokenKind} */
// |       const op = word.meaning._0;
// |       if ((op.tag === 'Portion' || op.tag === 'PercentPrefix')
// |         && (op.tag !== 'Portion' || portionHasPercent(input, pos + len))
// |         && (pos + len === input.length || !isAsciiAlphabetic(input[pos + len]))) {
// |         return [[op, pos, pos + len, text(input, pos, pos + len)]];
// |       }
// |     }
// |   }
// |   if (isCjk(input[pos])) {
// |     return scanCjk(input, pos);
// |   }
// |   const token = scanPhrase(input, pos);
// |   if (token !== null) {
// |     return [token];
// |   }
// |   const number = scanSpacedNumber(input, pos);
// |   return number === null ? null : [number];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | // A possession word also occurs in dates, units, and powers. Recognize the
// | // whole-before-part form only when the following text contains percent vocabulary.
// | /**
// |  * @param {string[]} input
// |  * @param {number} start
// |  * @returns {boolean}
// |  */
// | function portionHasPercent(input, start) {
// |   for (let pos = start; pos < input.length; pos++) {
// |     if (input[pos] === '%') {
// |       return true;
// |     }
// |     const longest = Math.min(vocabulary().longestWord, input.length - pos);
// |     for (let len = longest; len >= 1; len--) {
// |       const word = lookup(input, pos, pos + len);
// |       if (word !== null && word.meaning.tag === 'Operator'
// |         && (word.meaning._0.tag === 'PercentPrefix' || word.meaning._0.tag === 'PercentWord')) {
// |         return true;
// |       }
// |     }
// |     if (['+', '-', '*', '/', '(', ')'].includes(input[pos])) {
// |       break;
// |     }
// |   }
// |   return false;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {string} ch */
// | function isCjk(ch) {
// |   const cp = /** @type {number} */ (ch.codePointAt(0));
// |   return (cp >= 0x3400 && cp <= 0x4DBF) || (cp >= 0x4E00 && cp <= 0x9FFF) || (cp >= 0xF900 && cp <= 0xFAFF);
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * End of the word that starts at `pos`.
// |  * @param {string[]} input
// |  * @param {number} pos
// |  */
// | function wordEnd(input, pos) {
// |   let end = pos;
// |   while (end < input.length && isWordChar(input[end])) {
// |     end += 1;
// |   }
// |   return end;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {string[]} input
// |  * @param {number} start
// |  * @param {number} end
// |  */
// | function text(input, start, end) {
// |   return input.slice(start, end).join('');
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * An all-uppercase word that names a known currency ("DOS", "MIL", "ONE")
// |  * keeps its currency meaning, like "SEK" does over German "sek" seconds.
// |  * @param {string} word
// |  */
// | function isCurrencyCode(word) {
// |   return charCount(word) >= 2
// |     && [...word].every(isUppercase)
// |     && CurrencyDatabase.parseCurrency(word) !== null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Looks up the word spanning `start..end`, skipping currency codes.
// |  * @param {string[]} input
// |  * @param {number} start
// |  * @param {number} end
// |  * @returns {Word | null}
// |  */
// | function lookup(input, start, end) {
// |   const word = text(input, start, end);
// |   if (start === end || isCurrencyCode(word)) {
// |     return null;
// |   }
// |   return vocabulary().words.get(word.toLowerCase()) ?? null;
// | }

// meta-language:carried JavaScript function_declaration (type)
// | /**
// |  * @param {string[]} input
// |  * @param {number} pos
// |  */
// | function skipWhitespace(input, pos) {
// |   while (pos < input.length && isWhitespace(input[pos])) {
// |     pos += 1;
// |   }
// |   return pos;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Matches the longest operator phrase (or single operator word) at `pos`.
// |  * @param {string[]} input
// |  * @param {number} pos
// |  * @returns {WordToken | null}
// |  */
// | function scanPhrase(input, pos) {
// |   const firstEnd = wordEnd(input, pos);
// |   const firstText = text(input, pos, firstEnd);
// |   if (isCurrencyCode(firstText)) {
// |     return null;
// |   }
// |   const first = firstText.toLowerCase();
// |
// |   /** @type {[TokenKind, number] | null} */
// |   let matched = null;
// |   for (const [words, op] of vocabulary().phrases) {
// |     if (words[0] !== first) {
// |       continue;
// |     }
// |     let end = firstEnd;
// |     let found = true;
// |     for (const word of words.slice(1)) {
// |       const start = skipWhitespace(input, end);
// |       const nextEnd = wordEnd(input, start);
// |       if (start === end || text(input, start, nextEnd).toLowerCase() !== word) {
// |         found = false;
// |         break;
// |       }
// |       end = nextEnd;
// |     }
// |     if (found) {
// |       matched = [op, end];
// |       break;
// |     }
// |   }
// |   if (matched === null) {
// |     const word = vocabulary().words.get(first);
// |     if (word !== undefined && word.meaning.tag === 'Operator') {
// |       matched = [word.meaning._0, firstEnd];
// |     }
// |   }
// |
// |   if (matched === null) {
// |     return null;
// |   }
// |   const [op, end] = matched;
// |   // `mod(17, 5)` is the function, not the operator.
// |   if ((op.tag === 'Percent' || op.tag === 'PercentWord') && input[end] === '(') {
// |     return null;
// |   }
// |   return [op, pos, end, text(input, pos, end)];
// | }

// meta-language:carried JavaScript class_declaration (unsupported)
// | /** Combines consecutive number words into one value. */
// | class Composer {
// |   constructor() {
// |     /** @type {bigint} */
// |     this.total = 0n;
// |     /** @type {bigint} */
// |     this.current = 0n;
// |     /**
// |      * The last number word added since the last scale word.
// |      * @type {bigint | null}
// |      */
// |     this.last = null;
// |     /** @type {boolean} */
// |     this.any = false;
// |     /** @type {string | null} */
// |     this.lang = null;
// |   }
// |
// |   /** @returns {Composer} */
// |   static default() {
// |     return new Composer();
// |   }
// |
// |   /**
// |    * Adds a word if it continues the number; `false` ends the number.
// |    * @param {Word} word
// |    * @returns {boolean}
// |    */
// |   accept(word) {
// |     if (this.lang !== null && this.lang !== word.lang) {
// |       return false;
// |     }
// |     let accepted;
// |     switch (word.meaning.tag) {
// |       case 'Number':
// |         accepted = this.add(word.meaning._0);
// |         break;
// |       case 'Scale':
// |         accepted = this.scale(word.meaning._0);
// |         break;
// |       default:
// |         accepted = false;
// |     }
// |     if (accepted) {
// |       this.any = true;
// |       this.lang = word.lang;
// |     }
// |     return accepted;
// |   }
// |
// |   /** @param {bigint} n */
// |   add(n) {
// |     // "twenty one", "сто двадцать" and "two hundred five" continue a
// |     // number; "two three" or "ten five" do not.
// |     const continues = this.last === null ? true : this.last >= 20n && n < placeValue(this.last);
// |     if (!continues || (this.any && n === 0n)) {
// |       return false;
// |     }
// |     const current = U128.checkedAdd(this.current, n);
// |     if (current === null) {
// |       return false;
// |     }
// |     this.current = current;
// |     this.last = n;
// |     return true;
// |   }
// |
// |   /** @param {bigint} s */
// |   scale(s) {
// |     if (s >= 1000n) {
// |       // "thousand", "million", "万": close the group below it.
// |       if (this.any && this.current === 0n) {
// |         return false;
// |       }
// |       const group = U128.checkedMul(this.current > 1n ? this.current : 1n, s);
// |       const total = group === null ? null : U128.checkedAdd(this.total, group);
// |       if (total === null) {
// |         return false;
// |       }
// |       this.total = total;
// |       this.current = 0n;
// |     } else {
// |       // "hundred", "十", "百" multiply only the preceding digit, so that
// |       // 三百二十 is 3·100 + 2·10.
// |       if (this.any && this.last === null) {
// |         return false;
// |       }
// |       const digit = this.last ?? 0n;
// |       const scaled = U128.checkedMul(digit > 1n ? digit : 1n, s);
// |       const current = scaled === null ? null : U128.checkedAdd(this.current - digit, scaled);
// |       if (current === null) {
// |         return false;
// |       }
// |       this.current = current;
// |     }
// |     this.last = null;
// |     return true;
// |   }
// |
// |   value() {
// |     return this.total + this.current;
// |   }
// | }

// meta-language:translated JavaScript function_declaration items=2 sha256=6638127d0cf6b04031ff829751668c58977759d69e7798b02925a1473e6938e1
// | /**
// |  * The place value of the lowest non-zero digit (20 → 10, 100 → 100).
// |  * @param {bigint} n
// |  */
// | function placeValue(n) {
// |   let place = 1n;
// |   while (n > 0n && n % 10n === 0n) {
// |     n /= 10n;
// |     place *= 10n;
// |   }
// |   return place;
// | }
pub fn place_value(n: crate::ml::Big) -> crate::ml::Big {
    {
        let place = crate::ml::Big::from_i128(1);
        {
            let place_2 = crate::ml_place_value_loop1(n.clone(), place.clone());
            place_2.clone()
        }
    }
}

pub fn ml_place_value_loop1(mut n: crate::ml::Big, mut place: crate::ml::Big) -> crate::ml::Big {
    loop {
        return if ((n > (crate::ml::Big::from_i128(0))) && ((crate::ml::divide(&n, &crate::ml::Big::from_i128(10), "trunc", Some("Division by zero"), true)) == (crate::ml::Big::from_i128(0)))) {
            {
                let n_2 = crate::ml::divide(&n, &crate::ml::Big::from_i128(10), "trunc", Some("Division by zero"), false);
                {
                    let place_2 = place.mul(&crate::ml::Big::from_i128(10));
                    {
                        (n, place) = (n_2.clone(), place_2.clone());
                        continue;
                    }
                }
            }
        } else {
            place.clone()
        };
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Scans space-separated number words ("one hundred twenty-three").
// |  * @param {string[]} input
// |  * @param {number} pos
// |  * @returns {WordToken | null}
// |  */
// | function scanSpacedNumber(input, pos) {
// |   const composer = Composer.default();
// |   /** @type {number | null} */
// |   let end = null;
// |   let start = pos;
// |   while (true) {
// |     const wordStop = wordEnd(input, start);
// |     const word = lookup(input, start, wordStop);
// |     if (word !== null && composer.accept(word)) {
// |       end = wordStop;
// |     } else {
// |       break;
// |     }
// |     // "twenty-one": a hyphen directly between two number words.
// |     const next = input[wordStop] === '-'
// |       && input[wordStop + 1] !== undefined && isWordChar(input[wordStop + 1])
// |       ? wordStop + 1
// |       : skipWhitespace(input, wordStop);
// |     if (next === wordStop || next >= input.length || !isWordChar(input[next])) {
// |       break;
// |     }
// |     start = next;
// |   }
// |   if (end === null) {
// |     return null;
// |   }
// |   return [
// |     TokenKind.Number(composer.value().toString()),
// |     pos,
// |     end,
// |     text(input, pos, end),
// |   ];
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * Segments an unspaced run of CJK characters ("二十三加五") into vocabulary
// |  * words. The run must consist of vocabulary only, so that words like 千克
// |  * (kilogram) or 十二月 (December) keep their own meaning.
// |  * @param {string[]} input
// |  * @param {number} pos
// |  * @returns {WordToken[] | null}
// |  */
// | function scanCjk(input, pos) {
// |   let runEnd = pos;
// |   while (runEnd < input.length && isCjk(input[runEnd])) {
// |     runEnd += 1;
// |   }
// |
// |   /** @type {Array<[number, number, Word]>} */
// |   const segments = [];
// |   let start = pos;
// |   while (start < runEnd) {
// |     const longest = Math.min(vocabulary().longestWord, runEnd - start);
// |     /** @type {[number, Word] | null} */
// |     let found = null;
// |     for (let len = longest; len >= 1; len--) {
// |       const word = lookup(input, start, start + len);
// |       if (word !== null) {
// |         found = [start + len, word];
// |         break;
// |       }
// |     }
// |     if (found === null) {
// |       return null;
// |     }
// |     const [end, word] = found;
// |     segments.push([start, end, word]);
// |     start = end;
// |   }
// |
// |   /** @type {WordToken[]} */
// |   const tokens = [];
// |   /** @type {{ value: [number, number, Composer] | null }} */
// |   const number = { value: null };
// |   for (const [segStart, end, word] of segments) {
// |     if (word.meaning.tag === 'Operator') {
// |       flushNumber(tokens, number, input);
// |       tokens.push([word.meaning._0, segStart, end, text(input, segStart, end)]);
// |       continue;
// |     }
// |     if (number.value !== null) {
// |       const composer = number.value[2];
// |       if (composer.accept(word)) {
// |         number.value[1] = end;
// |         continue;
// |       }
// |       flushNumber(tokens, number, input);
// |     }
// |     const composer = Composer.default();
// |     if (!composer.accept(word)) {
// |       return null;
// |     }
// |     number.value = [segStart, end, composer];
// |   }
// |   flushNumber(tokens, number, input);
// |   return tokens;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {WordToken[]} tokens
// |  * @param {{ value: [number, number, Composer] | null }} number `&mut Option<...>`
// |  * @param {string[]} input
// |  */
// | function flushNumber(tokens, number, input) {
// |   const taken = number.value;
// |   number.value = null;
// |   if (taken !== null) {
// |     const [start, end, composer] = taken;
// |     tokens.push([
// |       TokenKind.Number(composer.value().toString()),
// |       start,
// |       end,
// |       text(input, start, end),
// |     ]);
// |   }
// | }
