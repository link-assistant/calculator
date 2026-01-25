//! Links notation support for expression representation.
//!
//! Links notation (lino) is a simple, intuitive format for representing
//! structured data as links between references to links.
//!
//! # Examples
//!
//! Basic expressions:
//! ```text
//! (2 + 3)           -> ((2) + (3))
//! 84 USD - 34 EUR   -> (((84 USD)) - ((34 EUR)))
//! ```
//!
//! Temporal expressions:
//! ```text
//! (84 USD - 34 EUR) at (22 Jan 2026)  -> ((((84 USD) - (34 EUR)) at (22 Jan 2026)))
//! ```

use crate::types::Expression;

/// Formats an expression into links notation.
#[must_use]
pub fn to_lino(expr: &Expression) -> String {
    expr.to_lino()
}

/// Represents a link in links notation.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// Optional identifier for this link.
    pub id: Option<String>,
    /// The references that make up this link.
    pub refs: Vec<LinkRef>,
}

/// A reference within a link.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkRef {
    /// A literal value (number, string, etc.).
    Literal(String),
    /// A reference to another link by its identifier.
    Ref(String),
    /// A nested link.
    Nested(Box<Link>),
}

impl Link {
    /// Creates a new link with the given references.
    #[must_use]
    pub const fn new(refs: Vec<LinkRef>) -> Self {
        Self { id: None, refs }
    }

    /// Creates a new link with an identifier.
    #[must_use]
    pub fn with_id(id: impl Into<String>, refs: Vec<LinkRef>) -> Self {
        Self {
            id: Some(id.into()),
            refs,
        }
    }

    /// Formats this link as a links notation string.
    #[must_use]
    pub fn to_lino(&self) -> String {
        let refs_str: Vec<String> = self.refs.iter().map(LinkRef::to_lino).collect();
        let content = refs_str.join(" ");

        match &self.id {
            Some(id) => format!("({id}: {content})"),
            None => {
                if self.refs.len() > 1 {
                    format!("({content})")
                } else {
                    content
                }
            }
        }
    }
}

impl LinkRef {
    /// Formats this reference as a links notation string.
    #[must_use]
    pub fn to_lino(&self) -> String {
        match self {
            Self::Literal(s) => s.clone(),
            Self::Ref(id) => id.clone(),
            Self::Nested(link) => link.to_lino(),
        }
    }
}

/// Parser for links notation.
#[derive(Debug, Default)]
pub struct LinoParser;

impl LinoParser {
    /// Creates a new links notation parser.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Parses links notation into a list of links.
    pub fn parse(&self, input: &str) -> Result<Vec<Link>, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut links = Vec::new();
        let mut pos = 0;
        let chars: Vec<char> = input.chars().collect();

        while pos < chars.len() {
            // Skip whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            if pos >= chars.len() {
                break;
            }

            let (link, new_pos) = self.parse_link(&chars, pos)?;
            links.push(link);
            pos = new_pos;
        }

        Ok(links)
    }

    fn parse_link(&self, chars: &[char], pos: usize) -> Result<(Link, usize), String> {
        if pos >= chars.len() {
            return Err("Unexpected end of input".to_string());
        }

        if chars[pos] == '(' {
            // Parenthesized link
            self.parse_parenthesized_link(chars, pos)
        } else {
            // Single token (could be identifier or literal)
            Self::parse_token_as_link(chars, pos)
        }
    }

    fn parse_parenthesized_link(
        &self,
        chars: &[char],
        pos: usize,
    ) -> Result<(Link, usize), String> {
        let mut pos = pos + 1; // Skip '('
        let mut refs = Vec::new();
        let mut id = None;

        while pos < chars.len() && chars[pos] != ')' {
            // Skip whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            if pos >= chars.len() || chars[pos] == ')' {
                break;
            }

            // Check for identifier with colon
            if id.is_none() && !refs.is_empty() {
                // Already have refs, can't set id now
            } else if let Some((token, new_pos)) = Self::try_parse_identifier_with_colon(chars, pos)
            {
                id = Some(token);
                pos = new_pos;
                continue;
            }

            // Parse a reference
            let (link_ref, new_pos) = self.parse_ref(chars, pos)?;
            refs.push(link_ref);
            pos = new_pos;
        }

        if pos >= chars.len() {
            return Err("Unclosed parenthesis".to_string());
        }

        pos += 1; // Skip ')'

        Ok((Link { id, refs }, pos))
    }

    fn parse_token_as_link(chars: &[char], pos: usize) -> Result<(Link, usize), String> {
        let (token, new_pos) = Self::parse_token(chars, pos)?;
        Ok((
            Link {
                id: None,
                refs: vec![LinkRef::Literal(token)],
            },
            new_pos,
        ))
    }

    fn parse_ref(&self, chars: &[char], pos: usize) -> Result<(LinkRef, usize), String> {
        if pos >= chars.len() {
            return Err("Unexpected end of input".to_string());
        }

        if chars[pos] == '(' {
            let (link, new_pos) = self.parse_parenthesized_link(chars, pos)?;
            Ok((LinkRef::Nested(Box::new(link)), new_pos))
        } else {
            let (token, new_pos) = Self::parse_token(chars, pos)?;
            Ok((LinkRef::Literal(token), new_pos))
        }
    }

    fn try_parse_identifier_with_colon(chars: &[char], pos: usize) -> Option<(String, usize)> {
        let (token, new_pos) = Self::parse_token(chars, pos).ok()?;

        // Skip whitespace
        let mut check_pos = new_pos;
        while check_pos < chars.len() && chars[check_pos].is_whitespace() {
            check_pos += 1;
        }

        if check_pos < chars.len() && chars[check_pos] == ':' {
            Some((token, check_pos + 1))
        } else {
            None
        }
    }

    fn parse_token(chars: &[char], pos: usize) -> Result<(String, usize), String> {
        let mut pos = pos;
        let mut token = String::new();

        // Skip leading whitespace
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        while pos < chars.len() {
            let ch = chars[pos];
            if ch.is_whitespace() || ch == '(' || ch == ')' || ch == ':' {
                break;
            }
            token.push(ch);
            pos += 1;
        }

        if token.is_empty() {
            Err(format!("Expected token at position {pos}"))
        } else {
            Ok((token, pos))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_to_lino() {
        let link = Link::new(vec![
            LinkRef::Literal("2".to_string()),
            LinkRef::Literal("+".to_string()),
            LinkRef::Literal("3".to_string()),
        ]);
        assert_eq!(link.to_lino(), "(2 + 3)");
    }

    #[test]
    fn test_link_with_id() {
        let link = Link::with_id(
            "sum",
            vec![
                LinkRef::Literal("2".to_string()),
                LinkRef::Literal("+".to_string()),
                LinkRef::Literal("3".to_string()),
            ],
        );
        assert_eq!(link.to_lino(), "(sum: 2 + 3)");
    }

    #[test]
    fn test_parse_simple() {
        let parser = LinoParser::new();
        let links = parser.parse("hello").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].refs.len(), 1);
    }

    #[test]
    fn test_parse_parenthesized() {
        let parser = LinoParser::new();
        let links = parser.parse("(2 + 3)").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].refs.len(), 3);
    }

    #[test]
    fn test_parse_nested() {
        let parser = LinoParser::new();
        let links = parser.parse("((2 + 3) * 4)").unwrap();
        assert_eq!(links.len(), 1);
    }
}
