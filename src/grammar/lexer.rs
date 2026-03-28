//! Lexer for tokenizing calculator input.

use crate::error::CalculatorError;

/// Checks if a character is a Unicode combining mark (General Category M).
///
/// This includes:
/// - Mn (Mark, Nonspacing) — e.g., Devanagari virama ्, Arabic fathah  َ
/// - Mc (Mark, Spacing Combining) — e.g., Devanagari dependent vowels ा ि ी
/// - Me (Mark, Enclosing) — rare, e.g., combining enclosing circle ⃝
///
/// These characters are integral parts of words in scripts like Devanagari (Hindi),
/// Arabic, and Thai, but are not classified as `is_alphabetic()` in Rust.
fn is_unicode_mark(ch: char) -> bool {
    // Unicode General Category "M" (Mark) covers Mn, Mc, and Me.
    // We check the ranges for Devanagari (U+0900-U+097F marks), Arabic (U+0610-U+065F),
    // and other common combining mark blocks.
    // Using a broad approach: if it's not alphanumeric, not whitespace, not ASCII,
    // and not a punctuation/symbol, it's likely a combining mark.
    // More precisely, we check the Unicode categories directly.
    matches!(
        unicode_general_category(ch),
        GeneralCategory::Mn | GeneralCategory::Mc | GeneralCategory::Me
    )
}

/// Minimal Unicode General Category detection for combining marks.
///
/// Only distinguishes Mark categories (Mn, Mc, Me) from everything else (Other).
/// This avoids pulling in a full Unicode data crate for a focused need.
#[derive(Debug, PartialEq, Eq)]
enum GeneralCategory {
    /// Nonspacing Mark (Mn)
    Mn,
    /// Spacing Combining Mark (Mc)
    Mc,
    /// Enclosing Mark (Me)
    Me,
    /// Any other category
    Other,
}

/// Returns the Unicode General Category for combining mark detection.
///
/// Covers the most common combining mark ranges needed for multilingual input:
/// Devanagari, Arabic, Bengali, Gurmukhi, Gujarati, Tamil, Telugu, Kannada,
/// Malayalam, Thai, and other Indic/Southeast Asian scripts.
fn unicode_general_category(ch: char) -> GeneralCategory {
    let cp = ch as u32;
    match cp {
        // Devanagari (U+0900–U+097F)
        0x0900..=0x0902
        | 0x093A
        | 0x093C
        | 0x0941..=0x0948
        | 0x094D
        | 0x0951..=0x0957
        | 0x0962..=0x0963 => GeneralCategory::Mn,
        0x0903 | 0x093B | 0x093E..=0x0940 | 0x0949..=0x094C | 0x094E..=0x094F | 0x0982..=0x0983 => {
            GeneralCategory::Mc
        }

        // Arabic combining marks (U+0610–U+065F, U+06D6–U+06ED, U+08D3–U+08FF)
        0x0610..=0x061A
        | 0x064B..=0x065F
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x06DF..=0x06E4
        | 0x06E7..=0x06E8
        | 0x06EA..=0x06ED
        | 0x08D3..=0x08FF => GeneralCategory::Mn,

        // Bengali (U+0980–U+09FF)
        0x09BC | 0x09C1..=0x09C4 | 0x09CD | 0x09E2..=0x09E3 => GeneralCategory::Mn,
        0x09BE..=0x09C0 | 0x09CB..=0x09CC | 0x09D7 => GeneralCategory::Mc,

        // Gurmukhi (U+0A00–U+0A7F)
        0x0A01..=0x0A02
        | 0x0A3C
        | 0x0A41..=0x0A42
        | 0x0A47..=0x0A48
        | 0x0A4B..=0x0A4D
        | 0x0A51
        | 0x0A70..=0x0A71
        | 0x0A75 => GeneralCategory::Mn,
        0x0A03 | 0x0A3E..=0x0A40 | 0x0A83 => GeneralCategory::Mc,

        // General combining marks (U+0300–U+036F: Combining Diacritical Marks)
        0x0300..=0x036F => GeneralCategory::Mn,

        // Combining Diacritical Marks Extended (U+1AB0–U+1AFF)
        0x1AB0..=0x1ACE => GeneralCategory::Mn,

        // Combining Diacritical Marks Supplement (U+1DC0–U+1DFF)
        0x1DC0..=0x1DFF => GeneralCategory::Mn,

        // Combining Half Marks (U+FE20–U+FE2F)
        0xFE20..=0xFE2F => GeneralCategory::Mn,

        // Enclosing marks (U+20DD–U+20E0, U+20E2–U+20E4)
        0x20DD..=0x20E0 | 0x20E2..=0x20E4 => GeneralCategory::Me,

        _ => GeneralCategory::Other,
    }
}

