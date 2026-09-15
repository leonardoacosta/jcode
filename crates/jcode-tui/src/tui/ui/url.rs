use regex::Regex;
use std::sync::OnceLock;
use unicode_width::UnicodeWidthStr;

pub(crate) fn url_regex() -> Option<&'static Regex> {
    static URL_REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    URL_REGEX
        .get_or_init(|| Regex::new(r#"(?i)(?:https?://|mailto:|file://)[^\s<>'\"]+"#).ok())
        .as_ref()
}

fn markdown_link_regex() -> Option<&'static Regex> {
    static MARKDOWN_LINK_REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    MARKDOWN_LINK_REGEX
        .get_or_init(|| Regex::new(r#"\[([^]\n]+)\]\(([^\s)]+)(?:\s+[^)]*)?\)"#).ok())
        .as_ref()
}

pub(crate) fn trim_url_candidate(candidate: &str) -> &str {
    let mut trimmed = candidate;
    loop {
        let next = if trimmed.ends_with(['.', ',', ';', ':', '!', '?'])
            || (trimmed.ends_with(')')
                && trimmed.matches(')').count() > trimmed.matches('(').count())
            || (trimmed.ends_with(']')
                && trimmed.matches(']').count() > trimmed.matches('[').count())
            || (trimmed.ends_with('}')
                && trimmed.matches('}').count() > trimmed.matches('{').count())
        {
            &trimmed[..trimmed.len() - 1]
        } else {
            trimmed
        };

        if next.len() == trimmed.len() {
            return trimmed;
        }
        trimmed = next;
    }
}

pub(crate) fn link_target_for_display_column(raw_text: &str, column: usize) -> Option<String> {
    if let Some(regex) = markdown_link_regex() {
        for captures in regex.captures_iter(raw_text) {
            let (Some(label), Some(target)) = (captures.get(1), captures.get(2)) else {
                continue;
            };
            let start_col = raw_text[..label.start()].width();
            let end_col = start_col + label.as_str().width();
            if column >= start_col && column < end_col {
                return Some(target.as_str().to_string());
            }
        }
    }

    for mat in url_regex()?.find_iter(raw_text) {
        let matched = &raw_text[mat.start()..mat.end()];
        let trimmed = trim_url_candidate(matched);
        if trimmed.is_empty() {
            continue;
        }

        let start_col = raw_text[..mat.start()].width();
        let end_col = start_col + trimmed.width();
        if column >= start_col && column < end_col && ::url::Url::parse(trimmed).is_ok() {
            return Some(trimmed.to_string());
        }
    }

    // Match whole tokens so malformed positions are rejected by the resolver,
    // not silently shortened to a valid line number. URLs take priority above.
    static SOURCE_REGEX: OnceLock<Option<Regex>> = OnceLock::new();
    let regex = SOURCE_REGEX.get_or_init(|| {
        Regex::new(r#"(?:^|[\s`(])([^\s`<>\[\]()]+(?:#L|:)[0-9][^\s`<>\[\]()]*)"#).ok()
    });
    if let Some(regex) = regex {
        for captures in regex.captures_iter(raw_text) {
            let target = captures.get(1)?;
            let candidate = target.as_str().trim_end_matches([',', '.', ';']);
            if candidate.contains("://") {
                continue;
            }
            let start = raw_text[..target.start()].width();
            if column >= start && column < start + candidate.width() {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{link_target_for_display_column, trim_url_candidate, url_regex};

    #[test]
    fn source_tokens_keep_full_locations_and_leave_urls_unchanged() {
        for (text, column, expected) in [
            ("`src/main.rs:43:1`", 3, "src/main.rs:43:1"),
            ("config.toml#L43", 13, "config.toml#L43"),
            ("guide (README.md#L43)", 10, "README.md#L43"),
            ("src/main.rs:43:bad", 4, "src/main.rs:43:bad"),
            ("https://host/main.rs:43", 9, "https://host/main.rs:43"),
            ("mailto:user@example.com", 9, "mailto:user@example.com"),
        ] {
            assert_eq!(
                link_target_for_display_column(text, column).as_deref(),
                Some(expected),
                "{text}"
            );
        }
    }

    #[test]
    fn url_regex_matches_supported_link_schemes() {
        let regex = url_regex();
        assert!(regex.is_some(), "test URL regex should initialize");
        let Some(regex) = regex else {
            return;
        };
        let text = "See https://example.com, mailto:user@example.com, and file:///tmp/a.txt";
        let matches: Vec<&str> = regex.find_iter(text).map(|mat| mat.as_str()).collect();

        assert_eq!(
            matches,
            vec![
                "https://example.com,",
                "mailto:user@example.com,",
                "file:///tmp/a.txt"
            ]
        );
    }

    #[test]
    fn trim_url_candidate_removes_trailing_sentence_punctuation() {
        assert_eq!(
            trim_url_candidate("https://example.com,"),
            "https://example.com"
        );
        assert_eq!(
            trim_url_candidate("https://example.com?!"),
            "https://example.com"
        );
        assert_eq!(
            trim_url_candidate("mailto:user@example.com."),
            "mailto:user@example.com"
        );
    }

    #[test]
    fn trim_url_candidate_preserves_balanced_closing_delimiters() {
        assert_eq!(
            trim_url_candidate("https://example.com/path_(draft)"),
            "https://example.com/path_(draft)"
        );
        assert_eq!(
            trim_url_candidate("https://example.com/path_(draft))."),
            "https://example.com/path_(draft)"
        );
        assert_eq!(
            trim_url_candidate("https://example.com/[docs]]"),
            "https://example.com/[docs]"
        );
    }

    #[test]
    fn link_target_for_display_column_returns_trimmed_url_when_inside_url() {
        let text = "Open https://example.com/docs, please";

        assert_eq!(
            link_target_for_display_column(text, "Open https://example".len()),
            Some("https://example.com/docs".to_string())
        );
        assert_eq!(
            link_target_for_display_column(text, "Open ".len() - 1),
            None
        );
        assert_eq!(
            link_target_for_display_column(text, "Open https://example.com/docs".len()),
            None
        );
    }

    #[test]
    fn link_target_for_display_column_uses_display_width_for_wide_prefixes() {
        let text = "🙂 https://example.com";

        assert_eq!(
            link_target_for_display_column(text, 3),
            Some("https://example.com".to_string())
        );
        assert_eq!(link_target_for_display_column(text, 1), None);
    }

    #[test]
    fn link_target_for_display_column_resolves_markdown_label() {
        let text = "Read the [guide](docs/guide.md#setup) today";

        assert_eq!(
            link_target_for_display_column(text, 10),
            Some("docs/guide.md#setup".to_string())
        );
        assert_eq!(link_target_for_display_column(text, 8), None);
    }
}
