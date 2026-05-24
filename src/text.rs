//! Small text helpers mirroring the JS implementation.

/// Lowercase, replace runs of non-alphanumeric chars with a single `-`, trim.
pub fn slugify(value: &str) -> String {
    let lower = value.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_dash = false;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Strip a leading `@scope/`, turn `._-` runs into spaces, collapse
/// whitespace and uppercase the first letter of each word.
pub fn humanize_name(value: &str) -> String {
    let mut s = value;
    if s.starts_with('@') {
        if let Some(idx) = s.find('/') {
            s = &s[idx + 1..];
        }
    }

    let mut spaced = String::with_capacity(s.len());
    let mut prev_sep = false;
    for ch in s.chars() {
        if ch == '.' || ch == '_' || ch == '-' {
            if !prev_sep {
                spaced.push(' ');
                prev_sep = true;
            }
        } else {
            spaced.push(ch);
            prev_sep = false;
        }
    }

    spaced
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Return the trimmed value when non-empty, otherwise the fallback.
pub fn safe_title(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Case-insensitive whole-word match (equivalent to a JS `\bword\b` test).
/// `word` must already be lowercase.
pub fn has_word(text_lower: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let bytes = text_lower.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text_lower[from..].find(word) {
        let start = from + rel;
        let end = start + word.len();
        let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
        let after_ok = end == bytes.len() || !is_word_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
        if from >= bytes.len() {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Acme Shop!"), "acme-shop");
        assert_eq!(slugify("@scope/My_Pkg"), "scope-my-pkg");
        assert_eq!(slugify("  --trim--  "), "trim");
    }

    #[test]
    fn humanize_strips_scope_and_caps() {
        assert_eq!(
            humanize_name("@wesleysimplicio/llm-project-mapper"),
            "Llm Project Mapper"
        );
        assert_eq!(humanize_name("acme-shop"), "Acme Shop");
        assert_eq!(humanize_name("qwen-0.5b"), "Qwen 0 5b");
    }

    #[test]
    fn word_boundary_match() {
        assert!(has_word("uses pg here", "pg"));
        assert!(has_word("\"pg\": \"^8\"", "pg"));
        assert!(!has_word("a postgres pkg", "pg")); // 'pg' inside 'pkg'? no; ensure no false hit
        assert!(!has_word("mypgx", "pg"));
        assert!(has_word("gh cli", "gh"));
    }

    #[test]
    fn safe_title_fallback() {
        assert_eq!(safe_title("  ", "fallback"), "fallback");
        assert_eq!(safe_title(" hi ", "fallback"), "hi");
    }
}
