// meta-language:carried JavaScript export_statement (unsupported)
// | /** Lexer for tokenizing calculator input. */
// | export class Lexer {
// |   /**
// |    * @param {{ input: string[], pos: number, previousTokenCanEndValue: boolean,
// |    *   previousIdentifier: string | null, parentheses: boolean[], decimalComma: boolean,
// |    *   pending: Token[] }} fields
// |    */
// |   constructor({ input, pos, previousTokenCanEndValue, previousIdentifier, parentheses, decimalComma, pending }) {
// |     this.input = input;
// |     this.pos = pos;
// |     this.previousTokenCanEndValue = previousTokenCanEndValue;
// |     this.previousIdentifier = previousIdentifier;
// |     this.parentheses = parentheses;
// |     this.decimalComma = decimalComma;
// |     /** Tokens already scanned from number or operator words. */
// |     this.pending = pending;
// |   }
// |
// |   /**
// |    * Creates a new lexer for the given input.
// |    * @param {string} input
// |    * @returns {Lexer}
// |    */
// |   static new(input) {
// |     return new Lexer({
// |       input: [...input],
// |       pos: 0,
// |       previousTokenCanEndValue: false,
// |       previousIdentifier: null,
// |       parentheses: [],
// |       decimalComma: false,
// |       pending: [],
// |     });
// |   }
// |
// |   /**
// |    * @param {string} input
// |    * @returns {Lexer}
// |    */
// |   static withDecimalCommas(input) {
// |     const lexer = Lexer.new(input);
// |     lexer.decimalComma = true;
// |     return lexer;
// |   }
// |
// |   /**
// |    * Tokenizes the entire input; throws `Err(CalculatorError)`.
// |    * @returns {Token[]}
// |    */
// |   tokenize() {
// |     const tokens = [];
// |
// |     while (true) {
// |       const token = this.nextToken();
// |       const isEof = token.kind.tag === 'Eof';
// |       tokens.push(token);
// |       if (isEof) {
// |         break;
// |       }
// |     }
// |
// |     return tokens;
// |   }
// |
// |   /**
// |    * Returns the next token; throws `Err(CalculatorError)`.
// |    * @returns {Token}
// |    */
// |   nextToken() {
// |     const token = this.pending.length > 0
// |       ? /** @type {Token} */ (this.pending.shift())
// |       : this.scanToken();
// |
// |     switch (token.kind.tag) {
// |       case 'LeftParen': {
// |         const fn = this.previousIdentifier !== null && isMathFunction(this.previousIdentifier);
// |         this.parentheses.push(fn);
// |         break;
// |       }
// |       case 'RightParen':
// |         this.parentheses.pop();
// |         break;
// |       default:
// |     }
// |     this.previousIdentifier = token.kind.tag === 'Identifier' ? token.kind._0 : null;
// |     this.previousTokenCanEndValue = [
// |       'Number',
// |       'Percent',
// |       'PercentWord',
// |       'DateLiteral',
// |       'Identifier',
// |       'RightParen',
// |       'Superscript',
// |       'Bang',
// |     ].includes(token.kind.tag);
// |
// |     return token;
// |   }
// |
// |   /** @returns {Token} */
// |   scanToken() {
// |     this.skipWhitespace();
// |
// |     if (this.isAtEnd()) {
// |       return Token.new(TokenKind.Eof, this.pos, this.pos, '');
// |     }
// |
// |     const start = this.pos;
// |     const ch = this.current();
// |
// |     // Single-character tokens
// |     /** @type {Token} */
// |     let token;
// |     switch (ch) {
// |       case '±':
// |         this.advance();
// |         token = Token.new(TokenKind.PlusMinus, start, this.pos, ch);
// |         break;
// |       case '+':
// |         this.advance();
// |         token = Token.new(TokenKind.Plus, start, this.pos, '+');
// |         break;
// |       case '-': case '−':
// |         this.advance();
// |         token = Token.new(TokenKind.Minus, start, this.pos, ch);
// |         break;
// |       case '*':
// |         this.advance();
// |         if (!this.isAtEnd()
// |           && this.current() === '*'
// |           && this.previousTokenCanEndValue
// |           && this.doubleStarHasRightOperand()) {
// |           this.advance();
// |           token = Token.new(TokenKind.Caret, start, this.pos, '**');
// |         } else {
// |           token = Token.new(TokenKind.Star, start, this.pos, '*');
// |         }
// |         break;
// |       case '×': case '·': case '⋅':
// |         this.advance();
// |         token = Token.new(TokenKind.Star, start, this.pos, ch);
// |         break;
// |       case '?':
// |         this.advance();
// |         token = Token.new(TokenKind.Question, start, this.pos, '?');
// |         break;
// |       case '/': case '÷':
// |         this.advance();
// |         token = Token.new(TokenKind.Slash, start, this.pos, ch);
// |         break;
// |       case '^':
// |         this.advance();
// |         token = Token.new(TokenKind.Caret, start, this.pos, '^');
// |         break;
// |       case '%':
// |         this.advance();
// |         token = Token.new(TokenKind.Percent, start, this.pos, '%');
// |         break;
// |       case '(':
// |         this.advance();
// |         token = Token.new(TokenKind.LeftParen, start, this.pos, '(');
// |         break;
// |       case ')':
// |         this.advance();
// |         token = Token.new(TokenKind.RightParen, start, this.pos, ')');
// |         break;
// |       case ':':
// |         this.advance();
// |         token = Token.new(TokenKind.Colon, start, this.pos, ':');
// |         break;
// |       case ',':
// |         this.advance();
// |         token = Token.new(TokenKind.Comma, start, this.pos, ',');
// |         break;
// |       case '=':
// |         this.advance();
// |         if (!this.isAtEnd() && this.current() === '=') {
// |           this.advance();
// |           token = Token.new(TokenKind.DoubleEquals, start, this.pos, '==');
// |         } else {
// |           token = Token.new(TokenKind.Equals, start, this.pos, '=');
// |         }
// |         break;
// |       case '!':
// |         this.advance();
// |         if (!this.isAtEnd() && this.current() === '=') {
// |           this.advance();
// |           token = Token.new(TokenKind.NotEqual, start, this.pos, '!=');
// |         } else {
// |           token = Token.new(TokenKind.Bang, start, this.pos, '!');
// |         }
// |         break;
// |       case '≠':
// |         this.advance();
// |         token = Token.new(TokenKind.NotEqual, start, this.pos, ch);
// |         break;
// |       case '≤':
// |         this.advance();
// |         token = Token.new(TokenKind.LessOrEqual, start, this.pos, ch);
// |         break;
// |       case '≥':
// |         this.advance();
// |         token = Token.new(TokenKind.GreaterOrEqual, start, this.pos, ch);
// |         break;
// |       case '<':
// |         this.advance();
// |         if (!this.isAtEnd() && this.current() === '=') {
// |           this.advance();
// |           token = Token.new(TokenKind.LessOrEqual, start, this.pos, '<=');
// |         } else {
// |           token = Token.new(TokenKind.Less, start, this.pos, '<');
// |         }
// |         break;
// |       case '>':
// |         this.advance();
// |         if (!this.isAtEnd() && this.current() === '=') {
// |           this.advance();
// |           token = Token.new(TokenKind.GreaterOrEqual, start, this.pos, '>=');
// |         } else {
// |           token = Token.new(TokenKind.Greater, start, this.pos, '>');
// |         }
// |         break;
// |       default:
// |         if (isAsciiDigit(ch)) {
// |           // Prefer a full numeric date literal (e.g. 2026-01-22, 15.10.2025)
// |           // over splitting it into separate numbers and operators.
// |           const date = this.tryScanDate();
// |           if (date !== null) {
// |             const [text, end] = date;
// |             token = Token.new(TokenKind.DateLiteral(text), start, end, text);
// |             this.pos = end;
// |           } else {
// |             token = this.scanNumber();
// |           }
// |         } else if (ch === '.') {
// |           token = this.scanNumber();
// |         } else if (superscriptValue(ch) !== null) {
// |           token = this.scanSuperscript();
// |         } else if (ch === '√' || ch === '∛') {
// |           this.advance();
// |           const name = ch === '√' ? 'sqrt' : 'cbrt';
// |           token = Token.new(TokenKind.Identifier(name), start, this.pos, ch);
// |         } else if (isAlphabetic(ch)) {
// |           token = this.scanWords() ?? this.scanIdentifier();
// |         } else if (['°', '℃', '℉', '\'', '′', '"', '″'].includes(ch)) {
// |           // Degree signs, `℃`/`℉`, and the foot/inch primes of `6'2"` are
// |           // unit symbols (issue #235); `°C` and `°F` stay one identifier.
// |           token = this.scanUnitSymbol();
// |         } else if (['$', '€', '£', '¥', '₽', '₹', '₩', '₿', '₫', '₸'].includes(ch)) {
// |           // Currency symbols used as prefix notation (e.g., $10, €5, £3)
// |           // These are recognized as single-character identifiers and mapped to ISO codes
// |           // by CurrencyDatabase::parse_currency().
// |           this.advance();
// |           const symbol = ch;
// |           token = Token.new(TokenKind.Identifier(symbol), start, this.pos, symbol);
// |         } else {
// |           return err(CalculatorError.parse(`Unexpected character '${ch}' at position ${start}`));
// |         }
// |     }
// |
// |     return token;
// |   }
// |
// |   /**
// |    * Number words and operator phrases from `data/words` (issue #222).
// |    * @returns {Token | null}
// |    */
// |   scanWords() {
// |     const tokens = arithmeticWords.scan(this.input, this.pos);
// |     if (tokens === null) {
// |       return null;
// |     }
// |     const mapped = tokens.map(([kind, start, end, text]) => Token.new(kind, start, end, text));
// |     if (mapped.length === 0) {
// |       return null;
// |     }
// |     const first = /** @type {Token} */ (mapped.shift());
// |     this.pending.push(...mapped);
// |     this.pos = this.pending.length > 0 ? this.pending[this.pending.length - 1].end : first.end;
// |     return first;
// |   }
// |
// |   /** @returns {boolean} */
// |   doubleStarHasRightOperand() {
// |     const character = this.input.slice(this.pos + 1).find((c) => !isWhitespace(c));
// |     return character !== undefined
// |       && (isAsciiDigit(character)
// |         || isAlphabetic(character)
// |         || ['.', '(', '+', '-', '−', '√', '∛'].includes(character));
// |   }
// |
// |   /**
// |    * Throws `Err(CalculatorError)`.
// |    * @returns {Token}
// |    */
// |   scanNumber() {
// |     const start = this.pos;
// |     if (this.current() === '.') {
// |       const next = this.peek();
// |       if (!(next !== null && isAsciiDigit(next))) {
// |         return err(CalculatorError.parse(`Unexpected character '.' at position ${start}`));
// |       }
// |     }
// |     const dateComponent = this.previousIdentifier !== null
// |       && DateTimeGrammar.looksLikeDatetime(this.previousIdentifier);
// |     const allowComma = !this.parentheses.includes(true) && !dateComponent;
// |     let [text, end] = localeNumbers.scan(this.input, start, allowComma, this.decimalComma);
// |     this.pos = end;
// |
// |     // Scientific notation is consumed only when the exponent is complete,
// |     // keeping an adjacent constant such as `2e` available for implicit
// |     // multiplication rather than accepting a malformed number.
// |     if (this.current() === 'e' || this.current() === 'E') {
// |       const exponentMarker = this.pos;
// |       let exponentPos = exponentMarker + 1;
// |       if (this.input[exponentPos] === '+' || this.input[exponentPos] === '-') {
// |         exponentPos += 1;
// |       }
// |       if (this.input[exponentPos] !== undefined && isAsciiDigit(this.input[exponentPos])) {
// |         text += this.current();
// |         this.advance();
// |         if (this.current() === '+' || this.current() === '-') {
// |           text += this.current();
// |           this.advance();
// |         }
// |         while (!this.isAtEnd()) {
// |           const ch = this.current();
// |           const next = this.peek();
// |           if (isAsciiDigit(ch)) {
// |             text += ch;
// |             this.advance();
// |           } else if (ch === '_'
// |             && text.length > 0
// |             && isAsciiDigit(text[text.length - 1])
// |             && next !== null && isAsciiDigit(next)) {
// |             this.advance();
// |           } else {
// |             break;
// |           }
// |         }
// |       }
// |     }
// |     end = this.pos;
// |
// |     const original = this.input.slice(start, this.pos).join('');
// |
// |     return Token.new(TokenKind.Number(text), start, end, original);
// |   }
// |
// |   /**
// |    * Scans a unit symbol that is not a word: `°`, `°C`, `°F`, `℃`, `℉`,
// |    * and the primes `'`/`′` (foot) and `"`/`″` (inch).
// |    * @returns {Token}
// |    */
// |   scanUnitSymbol() {
// |     const start = this.pos;
// |     const ch = this.current();
// |     this.advance();
// |     let text = ch;
// |     const next = this.peek();
// |     if (ch === '°'
// |       && (this.current() === 'C' || this.current() === 'F')
// |       && !(next !== null && isWordChar(next))) {
// |       text += this.current();
// |       this.advance();
// |     }
// |     return Token.new(TokenKind.Identifier(text), start, this.pos, text);
// |   }
// |
// |   /** @returns {Token} */
// |   scanSuperscript() {
// |     const start = this.pos;
// |     let normalized = '';
// |
// |     while (!this.isAtEnd()) {
// |       const value = superscriptValue(this.current());
// |       if (value === null) {
// |         break;
// |       }
// |       normalized += value;
// |       this.advance();
// |     }
// |
// |     const original = this.input.slice(start, this.pos).join('');
// |     return Token.new(TokenKind.Superscript(normalized), start, this.pos, original);
// |   }
// |
// |   /**
// |    * Attempts to scan a full numeric date literal starting at the current
// |    * position, returning the matched text and the end index on success.
// |    *
// |    * A date literal is three digit groups joined by a single, consistent
// |    * separator (`-`, `/`, or `.`), where exactly one of the outer groups is a
// |    * 4-digit year. The candidate is only accepted when it parses as a real
// |    * calendar date, so ordinary arithmetic such as `2026 - 1 - 22` (written
// |    * with spaces) or `1/3` is never swallowed.
// |    * @returns {[string, number] | null}
// |    */
// |   tryScanDate() {
// |     const len = this.input.length;
// |
// |     // Scans a run of ASCII digits from `j`, returning the end index.
// |     const scanDigits = (/** @type {number} */ j) => {
// |       let end = j;
// |       while (end < len && isAsciiDigit(this.input[end])) {
// |         end += 1;
// |       }
// |       return end;
// |     };
// |
// |     // First digit group.
// |     const start = this.pos;
// |     const e1 = scanDigits(start);
// |     const l1 = e1 - start;
// |     if (l1 === 0) {
// |       return null;
// |     }
// |
// |     // First separator.
// |     if (e1 >= len) {
// |       return null;
// |     }
// |     const sep = this.input[e1];
// |     if (sep !== '-' && sep !== '/' && sep !== '.') {
// |       return null;
// |     }
// |
// |     // Second digit group.
// |     const e2 = scanDigits(e1 + 1);
// |     const l2 = e2 - (e1 + 1);
// |     if (l2 === 0) {
// |       return null;
// |     }
// |
// |     // Second separator must match the first.
// |     if (e2 >= len || this.input[e2] !== sep) {
// |       return null;
// |     }
// |
// |     // Third digit group.
// |     const e3 = scanDigits(e2 + 1);
// |     const l3 = e3 - (e2 + 1);
// |     if (l3 === 0) {
// |       return null;
// |     }
// |
// |     // Reject anything that continues into a 4th component or more digits
// |     // (e.g. version strings like `1.2.3.4` or `12.3456`).
// |     if (e3 < len && (this.input[e3] === sep || isAsciiDigit(this.input[e3]))) {
// |       return null;
// |     }
// |
// |     // Require exactly one 4-digit year in an outer position.
// |     const yearFirst = l1 === 4 && l2 >= 1 && l2 <= 2 && l3 >= 1 && l3 <= 2;
// |     const yearLast = l3 === 4 && l1 >= 1 && l1 <= 2 && l2 >= 1 && l2 <= 2;
// |     if (!yearFirst && !yearLast) {
// |       return null;
// |     }
// |
// |     const candidate = this.input.slice(start, e3).join('');
// |     if (isOk(() => DateTime.parse(candidate))) {
// |       return [candidate, e3];
// |     }
// |     return null;
// |   }
// |
// |   /** @returns {Token} */
// |   scanIdentifier() {
// |     const start = this.pos;
// |     let text = '';
// |
// |     while (!this.isAtEnd()) {
// |       const ch = this.current();
// |       // Accept alphanumeric, underscore, and Unicode combining marks (Mn/Mc/Me).
// |       // Combining marks are needed for scripts like Devanagari (Hindi) where
// |       // virama (्, U+094D) and dependent vowels (ा, ि, etc.) are part of words
// |       // but not classified as alphabetic.
// |       // Superscripts end a word so that `m²` is `m` squared.
// |       if ((isAlphanumeric(ch) || ch === '_' || isUnicodeMark(ch))
// |         && superscriptValue(ch) === null) {
// |         text += ch;
// |         this.advance();
// |       } else {
// |         break;
// |       }
// |     }
// |
// |     // Abbreviations are commonly written with a trailing dot ("руб.", "кг.", "янв.").
// |     // Consume that dot when it terminates the word, i.e. when it is not followed by
// |     // another word character (so "янв.2026" or "3.5" are left untouched).
// |     const next = this.peek();
// |     if (text !== ''
// |       && this.current() === '.'
// |       && !(next !== null && (isAlphanumeric(next) || next === '_' || isUnicodeMark(next)))) {
// |       this.advance();
// |     }
// |
// |     // Check for keywords (including multilingual equivalents)
// |     /** @type {TokenKind} */
// |     let kind;
// |     switch (text.toLowerCase()) {
// |       // Operator words ("plus", "multiplied by", "умножить на", …) come
// |       // from data/words/arithmetic-words.lino via `scan_words`; `mod(`
// |       // is left to us as the function name.
// |       case 'mod': case 'modulo': kind = TokenKind.Identifier('mod'); break;
// |       case 'π': kind = TokenKind.Identifier('pi'); break;
// |       case 'fact': kind = TokenKind.Identifier('factorial'); break;
// |       case 'arcsin': kind = TokenKind.Identifier('asin'); break;
// |       case 'arccos': kind = TokenKind.Identifier('acos'); break;
// |       case 'arctan': kind = TokenKind.Identifier('atan'); break;
// |       case 'at': kind = TokenKind.At; break;
// |       case 'as': kind = TokenKind.As; break;
// |       case 'in': kind = TokenKind.In; break;
// |       case 'to': kind = TokenKind.To; break;
// |       case 'until': kind = TokenKind.Until; break;
// |       case 'of': kind = TokenKind.Of; break;
// |       case 'and': kind = TokenKind.And; break;
// |       case 'compare': kind = TokenKind.Compare; break;
// |       case 'vs': case 'versus': kind = TokenKind.Vs; break;
// |       // Russian: "от" means "of/from" in percent-of expressions (e.g. "38% от 100к").
// |       case 'от': kind = TokenKind.Of; break;
// |       // Russian: "в" means "in/into" (e.g. "1000 рублей в долларах")
// |       case 'в': kind = TokenKind.In; break;
// |       // Russian: "по" means "by/according to" for timezone context (e.g. "11 по мск").
// |       case 'по': kind = TokenKind.At; break;
// |       // Russian: "на" means "at/on" for temporal context (e.g. "1000 рублей на 11 апреля 2026")
// |       case 'на': kind = TokenKind.At; break;
// |       // French: "en" means "in/into" (e.g. "1000 dollars en euros")
// |       // Note: German "in" is identical to English "in", no extra entry needed.
// |       case 'en': kind = TokenKind.In; break;
// |       // Chinese: conversion phrases (e.g. "1000美元换成欧元")
// |       case '换成': case '兑换成': case '转换为': case '兑成': case '转为': kind = TokenKind.To; break;
// |       // Hindi: "में" means "in" as postposition (e.g. "1000 डॉलर में यूरो")
// |       case 'में': kind = TokenKind.In; break;
// |       // Arabic: "إلى" means "to/into" (e.g. "1000 دولار إلى يورو")
// |       case 'إلى': kind = TokenKind.To; break;
// |       default: kind = TokenKind.Identifier(text);
// |     }
// |
// |     return Token.new(kind, start, this.pos, text);
// |   }
// |
// |   skipWhitespace() {
// |     while (!this.isAtEnd() && isWhitespace(this.current())) {
// |       this.advance();
// |     }
// |   }
// |
// |   /** @returns {string} */
// |   current() {
// |     return this.input[this.pos] ?? '\0';
// |   }
// |
// |   /** @returns {string | null} */
// |   peek() {
// |     return this.input[this.pos + 1] ?? null;
// |   }
// |
// |   advance() {
// |     this.pos += 1;
// |   }
// |
// |   /** @returns {boolean} */
// |   isAtEnd() {
// |     return this.pos >= this.input.length;
// |   }
// | }
