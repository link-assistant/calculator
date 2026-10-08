//! Spelled numbers and arithmetic operator words as an optional front end
//! (issue #222).
//!
//! The vocabulary lives in `data/words/arithmetic-words.lino` (one link per
//! surface form), so adding a language means editing data, not code. The lexer
//! calls [`scan`] at the start of every word: when the input there is a number
//! word ("twenty one", "сто двадцать", "二十三") or an operator phrase
//! ("multiplied by", "умножить на", "除以"), it receives ready-made `Number` or
//! operator tokens; otherwise it lexes an ordinary identifier.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::lexer::{is_word_char, TokenKind};
use crate::types::CurrencyDatabase;

/// The bundled vocabulary.
const ARITHMETIC_WORDS_LINO: &str = include_str!("../../data/words/arithmetic-words.lino");

/// What a single vocabulary word means.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Meaning {
    /// A cardinal number ("seven", "сорок").
    Number(u128),
    /// A multiplier of what precedes it ("hundred", "тысяча", "十").
    Scale(u128),
    /// An arithmetic operator.
    Operator(TokenKind),
}

#[derive(Debug, Clone)]
struct Word {
    lang: String,
    meaning: Meaning,
}

/// The parsed vocabulary.
#[derive(Debug, Default)]
struct Vocabulary {
    /// Single words (lowercase) to their meaning.
    words: HashMap<String, Word>,
    /// Multi-word operator phrases (lowercase words), longest first.
    phrases: Vec<(Vec<String>, TokenKind)>,
    /// Length in characters of the longest single word, for CJK segmentation.
    longest_word: usize,
}

fn vocabulary() -> &'static Vocabulary {
    static VOCABULARY: OnceLock<Vocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(|| {
        parse_vocabulary(ARITHMETIC_WORDS_LINO).expect("data/words/arithmetic-words.lino is valid")
    })
}

fn operator_kind(symbol: &str) -> Option<TokenKind> {
    Some(match symbol {
        "+" => TokenKind::Plus,
        "-" => TokenKind::Minus,
        "*" => TokenKind::Star,
        "/" => TokenKind::Slash,
        "%" => TokenKind::Percent,
        "^" => TokenKind::Caret,
        "percent" => TokenKind::PercentWord,
        "percent-prefix" => TokenKind::PercentPrefix,
        "portion" => TokenKind::Portion,
        "increase-from" => TokenKind::IncreaseFrom,
        "decrease-from" => TokenKind::DecreaseFrom,
        _ => return None,
    })
}

/// Parses lines of the form `(kind lang value 'surface form')`.
fn parse_vocabulary(content: &str) -> Result<Vocabulary, String> {
    let mut vocabulary = Vocabulary::default();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let error = || {
            format!(
                "arithmetic-words.lino:{}: malformed link {line:?}",
                index + 1
            )
        };
        let body = line
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
            .ok_or_else(error)?;
        let (head, surface) = body.split_once('\'').ok_or_else(error)?;
        let surface = surface.strip_suffix('\'').ok_or_else(error)?.to_lowercase();
        let mut fields = head.split_whitespace();
        let (Some(kind), Some(lang), Some(value), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(error());
        };
        let meaning = match kind {
            "number" => Meaning::Number(value.parse().map_err(|_| error())?),
            "scale" => Meaning::Scale(value.parse().map_err(|_| error())?),
            "operator" => Meaning::Operator(operator_kind(value).ok_or_else(error)?),
            _ => return Err(error()),
        };

        let words: Vec<String> = surface.split_whitespace().map(str::to_string).collect();
        match (words.len(), meaning) {
            (0, _) => return Err(error()),
            (1, meaning) => {
                vocabulary.longest_word = vocabulary.longest_word.max(surface.chars().count());
                let word = Word {
                    lang: lang.to_string(),
                    meaning,
                };
                if let Some(previous) = vocabulary.words.insert(surface.clone(), word) {
                    if previous.meaning != vocabulary.words[&surface].meaning {
                        return Err(format!("{}: {surface:?} has two meanings", error()));
                    }
                }
            }
            (_, Meaning::Operator(op)) => vocabulary.phrases.push((words, op)),
            _ => return Err(format!("{}: only operators may span words", error())),
        }
    }
    vocabulary
        .phrases
        .sort_by_key(|(words, _)| std::cmp::Reverse(words.len()));
    Ok(vocabulary)
}

