//! Deterministic numeric conventions shared by every lexer entry point.
//!
//! A single comma before three digits groups thousands; otherwise it is a
//! decimal comma. With both punctuation marks the rightmost one is decimal.
//! The documented fend outlier `1,1` means 11 in the primary convention.
//! Function-call commas remain argument separators.

use crate::error::CalculatorError;

pub(super) fn scan(
    input: &[char],
    start: usize,
    allow_comma: bool,
    decimal_comma: bool,
) -> Result<(String, usize), CalculatorError> {
    if let Some(number) = scan_radix(input, start)? {
        return Ok(number);
    }
    let mut end = start;
    let mut first_group_digits = 0;
    let mut has_group_space = false;
    let mut can_group_space = true;
    while let Some(&ch) = input.get(end) {
        if ch.is_ascii_digit() || ch == '_' {
            if ch == '_' {
                first_group_digits = 0;
            }
            if ch.is_ascii_digit() && !has_group_space {
                first_group_digits += 1;
            }
        } else if (matches!(ch, '.' | '\'' | '’') || (ch == ',' && allow_comma) || is_space(ch))
            && input.get(end + 1).is_some_and(char::is_ascii_digit)
        {
            if is_space(ch) {
                // Bound lookahead to four digits. Invalid groups leave a token
                // boundary, also preserving date parts such as `Jan 17 2027`.
                let digits = input[end + 1..]
                    .iter()
                    .take(4)
                    .take_while(|ch| ch.is_ascii_digit())
                    .count();
                if !can_group_space || first_group_digits > 3 || digits != 3 {
                    break;
                }
                has_group_space = true;
            } else {
                can_group_space = ch == '.';
                first_group_digits = 0;
                has_group_space = false;
            }
        } else {
            break;
        }
        end += 1;
    }
    let candidate: String = input[start..end].iter().collect();
    let normalized = normalize(&candidate, decimal_comma).ok_or_else(|| {
        CalculatorError::parse(format!(
            "Invalid number separators in '{candidate}' at position {start}"
        ))
    })?;
    Ok((normalized, end))
}

fn is_space(ch: char) -> bool {
    matches!(ch, ' ' | '\u{00a0}' | '\u{2007}' | '\u{2009}' | '\u{202f}')
}

fn normalize(candidate: &str, decimal_comma: bool) -> Option<String> {
    // Underscores may separate digits, including fractional digits, but cannot
    // hide malformed punctuation or occur consecutively.
    let chars: Vec<char> = candidate.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '_'
            && (i == 0
                || !chars[i - 1].is_ascii_digit()
                || !chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            return None;
        }
    }
    // Space grouping may also continue an underscore-grouped integer.
    let number = if candidate.chars().any(is_space) && !candidate.contains(['.', ',']) {
        candidate.replace('_', " ")
    } else {
        candidate.replace('_', "")
    };
    // Issue #233 explicitly requires this fend compatibility outlier alongside
    // decimal commas. Keep it narrow; `1,2` and `12,3` remain decimal fractions.
    if number == "1,1" && !decimal_comma {
        return Some("11".to_string());
    }
    let commas = number.matches(',').count();
    let dots = number.matches('.').count();
    let decimal = match (commas, dots) {
        (0, 0) => None,
        (0, 1) => Some('.'),
        (1, 0) if decimal_comma || !valid_groups(&number, ',') => Some(','),
        (_, 0) => None,
        (0, _) => return None,
        (_, _) if number.rfind(',') > number.rfind('.') => Some(','),
        _ => Some('.'),
    };
    let (integer, fraction) = if let Some(decimal) = decimal {
        let (integer, fraction) = number.rsplit_once(decimal)?;
        if integer.contains(decimal)
            || fraction.is_empty()
            || !(fraction.chars().all(|ch| ch.is_ascii_digit())
                || fraction.chars().any(is_space) && valid_groups(fraction, ' '))
        {
            return None;
        }
        (integer, Some(fraction))
    } else {
        (number.as_str(), None)
    };
    let grouping: Vec<char> = integer.chars().filter(|ch| !ch.is_ascii_digit()).collect();
    let integer = if let Some(&separator) = grouping.first() {
        if grouping.iter().any(|&ch| !same_group(ch, separator))
            || !valid_groups(integer, separator)
        {
            return None;
        }
        integer
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>()
    } else if integer.is_empty() {
        // Preserve the conventional leading-dot decimal spelling `.5`.
        if decimal != Some('.') {
            return None;
        }
        "0".to_string()
    } else {
        integer.to_string()
    };
    Some(fraction.map_or_else(
        || integer.clone(),
        |fraction| {
            format!(
                "{integer}.{}",
                fraction
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
            )
        },
    ))
}

