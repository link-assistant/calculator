/**
 * Spelled numbers and arithmetic operator words as an optional front end
 * (issue #222).
 *
 * The vocabulary lives in `data/words/arithmetic-words.lino` (one link per
 * surface form), so adding a language means editing data, not code. The lexer
 * calls [`scan`] at the start of every word: when the input there is a number
 * word ("twenty one", "сто двадцать", "二十三") or an operator phrase
 * ("multiplied by", "умножить на", "除以"), it receives ready-made `Number` or
 * operator tokens; otherwise it lexes an ordinary identifier.
 *
 * `input: &[char]` is an array of one-code-point strings (`[...text]`), so
 * every position is a char index, as in Rust.
 */

import { isWordChar, TokenKind } from './lexer.js';
import { CurrencyDatabase } from '../types/currency.js';
import { ARITHMETIC_WORDS_LINO } from '../generated/arithmetic-words-lino.js';
import { eq, ok, panic } from '../rust/core.js';
import { U128 } from '../rust/num.js';
import {
  charCount,
  debugStr,
  isAsciiAlphabetic,
  isUppercase,
  isWhitespace,
  lines,
  splitOnce,
  splitWhitespace,
  startsWith,
  stripPrefix,
  stripSuffix,
  trim,
} from '../rust/str.js';

/** What a single vocabulary word means. */
class Meaning {
  /** @param {string} tag @param {object} [fields] */
  constructor(tag, fields) {
    /** @type {string} */
    this.tag = tag;
    Object.assign(this, fields);
    Object.freeze(this);
  }

  /** A cardinal number ("seven", "сорок"). @param {bigint} n */
  static Number(n) { return new Meaning('Number', { _0: n }); }

  /** A multiplier of what precedes it ("hundred", "тысяча", "十"). @param {bigint} s */
  static Scale(s) { return new Meaning('Scale', { _0: s }); }

  /** An arithmetic operator. @param {TokenKind} op */
  static Operator(op) { return new Meaning('Operator', { _0: op }); }
}

/**
 * @typedef {{ lang: string, meaning: Meaning & { _0: any } }} Word
 */

/**
 * The parsed vocabulary.
 * @typedef {object} Vocabulary
 * @property {Map<string, Word>} words Single words (lowercase) to their meaning.
 * @property {Array<[string[], TokenKind]>} phrases Multi-word operator phrases (lowercase words), longest first.
 * @property {number} longestWord Length in characters of the longest single word, for CJK segmentation.
 */

/** @type {Vocabulary | null} */
let VOCABULARY = null;

/**
 * @internal
 * @returns {Vocabulary}
 */
export function vocabulary() {
  if (VOCABULARY === null) {
    const result = parseVocabulary(ARITHMETIC_WORDS_LINO);
    if (typeof result === 'string') {
      panic(`data/words/arithmetic-words.lino is valid: ${debugStr(result)}`);
    }
    VOCABULARY = result;
  }
  return VOCABULARY;
}

/**
 * @param {string} symbol
 * @returns {TokenKind | null}
 */
function operatorKind(symbol) {
  switch (symbol) {
    case '+': return TokenKind.Plus;
    case '-': return TokenKind.Minus;
    case '*': return TokenKind.Star;
    case '/': return TokenKind.Slash;
    case '%': return TokenKind.Percent;
    case '^': return TokenKind.Caret;
    case 'percent': return TokenKind.PercentWord;
    case 'percent-prefix': return TokenKind.PercentPrefix;
    case 'portion': return TokenKind.Portion;
    case 'increase-from': return TokenKind.IncreaseFrom;
    case 'decrease-from': return TokenKind.DecreaseFrom;
    default: return null;
  }
}

/**
 * Parses lines of the form `(kind lang value 'surface form')`.
 *
 * `Result<Vocabulary, String>`: returns the vocabulary, or the error message
 * as a string.
 * @internal
 * @param {string} content
 * @returns {Vocabulary | string}
 */