/// A token produced from words: kind, start, end (char positions) and text.
pub(super) type WordToken = (TokenKind, usize, usize, String);

/// Recognizes number words or an operator phrase starting at `pos`.
///
/// Returns the tokens (at least one) in input order, or `None` when the word
/// at `pos` is not arithmetic vocabulary.
pub(super) fn scan(input: &[char], pos: usize) -> Option<Vec<WordToken>> {
    let longest = vocabulary().longest_word.min(input.len() - pos);
    for len in (1..=longest).rev() {
        if let Some(Word {
            meaning: Meaning::Operator(op),
            ..
        }) = lookup(input, pos, pos + len)
        {
            if matches!(op, TokenKind::Portion | TokenKind::PercentPrefix)
                && (*op != TokenKind::Portion || portion_has_percent(input, pos + len))
                && (pos + len == input.len() || !input[pos + len].is_ascii_alphabetic())
            {
                return Some(vec![(
                    op.clone(),
                    pos,
                    pos + len,
                    text(input, pos, pos + len),
                )]);
            }
        }
    }
    if is_cjk(input[pos]) {
        return scan_cjk(input, pos);
    }
    if let Some(token) = scan_phrase(input, pos) {
        return Some(vec![token]);
    }
    scan_spaced_number(input, pos).map(|token| vec![token])
}

// A possession word also occurs in dates, units, and powers. Recognize the
// whole-before-part form only when the following text contains percent vocabulary.
fn portion_has_percent(input: &[char], start: usize) -> bool {
    for pos in start..input.len() {
        if input[pos] == '%' {
            return true;
        }
        let longest = vocabulary().longest_word.min(input.len() - pos);
        for len in (1..=longest).rev() {
            if matches!(
                lookup(input, pos, pos + len),
                Some(Word {
                    meaning: Meaning::Operator(TokenKind::PercentPrefix | TokenKind::PercentWord),
                    ..
                })
            ) {
                return true;
            }
        }
        if matches!(input[pos], '+' | '-' | '*' | '/' | '(' | ')') {
            break;
        }
    }
    false
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF)
}

/// End of the word that starts at `pos`.
fn word_end(input: &[char], pos: usize) -> usize {
    let mut end = pos;
    while end < input.len() && is_word_char(input[end]) {
        end += 1;
    }
    end
}

fn text(input: &[char], start: usize, end: usize) -> String {
    input[start..end].iter().collect()
}

/// An all-uppercase word that names a known currency ("DOS", "MIL", "ONE")
/// keeps its currency meaning, like "SEK" does over German "sek" seconds.
fn is_currency_code(word: &str) -> bool {
    word.chars().count() >= 2
        && word.chars().all(char::is_uppercase)
        && CurrencyDatabase::parse_currency(word).is_some()
}

/// Looks up the word spanning `start..end`, skipping currency codes.
fn lookup(input: &[char], start: usize, end: usize) -> Option<&'static Word> {
    let word = text(input, start, end);
    if start == end || is_currency_code(&word) {
        return None;
    }
    vocabulary().words.get(&word.to_lowercase())
}

fn skip_whitespace(input: &[char], mut pos: usize) -> usize {
    while pos < input.len() && input[pos].is_whitespace() {
        pos += 1;
    }
    pos
}

/// Matches the longest operator phrase (or single operator word) at `pos`.
fn scan_phrase(input: &[char], pos: usize) -> Option<WordToken> {
    let first_end = word_end(input, pos);
    let first = text(input, pos, first_end);
    if is_currency_code(&first) {
        return None;
    }
    let first = first.to_lowercase();

    let mut matched = vocabulary().phrases.iter().find_map(|(words, op)| {
        if words[0] != first {
            return None;
        }
        let mut end = first_end;
        for word in &words[1..] {
            let start = skip_whitespace(input, end);
            let next_end = word_end(input, start);
            if start == end || text(input, start, next_end).to_lowercase() != *word {
                return None;
            }
            end = next_end;
        }
        Some((op.clone(), end))
    });
    if matched.is_none() {
        if let Some(Word {
            meaning: Meaning::Operator(op),
            ..
        }) = vocabulary().words.get(&first)
        {
            matched = Some((op.clone(), first_end));
        }
    }

    let (op, end) = matched?;
    // `mod(17, 5)` is the function, not the operator.
    if matches!(op, TokenKind::Percent | TokenKind::PercentWord) && input.get(end) == Some(&'(') {
        return None;
    }
    Some((op, pos, end, text(input, pos, end)))
}

