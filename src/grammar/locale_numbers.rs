//! Locale-aware number normalization helpers.
//!
//! The grammar itself uses `.` as the decimal separator. These helpers build
//! normalized expression variants for user input that uses decimal/grouping
//! conventions from supported UI locales, such as `82,6172` or `1.234,56`.

#[derive(Debug, Clone, Copy)]
struct NumberLocale {
    decimal_separator: char,
    grouping_separator: Option<char>,
}

const LOCALES: &[NumberLocale] = &[
    // Russian, German, French, and other comma-decimal inputs without grouping.
    NumberLocale {
        decimal_separator: ',',
        grouping_separator: None,
    },
    // German/Russian style: 1.234,56 -> 1234.56.
    NumberLocale {
        decimal_separator: ',',
        grouping_separator: Some('.'),
    },
    // English/Hindi/Chinese style: 1,234.56 -> 1234.56.
    NumberLocale {
        decimal_separator: '.',
        grouping_separator: Some(','),
    },
];

/// Returns normalized variants of `input` using supported locale number
/// conventions. Variants are ordered by locale preference and de-duplicated.
pub(super) fn variants(input: &str) -> Vec<String> {
    let mut variants = Vec::new();

    for locale in LOCALES {
        if let Some(variant) = rewrite_with_locale(input, *locale) {
            push_unique(&mut variants, variant);
        }
    }

    variants
}

fn rewrite_with_locale(input: &str, locale: NumberLocale) -> Option<String> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    let mut changed = false;

    while let Some((start, ch)) = chars.next() {
        if !ch.is_ascii_digit() {
            output.push(ch);
            continue;
        }

        let mut end = start + ch.len_utf8();
        while let Some(&(idx, next)) = chars.peek() {
            if next.is_ascii_digit() || next == '.' || next == ',' {
                chars.next();
                end = idx + next.len_utf8();
            } else {
                break;
            }
        }

        let candidate = &input[start..end];
        if let Some(normalized) = normalize_number(candidate, locale) {
            if normalized != candidate {
                changed = true;
            }
            output.push_str(&normalized);
        } else {
            output.push_str(candidate);
        }
    }

    changed.then_some(output)
}

fn normalize_number(candidate: &str, locale: NumberLocale) -> Option<String> {
    if !candidate
        .chars()
        .any(|ch| ch == locale.decimal_separator || Some(ch) == locale.grouping_separator)
    {
        return None;
    }

    let decimal_count = candidate
        .chars()
        .filter(|ch| *ch == locale.decimal_separator)
        .count();

    if decimal_count > 1 {
        return normalize_grouped_integer(candidate, locale);
    }

    if decimal_count == 1 {
        return normalize_decimal(candidate, locale);
    }

    normalize_grouped_integer(candidate, locale)
}

fn normalize_decimal(candidate: &str, locale: NumberLocale) -> Option<String> {
    let (integer, fraction) = candidate.split_once(locale.decimal_separator)?;
    if integer.is_empty() || fraction.is_empty() || !fraction.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let integer = normalize_integer_part(integer, locale)?;
    Some(format!("{integer}.{fraction}"))
}

fn normalize_grouped_integer(candidate: &str, locale: NumberLocale) -> Option<String> {
    let group = locale.grouping_separator?;
    if !candidate.contains(group) {
        return None;
    }

    normalize_integer_part(candidate, locale)
}

fn normalize_integer_part(integer: &str, locale: NumberLocale) -> Option<String> {
    if integer.is_empty() {
        return None;
    }

    let group = locale.grouping_separator;
    if let Some(group) = group {
        if integer.contains(group) {
            let groups: Vec<&str> = integer.split(group).collect();
            let first = groups.first()?;
            if first.is_empty() || first.len() > 3 || !first.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }

            if groups
                .iter()
                .skip(1)
                .any(|part| part.len() != 3 || !part.chars().all(|ch| ch.is_ascii_digit()))
            {
                return None;
            }

            return Some(groups.join(""));
        }
    }

    integer.chars().all(|ch| ch.is_ascii_digit()).then(|| {
        integer
            .chars()
            .filter(|ch| Some(*ch) != group)
            .collect::<String>()
    })
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::variants;

    #[test]
    fn normalizes_decimal_comma() {
        assert_eq!(variants("82,6172 / 100"), vec!["82.6172 / 100"]);
    }

    #[test]
    fn exposes_ambiguous_comma_interpretations() {
        assert_eq!(variants("1,234 / 100"), vec!["1.234 / 100", "1234 / 100"]);
    }

    #[test]
    fn normalizes_grouped_decimal_forms() {
        assert_eq!(variants("1.234,56"), vec!["1234.56"]);
        assert_eq!(variants("1,234.56"), vec!["1234.56"]);
    }

    #[test]
    fn ignores_argument_separator_with_spaces() {
        assert!(variants("integrate(x^2, x, 0, 3)").is_empty());
    }
}