fn same_group(left: char, right: char) -> bool {
    left == right
        || (is_space(left) && is_space(right))
        || (matches!(left, '\'' | '’') && matches!(right, '\'' | '’'))
}

fn valid_groups(number: &str, separator: char) -> bool {
    let groups: Vec<&str> = number.split(|ch| same_group(ch, separator)).collect();
    let Some(first) = groups.first() else {
        return false;
    };
    if first.is_empty()
        || first.len() > 3
        || groups
            .iter()
            .any(|part| !part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return false;
    }
    let western = groups.iter().skip(1).all(|part| part.len() == 3);
    let indian = separator == ','
        && groups.len() >= 3
        && first.len() <= 2
        && groups.last().is_some_and(|part| part.len() == 3)
        && groups[1..groups.len() - 1]
            .iter()
            .all(|part| part.len() == 2);
    western || indian
}

/// Parse radix fractions exactly within the decimal grammar's 28-place limit.
/// Accumulation and expansion are bounded, and overflow is recoverable.
fn scan_radix(input: &[char], start: usize) -> Result<Option<(String, usize)>, CalculatorError> {
    if input.get(start) != Some(&'0') {
        return Ok(None);
    }
    let radix = match input.get(start + 1) {
        Some('b' | 'B') => 2,
        Some('o' | 'O') => 8,
        Some('x' | 'X') => 16,
        _ => return Ok(None),
    };
    if !input.get(start + 2).is_some_and(|ch| ch.is_digit(radix)) {
        return Ok(None);
    }
    let mut end = start + 2;
    let mut numerator = 0u128;
    let mut denominator = 1u128;
    let mut fraction = false;
    while let Some(&ch) = input.get(end) {
        if let Some(digit) = ch.to_digit(radix) {
            numerator = numerator
                .checked_mul(u128::from(radix))
                .and_then(|n| n.checked_add(u128::from(digit)))
                .ok_or(CalculatorError::Overflow)?;
            if fraction {
                denominator = denominator
                    .checked_mul(u128::from(radix))
                    .ok_or(CalculatorError::Overflow)?;
            }
        } else if ch == '_'
            && input[end - 1].is_digit(radix)
            && input.get(end + 1).is_some_and(|next| next.is_digit(radix))
        {
            // Digit separator, preserving the existing underscore convention.
        } else if ch == '.' && !fraction {
            fraction = true;
        } else {
            break;
        }
        end += 1;
    }
    if matches!(input.get(end), Some('e' | 'E')) {
        let (negative, exponent, exponent_end) = scan_radix_exponent(input, end + 1, radix)?;
        end = exponent_end;
        let factor = u128::from(radix)
            .checked_pow(exponent)
            .ok_or(CalculatorError::Overflow)?;
        if negative {
            denominator = denominator
                .checked_mul(factor)
                .ok_or(CalculatorError::Overflow)?;
        } else {
            numerator = numerator
                .checked_mul(factor)
                .ok_or(CalculatorError::Overflow)?;
        }
    }
    let mut number = (numerator / denominator).to_string();
    let mut remainder = numerator % denominator;
    if remainder != 0 {
        number.push('.');
        for _ in 0..28 {
            remainder = remainder.checked_mul(10).ok_or(CalculatorError::Overflow)?;
            let digit =
                u32::try_from(remainder / denominator).map_err(|_| CalculatorError::Overflow)?;
            number.push(char::from_digit(digit, 10).ok_or(CalculatorError::Overflow)?);
            remainder %= denominator;
            if remainder == 0 {
                break;
            }
        }
        if remainder != 0 {
            return Err(CalculatorError::Overflow);
        }
    }
    Ok(Some((number, end)))
}

/// fend uses exponent digits and powers in the literal's radix, e.g. `0b1e10 = 4`.
fn scan_radix_exponent(
    input: &[char],
    mut end: usize,
    radix: u32,
) -> Result<(bool, u32, usize), CalculatorError> {
    let negative = input.get(end) == Some(&'-');
    if matches!(input.get(end), Some('+' | '-')) {
        end += 1;
    }
    if !input.get(end).is_some_and(|ch| ch.is_digit(radix)) {
        return Err(CalculatorError::parse("Invalid radix exponent"));
    }
    let mut exponent = 0u32;
    while let Some(&ch) = input.get(end) {
        if let Some(digit) = ch.to_digit(radix) {
            exponent = exponent
                .checked_mul(radix)
                .and_then(|n| n.checked_add(digit))
                .ok_or(CalculatorError::Overflow)?;
        } else if ch != '_'
            || !input[end - 1].is_digit(radix)
            || !input.get(end + 1).is_some_and(|ch| ch.is_digit(radix))
        {
            break;
        }
        end += 1;
    }
    Ok((negative, exponent, end))
}
