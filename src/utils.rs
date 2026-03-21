//! Internal utility functions.

/// Generates a GitHub issue link for unrecognized input.
pub fn generate_issue_link(input: &str, error: &str) -> String {
    let title = format!("Unrecognized input: {}", truncate(input, 50));
    let body = format!(
        "## Input that failed to parse\n\n```\n{}\n```\n\n## Error message\n\n```\n{}\n```\n\n## Expected behavior\n\nPlease describe what you expected the calculator to do with this input.",
        input, error
    );
    format!(
        "https://github.com/link-assistant/calculator/issues/new?title={}&body={}",
        urlencoding_encode(&title),
        urlencoding_encode(&body)
    )
}

pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            '\n' => result.push_str("%0A"),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_link_generation() {
        let link = generate_issue_link("invalid input", "Parse error");
        assert!(link.contains("github.com"));
        assert!(link.contains("issues/new"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 5), "hello");
        assert_eq!(truncate("hi", 10), "hi");
    }
}