/// Combines consecutive number words into one value.
#[derive(Debug, Default)]
struct Composer {
    total: u128,
    current: u128,
    /// The last number word added since the last scale word.
    last: Option<u128>,
    any: bool,
    lang: Option<String>,
}

impl Composer {
    /// Adds a word if it continues the number; `false` ends the number.
    fn accept(&mut self, word: &Word) -> bool {
        if self.lang.as_ref().is_some_and(|lang| *lang != word.lang) {
            return false;
        }
        let accepted = match word.meaning {
            Meaning::Number(n) => self.add(n),
            Meaning::Scale(s) => self.scale(s),
            Meaning::Operator(_) => false,
        };
        if accepted {
            self.any = true;
            self.lang = Some(word.lang.clone());
        }
        accepted
    }

    fn add(&mut self, n: u128) -> bool {
        // "twenty one", "сто двадцать" and "two hundred five" continue a
        // number; "two three" or "ten five" do not.
        let continues = match self.last {
            None => true,
            Some(last) => last >= 20 && n < place_value(last),
        };
        if !continues || (self.any && n == 0) {
            return false;
        }
        match self.current.checked_add(n) {
            Some(current) => {
                self.current = current;
                self.last = Some(n);
                true
            }
            None => false,
        }
    }

    fn scale(&mut self, s: u128) -> bool {
        if s >= 1000 {
            // "thousand", "million", "万": close the group below it.
            if self.any && self.current == 0 {
                return false;
            }
            let Some(total) = self
                .current
                .max(1)
                .checked_mul(s)
                .and_then(|group| self.total.checked_add(group))
            else {
                return false;
            };
            self.total = total;
            self.current = 0;
        } else {
            // "hundred", "十", "百" multiply only the preceding digit, so that
            // 三百二十 is 3·100 + 2·10.
            if self.any && self.last.is_none() {
                return false;
            }
            let digit = self.last.unwrap_or(0);
            let Some(current) = digit
                .max(1)
                .checked_mul(s)
                .and_then(|scaled| (self.current - digit).checked_add(scaled))
            else {
                return false;
            };
            self.current = current;
        }
        self.last = None;
        true
    }

    fn value(&self) -> u128 {
        self.total + self.current
    }
}

/// The place value of the lowest non-zero digit (20 → 10, 100 → 100).
fn place_value(mut n: u128) -> u128 {
    let mut place = 1;
    while n > 0 && n % 10 == 0 {
        n /= 10;
        place *= 10;
    }
    place
}

/// Scans space-separated number words ("one hundred twenty-three").
fn scan_spaced_number(input: &[char], pos: usize) -> Option<WordToken> {
    let mut composer = Composer::default();
    let mut end = None;
    let mut start = pos;
    loop {
        let word_stop = word_end(input, start);
        match lookup(input, start, word_stop) {
            Some(word) if composer.accept(word) => end = Some(word_stop),
            _ => break,
        }
        // "twenty-one": a hyphen directly between two number words.
        let next = if input.get(word_stop) == Some(&'-')
            && input.get(word_stop + 1).is_some_and(|c| is_word_char(*c))
        {
            word_stop + 1
        } else {
            skip_whitespace(input, word_stop)
        };
        if next == word_stop || next >= input.len() || !is_word_char(input[next]) {
            break;
        }
        start = next;
    }
    let end = end?;
    Some((
        TokenKind::Number(composer.value().to_string()),
        pos,
        end,
        text(input, pos, end),
    ))
}

/// Segments an unspaced run of CJK characters ("二十三加五") into vocabulary
/// words. The run must consist of vocabulary only, so that words like 千克
/// (kilogram) or 十二月 (December) keep their own meaning.
fn scan_cjk(input: &[char], pos: usize) -> Option<Vec<WordToken>> {
    let mut run_end = pos;
    while run_end < input.len() && is_cjk(input[run_end]) {
        run_end += 1;
    }

    let mut segments = Vec::new();
    let mut start = pos;
    while start < run_end {
        let longest = vocabulary().longest_word.min(run_end - start);
        let (end, word) = (1..=longest)
            .rev()
            .find_map(|len| lookup(input, start, start + len).map(|word| (start + len, word)))?;
        segments.push((start, end, word));
        start = end;
    }

    let mut tokens = Vec::new();
    let mut number: Option<(usize, usize, Composer)> = None;
    for (start, end, word) in segments {
        if let Meaning::Operator(op) = &word.meaning {
            flush_number(&mut tokens, &mut number, input);
            tokens.push((op.clone(), start, end, text(input, start, end)));
            continue;
        }
        if let Some((_, number_end, composer)) = number.as_mut() {
            if composer.accept(word) {
                *number_end = end;
                continue;
            }
            flush_number(&mut tokens, &mut number, input);
        }
        let mut composer = Composer::default();
        if !composer.accept(word) {
            return None;
        }
        number = Some((start, end, composer));
    }
    flush_number(&mut tokens, &mut number, input);
    Some(tokens)
}

