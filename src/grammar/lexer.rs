//! Lexer for tokenizing calculator input.

use crate::error::CalculatorError;

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
            _ if ch.is_ascii_digit() || ch == '.' => self.scan_number(),
            _ if ch.is_alphabetic() => self.scan_identifier(),
            // Currency symbols used as prefix notation (e.g., $10, €5, £3)
            // These are recognized as single-character identifiers and mapped to ISO codes
            // by CurrencyDatabase::parse_currency().
            '$' | '€' | '£' | '¥' | '₽' | '₹' | '₩' | '₿' => {
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
            if ch.is_alphanumeric() || ch == '_' {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        let kind = match text.to_lowercase().as_str() {
            "at" => TokenKind::At,
            "as" => TokenKind::As,
            "in" => TokenKind::In,
            "to" => TokenKind::To,
            "until" => TokenKind::Until,
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