/// Token kinds in the calculator grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// A number (integer or decimal).
    Number(String),
    /// An identifier (variable name, currency code, etc.).
    Identifier(String),
    /// The plus operator.
    Plus,
    /// The minus operator.
    Minus,
    /// The multiplication operator.
    Star,
    /// The division operator.
    Slash,
    /// The power/exponent operator.
    Caret,
    /// The percent operator (e.g., `3%` means `0.03`).
    Percent,
    /// Left parenthesis.
    LeftParen,
    /// Right parenthesis.
    RightParen,
    /// A colon (for time).
    Colon,
    /// A comma.
    Comma,
    /// The "at" keyword for temporal context.
    At,
    /// The "as" keyword for unit conversion (e.g., "741 KB as MB").
    As,
    /// The "in" keyword for unit conversion (e.g., "19 TON in USD").
    In,
    /// The "to" keyword for unit conversion (e.g., "19 TON to USD").
    To,
    /// The "until" keyword for duration until a datetime.
    Until,
    /// The equals sign for equality checks (e.g., `1 + 1 = 2`).
    Equals,
    /// End of input.
    Eof,
}

/// A token with its position in the input.
#[derive(Debug, Clone)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// Start position in the input.
    pub start: usize,
    /// End position in the input.
    pub end: usize,
    /// The original text of the token.
    pub text: String,
}

impl Token {
    /// Creates a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, start: usize, end: usize, text: String) -> Self {
        Self {
            kind,
            start,
            end,
            text,
        }
    }

    /// Checks if this token is a number.
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(self.kind, TokenKind::Number(_))
    }

    /// Checks if this token is an identifier.
    #[must_use]
    pub const fn is_identifier(&self) -> bool {
        matches!(self.kind, TokenKind::Identifier(_))
    }
}