export function parseVocabulary(content) {
  /** @type {Vocabulary} */
  const vocabulary = { words: new Map(), phrases: [], longestWord: 0 };
  const allLines = lines(content);
  for (let index = 0; index < allLines.length; index++) {
    const line = trim(allLines[index]);
    if (line === '' || startsWith(line, '//')) {
      continue;
    }
    const error = () => `arithmetic-words.lino:${index + 1}: malformed link ${debugStr(line)}`;
    const rest = stripPrefix(line, '(');
    const body = rest === null ? null : stripSuffix(rest, ')');
    if (body === null) {
      return error();
    }
    const split = splitOnce(body, '\'');
    if (split === null) {
      return error();
    }
    const [head, rawSurface] = split;
    const stripped = stripSuffix(rawSurface, '\'');
    if (stripped === null) {
      return error();
    }
    const surface = stripped.toLowerCase();
    const fields = splitWhitespace(head);
    if (fields.length !== 3) {
      return error();
    }
    const [kind, lang, value] = fields;
    let meaning;
    switch (kind) {
      case 'number': {
        const n = ok(() => U128.parse(value));
        if (n === null) {
          return error();
        }
        meaning = Meaning.Number(n);
        break;
      }
      case 'scale': {
        const n = ok(() => U128.parse(value));
        if (n === null) {
          return error();
        }
        meaning = Meaning.Scale(n);
        break;
      }
      case 'operator': {
        const op = operatorKind(value);
        if (op === null) {
          return error();
        }
        meaning = Meaning.Operator(op);
        break;
      }
      default:
        return error();
    }

    const words = splitWhitespace(surface);
    if (words.length === 0) {
      return error();
    } else if (words.length === 1) {
      vocabulary.longestWord = Math.max(vocabulary.longestWord, charCount(surface));
      const word = { lang, meaning: /** @type {Meaning & { _0: any }} */ (meaning) };
      const previous = vocabulary.words.get(surface);
      vocabulary.words.set(surface, word);
      if (previous !== undefined) {
        if (!eq(previous.meaning, word.meaning)) {
          return `${error()}: ${debugStr(surface)} has two meanings`;
        }
      }
    } else if (meaning.tag === 'Operator') {
      vocabulary.phrases.push([words, /** @type {any} */ (meaning)._0]);
    } else {
      return `${error()}: only operators may span words`;
    }
  }
  vocabulary.phrases.sort((a, b) => b[0].length - a[0].length);
  return vocabulary;
}

/**
 * A token produced from words: kind, start, end (char positions) and text.
 * @typedef {[TokenKind, number, number, string]} WordToken
 */

/**
 * Recognizes number words or an operator phrase starting at `pos`.
 *
 * Returns the tokens (at least one) in input order, or `None` when the word
 * at `pos` is not arithmetic vocabulary.
 * @param {string[]} input
 * @param {number} pos
 * @returns {WordToken[] | null}
 */
export function scan(input, pos) {
  const longest = Math.min(vocabulary().longestWord, input.length - pos);
  for (let len = longest; len >= 1; len--) {
    const word = lookup(input, pos, pos + len);
    if (word !== null && word.meaning.tag === 'Operator') {
      /** @type {TokenKind} */
      const op = word.meaning._0;
      if ((op.tag === 'Portion' || op.tag === 'PercentPrefix')
        && (op.tag !== 'Portion' || portionHasPercent(input, pos + len))
        && (pos + len === input.length || !isAsciiAlphabetic(input[pos + len]))) {
        return [[op, pos, pos + len, text(input, pos, pos + len)]];
      }
    }
  }
  if (isCjk(input[pos])) {
    return scanCjk(input, pos);
  }
  const token = scanPhrase(input, pos);
  if (token !== null) {
    return [token];
  }
  const number = scanSpacedNumber(input, pos);
  return number === null ? null : [number];
}

// A possession word also occurs in dates, units, and powers. Recognize the
// whole-before-part form only when the following text contains percent vocabulary.
/**
 * @param {string[]} input
 * @param {number} start
 * @returns {boolean}
 */
function portionHasPercent(input, start) {
  for (let pos = start; pos < input.length; pos++) {
    if (input[pos] === '%') {
      return true;
    }
    const longest = Math.min(vocabulary().longestWord, input.length - pos);
    for (let len = longest; len >= 1; len--) {
      const word = lookup(input, pos, pos + len);
      if (word !== null && word.meaning.tag === 'Operator'
        && (word.meaning._0.tag === 'PercentPrefix' || word.meaning._0.tag === 'PercentWord')) {
        return true;
      }
    }
    if (['+', '-', '*', '/', '(', ')'].includes(input[pos])) {
      break;
    }
  }
  return false;
}

/** @param {string} ch */
function isCjk(ch) {
  const cp = /** @type {number} */ (ch.codePointAt(0));
  return (cp >= 0x3400 && cp <= 0x4DBF) || (cp >= 0x4E00 && cp <= 0x9FFF) || (cp >= 0xF900 && cp <= 0xFAFF);
}

/**
 * End of the word that starts at `pos`.
 * @param {string[]} input
 * @param {number} pos
 */