fn flush_number(
    tokens: &mut Vec<WordToken>,
    number: &mut Option<(usize, usize, Composer)>,
    input: &[char],
) {
    if let Some((start, end, composer)) = number.take() {
        tokens.push((
            TokenKind::Number(composer.value().to_string()),
            start,
            end,
            text(input, start, end),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Option<Vec<TokenKind>> {
        let chars: Vec<char> = input.chars().collect();
        scan(&chars, 0).map(|tokens| tokens.into_iter().map(|t| t.0).collect())
    }

    fn number(input: &str) -> Option<String> {
        match kinds(input)?.as_slice() {
            [TokenKind::Number(n)] => Some(n.clone()),
            _ => None,
        }
    }

    #[test]
    fn test_vocabulary_parses() {
        let vocabulary = vocabulary();
        assert!(vocabulary.words.len() > 150);
        assert!(vocabulary
            .phrases
            .iter()
            .any(|(w, _)| w == &["multiplied", "by"]));
    }

    #[test]
    fn test_malformed_link_is_rejected() {
        assert!(parse_vocabulary("(number en x 'one')").is_err());
        assert!(parse_vocabulary("(number en 1 'one two')").is_err());
        assert!(parse_vocabulary("(number en 1 'one')\n(number es 2 'one')").is_err());
    }

    #[test]
    fn test_compound_numbers() {
        assert_eq!(number("seven").as_deref(), Some("7"));
        assert_eq!(number("twenty one").as_deref(), Some("21"));
        assert_eq!(number("twenty-one").as_deref(), Some("21"));
        assert_eq!(number("one hundred twenty three").as_deref(), Some("123"));
        assert_eq!(number("two thousand twenty six").as_deref(), Some("2026"));
        assert_eq!(number("сто двадцать три").as_deref(), Some("123"));
        assert_eq!(number("две тысячи").as_deref(), Some("2000"));
        assert_eq!(number("दो सौ").as_deref(), Some("200"));
        assert_eq!(number("二十三").as_deref(), Some("23"));
        assert_eq!(number("三百二十").as_deref(), Some("320"));
        assert_eq!(number("十五").as_deref(), Some("15"));
        assert_eq!(number("一万二千").as_deref(), Some("12000"));
    }

    #[test]
    fn test_number_words_do_not_run_together() {
        // "two three" is two numbers, so only "two" is consumed.
        let chars: Vec<char> = "two three".chars().collect();
        let tokens = scan(&chars, 0).unwrap();
        assert_eq!(
            tokens,
            vec![(TokenKind::Number("2".into()), 0, 3, "two".into())]
        );
        // A hyphen between non-combining words stays a minus sign.
        let chars: Vec<char> = "five-two".chars().collect();
        assert_eq!(scan(&chars, 0).unwrap()[0].2, 4);
    }

    #[test]
    fn test_operator_phrases() {
        assert_eq!(kinds("multiplied by 3"), Some(vec![TokenKind::Star]));
        assert_eq!(kinds("умножить на 7"), Some(vec![TokenKind::Star]));
        assert_eq!(kinds("по модулю 3"), Some(vec![TokenKind::Percent]));
        assert_eq!(kinds("dividido por 2"), Some(vec![TokenKind::Slash]));
        assert_eq!(kinds("mod(17, 5)"), None);
        assert_eq!(kinds("по мск"), None);
    }

    #[test]
    fn test_cjk_runs() {
        assert_eq!(
            kinds("二加二"),
            Some(vec![
                TokenKind::Number("2".into()),
                TokenKind::Plus,
                TokenKind::Number("2".into())
            ])
        );
        assert_eq!(kinds("千克"), None);
        assert_eq!(kinds("十二月"), None);
    }

    #[test]
    fn test_uppercase_currency_codes_are_not_words() {
        assert_eq!(kinds("ONE"), None);
        assert_eq!(number("one").as_deref(), Some("1"));
    }
}