/// Lexer for tokenizing calculator input.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Creates a new lexer for the given input.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Tokenizes the entire input.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, CalculatorError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Returns the next token.
    pub fn next_token(&mut self) -> Result<Token, CalculatorError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token::new(
                TokenKind::Eof,
                self.pos,
                self.pos,
                String::new(),
            ));
        }

        let start = self.pos;
        let ch = self.current();

        // Single-character tokens
        let token = match ch {
            '+' => {
                self.advance();
                Token::new(TokenKind::Plus, start, self.pos, "+".to_string())
            }
            '-' => {
                self.advance();
                Token::new(TokenKind::Minus, start, self.pos, "-".to_string())
            }
            '*' => {
                self.advance();
                Token::new(TokenKind::Star, start, self.pos, "*".to_string())
            }
            '/' => {
                self.advance();
                Token::new(TokenKind::Slash, start, self.pos, "/".to_string())
            }
            '^' => {
                self.advance();
                Token::new(TokenKind::Caret, start, self.pos, "^".to_string())
            }
            '%' => {
                self.advance();
                Token::new(TokenKind::Percent, start, self.pos, "%".to_string())
            }
            '(' => {
                self.advance();
                Token::new(TokenKind::LeftParen, start, self.pos, "(".to_string())
            }
            ')' => {
                self.advance();
                Token::new(TokenKind::RightParen, start, self.pos, ")".to_string())
            }
            ':' => {
                self.advance();
                Token::new(TokenKind::Colon, start, self.pos, ":".to_string())
            }
            ',' => {
                self.advance();
                Token::new(TokenKind::Comma, start, self.pos, ",".to_string())
            }
            '=' => {
                self.advance();
                Token::new(TokenKind::Equals, start, self.pos, "=".to_string())
            }
            _ if ch.is_ascii_digit() || ch == '.' => self.scan_number(),
            _ if ch.is_alphabetic() => self.scan_identifier(),
            // Currency symbols used as prefix notation (e.g., $10, €5, £3)
            // These are recognized as single-character identifiers and mapped to ISO codes
            // by CurrencyDatabase::parse_currency().
            '$' | '€' | '£' | '¥' | '₽' | '₹' | '₩' | '₿' | '₫' => {
                self.advance();
                let symbol = ch.to_string();
                Token::new(
                    TokenKind::Identifier(symbol.clone()),
                    start,
                    self.pos,
                    symbol,
                )
            }
            _ => {
                return Err(CalculatorError::parse(format!(
                    "Unexpected character '{ch}' at position {start}"
                )));
            }
        };

        Ok(token)
    }

    fn scan_number(&mut self) -> Token {
        let start = self.pos;
        let mut text = String::new();
        let mut has_dot = false;

        while !self.is_at_end() {
            let ch = self.current();
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                // Check if next char is a digit (otherwise it might be something else)
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    has_dot = true;
                    text.push(ch);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Token::new(TokenKind::Number(text.clone()), start, self.pos, text)
    }

    fn scan_identifier(&mut self) -> Token {
        let start = self.pos;
        let mut text = String::new();

        while !self.is_at_end() {
            let ch = self.current();
            // Accept alphanumeric, underscore, and Unicode combining marks (Mn/Mc/Me).
            // Combining marks are needed for scripts like Devanagari (Hindi) where
            // virama (्, U+094D) and dependent vowels (ा, ि, etc.) are part of words
            // but not classified as alphabetic.
            if ch.is_alphanumeric() || ch == '_' || is_unicode_mark(ch) {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords (including multilingual equivalents)
        let kind = match text.to_lowercase().as_str() {
            "at" => TokenKind::At,
            "as" => TokenKind::As,
            "in" => TokenKind::In,
            "to" => TokenKind::To,
            "until" => TokenKind::Until,
            // Russian: "в" means "in/into" (e.g. "1000 рублей в долларах")
            "в" => TokenKind::In,
            // French: "en" means "in/into" (e.g. "1000 dollars en euros")
            // Note: German "in" is identical to English "in", no extra entry needed.
            "en" => TokenKind::In,
            // Chinese: conversion phrases (e.g. "1000美元换成欧元")
            "换成" | "兑换成" | "转换为" | "兑成" | "转为" => TokenKind::To,
            // Hindi: "में" means "in" as postposition (e.g. "1000 डॉलर में यूरो")
            "में" => TokenKind::In,
            // Arabic: "إلى" means "to/into" (e.g. "1000 دولار إلى يورو")
            "إلى" => TokenKind::To,
            _ => TokenKind::Identifier(text.clone()),
        };

        Token::new(kind, start, self.pos, text)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.current().is_whitespace() {
            self.advance();
        }
    }

    fn current(&self) -> char {
        self.input.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // number + eof
        assert!(matches!(tokens[0].kind, TokenKind::Number(ref s) if s == "42"));
    }

    #[test]
    fn test_tokenize_decimal() {
        let mut lexer = Lexer::new("3.14");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Number(ref s) if s == "3.14"));
    }

    #[test]
    fn test_tokenize_expression() {
        let mut lexer = Lexer::new("2 + 3");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4); // 2 + 3 eof
        assert!(matches!(tokens[1].kind, TokenKind::Plus));
    }

    #[test]
    fn test_tokenize_parentheses() {
        let mut lexer = Lexer::new("(2 + 3)");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::LeftParen));
        assert!(matches!(tokens[4].kind, TokenKind::RightParen));
    }

    #[test]
    fn test_tokenize_identifier() {
        let mut lexer = Lexer::new("USD EUR");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Identifier(ref s) if s == "USD"));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "EUR"));
    }

    #[test]
    fn test_tokenize_at_keyword() {
        let mut lexer = Lexer::new("value at time");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[1].kind, TokenKind::At));
    }

    #[test]
    fn test_tokenize_currency_expression() {
        let mut lexer = Lexer::new("84 USD - 34 EUR");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 6); // 84 USD - 34 EUR eof
    }

    #[test]
    fn test_tokenize_percent() {
        let mut lexer = Lexer::new("3%");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3); // 3 % eof
        assert!(matches!(tokens[0].kind, TokenKind::Number(ref s) if s == "3"));
        assert!(matches!(tokens[1].kind, TokenKind::Percent));
    }

    #[test]
    fn test_tokenize_percent_expression() {
        let mut lexer = Lexer::new("3% * 50");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5); // 3 % * 50 eof
        assert!(matches!(tokens[1].kind, TokenKind::Percent));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
    }
}