function wordEnd(input, pos) {
  let end = pos;
  while (end < input.length && isWordChar(input[end])) {
    end += 1;
  }
  return end;
}

/**
 * @param {string[]} input
 * @param {number} start
 * @param {number} end
 */
function text(input, start, end) {
  return input.slice(start, end).join('');
}

/**
 * An all-uppercase word that names a known currency ("DOS", "MIL", "ONE")
 * keeps its currency meaning, like "SEK" does over German "sek" seconds.
 * @param {string} word
 */
function isCurrencyCode(word) {
  return charCount(word) >= 2
    && [...word].every(isUppercase)
    && CurrencyDatabase.parseCurrency(word) !== null;
}

/**
 * Looks up the word spanning `start..end`, skipping currency codes.
 * @param {string[]} input
 * @param {number} start
 * @param {number} end
 * @returns {Word | null}
 */
function lookup(input, start, end) {
  const word = text(input, start, end);
  if (start === end || isCurrencyCode(word)) {
    return null;
  }
  return vocabulary().words.get(word.toLowerCase()) ?? null;
}

/**
 * @param {string[]} input
 * @param {number} pos
 */
function skipWhitespace(input, pos) {
  while (pos < input.length && isWhitespace(input[pos])) {
    pos += 1;
  }
  return pos;
}

/**
 * Matches the longest operator phrase (or single operator word) at `pos`.
 * @param {string[]} input
 * @param {number} pos
 * @returns {WordToken | null}
 */
function scanPhrase(input, pos) {
  const firstEnd = wordEnd(input, pos);
  const firstText = text(input, pos, firstEnd);
  if (isCurrencyCode(firstText)) {
    return null;
  }
  const first = firstText.toLowerCase();

  /** @type {[TokenKind, number] | null} */
  let matched = null;
  for (const [words, op] of vocabulary().phrases) {
    if (words[0] !== first) {
      continue;
    }
    let end = firstEnd;
    let found = true;
    for (const word of words.slice(1)) {
      const start = skipWhitespace(input, end);
      const nextEnd = wordEnd(input, start);
      if (start === end || text(input, start, nextEnd).toLowerCase() !== word) {
        found = false;
        break;
      }
      end = nextEnd;
    }
    if (found) {
      matched = [op, end];
      break;
    }
  }
  if (matched === null) {
    const word = vocabulary().words.get(first);
    if (word !== undefined && word.meaning.tag === 'Operator') {
      matched = [word.meaning._0, firstEnd];
    }
  }

  if (matched === null) {
    return null;
  }
  const [op, end] = matched;
  // `mod(17, 5)` is the function, not the operator.
  if ((op.tag === 'Percent' || op.tag === 'PercentWord') && input[end] === '(') {
    return null;
  }
  return [op, pos, end, text(input, pos, end)];
}

/** Combines consecutive number words into one value. */
class Composer {
  constructor() {
    /** @type {bigint} */
    this.total = 0n;
    /** @type {bigint} */
    this.current = 0n;
    /**
     * The last number word added since the last scale word.
     * @type {bigint | null}
     */
    this.last = null;
    /** @type {boolean} */
    this.any = false;
    /** @type {string | null} */
    this.lang = null;
  }

  /** @returns {Composer} */
  static default() {
    return new Composer();
  }

  /**
   * Adds a word if it continues the number; `false` ends the number.
   * @param {Word} word
   * @returns {boolean}
   */
  accept(word) {
    if (this.lang !== null && this.lang !== word.lang) {
      return false;
    }
    let accepted;
    switch (word.meaning.tag) {
      case 'Number':
        accepted = this.add(word.meaning._0);
        break;
      case 'Scale':
        accepted = this.scale(word.meaning._0);
        break;
      default:
        accepted = false;
    }
    if (accepted) {
      this.any = true;
      this.lang = word.lang;
    }
    return accepted;
  }

  /** @param {bigint} n */
  add(n) {
    // "twenty one", "сто двадцать" and "two hundred five" continue a
    // number; "two three" or "ten five" do not.
    const continues = this.last === null ? true : this.last >= 20n && n < placeValue(this.last);
    if (!continues || (this.any && n === 0n)) {
      return false;
    }
    const current = U128.checkedAdd(this.current, n);
    if (current === null) {
      return false;
    }
    this.current = current;
    this.last = n;
    return true;
  }

  /** @param {bigint} s */
  scale(s) {
    if (s >= 1000n) {
      // "thousand", "million", "万": close the group below it.
      if (this.any && this.current === 0n) {
        return false;
      }
      const group = U128.checkedMul(this.current > 1n ? this.current : 1n, s);
      const total = group === null ? null : U128.checkedAdd(this.total, group);
      if (total === null) {
        return false;
      }
      this.total = total;
      this.current = 0n;
    } else {
      // "hundred", "十", "百" multiply only the preceding digit, so that
      // 三百二十 is 3·100 + 2·10.
      if (this.any && this.last === null) {
        return false;
      }
      const digit = this.last ?? 0n;
      const scaled = U128.checkedMul(digit > 1n ? digit : 1n, s);
      const current = scaled === null ? null : U128.checkedAdd(this.current - digit, scaled);
      if (current === null) {
        return false;
      }
      this.current = current;
    }
    this.last = null;
    return true;
  }

  value() {
    return this.total + this.current;
  }
}

/**
 * The place value of the lowest non-zero digit (20 → 10, 100 → 100).
 * @param {bigint} n
 */
function placeValue(n) {
  let place = 1n;
  while (n > 0n && n % 10n === 0n) {
    n /= 10n;
    place *= 10n;
  }
  return place;
}

/**
 * Scans space-separated number words ("one hundred twenty-three").
 * @param {string[]} input
 * @param {number} pos
 * @returns {WordToken | null}
 */
function scanSpacedNumber(input, pos) {
  const composer = Composer.default();
  /** @type {number | null} */
  let end = null;
  let start = pos;
  while (true) {
    const wordStop = wordEnd(input, start);
    const word = lookup(input, start, wordStop);
    if (word !== null && composer.accept(word)) {
      end = wordStop;
    } else {
      break;
    }
    // "twenty-one": a hyphen directly between two number words.
    const next = input[wordStop] === '-'
      && input[wordStop + 1] !== undefined && isWordChar(input[wordStop + 1])
      ? wordStop + 1
      : skipWhitespace(input, wordStop);
    if (next === wordStop || next >= input.length || !isWordChar(input[next])) {
      break;
    }
    start = next;
  }
  if (end === null) {
    return null;
  }
  return [
    TokenKind.Number(composer.value().toString()),
    pos,
    end,
    text(input, pos, end),
  ];
}

/**
 * Segments an unspaced run of CJK characters ("二十三加五") into vocabulary
 * words. The run must consist of vocabulary only, so that words like 千克
 * (kilogram) or 十二月 (December) keep their own meaning.
 * @param {string[]} input
 * @param {number} pos
 * @returns {WordToken[] | null}
 */
function scanCjk(input, pos) {
  let runEnd = pos;
  while (runEnd < input.length && isCjk(input[runEnd])) {
    runEnd += 1;
  }

  /** @type {Array<[number, number, Word]>} */
  const segments = [];
  let start = pos;
  while (start < runEnd) {
    const longest = Math.min(vocabulary().longestWord, runEnd - start);
    /** @type {[number, Word] | null} */
    let found = null;
    for (let len = longest; len >= 1; len--) {
      const word = lookup(input, start, start + len);
      if (word !== null) {
        found = [start + len, word];
        break;
      }
    }
    if (found === null) {
      return null;
    }
    const [end, word] = found;
    segments.push([start, end, word]);
    start = end;
  }

  /** @type {WordToken[]} */
  const tokens = [];
  /** @type {{ value: [number, number, Composer] | null }} */
  const number = { value: null };
  for (const [segStart, end, word] of segments) {
    if (word.meaning.tag === 'Operator') {
      flushNumber(tokens, number, input);
      tokens.push([word.meaning._0, segStart, end, text(input, segStart, end)]);
      continue;
    }
    if (number.value !== null) {
      const composer = number.value[2];
      if (composer.accept(word)) {
        number.value[1] = end;
        continue;
      }
      flushNumber(tokens, number, input);
    }
    const composer = Composer.default();
    if (!composer.accept(word)) {
      return null;
    }
    number.value = [segStart, end, composer];
  }
  flushNumber(tokens, number, input);
  return tokens;
}

/**
 * @param {WordToken[]} tokens
 * @param {{ value: [number, number, Composer] | null }} number `&mut Option<...>`
 * @param {string[]} input
 */
function flushNumber(tokens, number, input) {
  const taken = number.value;
  number.value = null;
  if (taken !== null) {
    const [start, end, composer] = taken;
    tokens.push([
      TokenKind.Number(composer.value().toString()),
      start,
      end,
      text(input, start, end),
    ]);
  }
}
