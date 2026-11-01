pub(super) fn truncate_smart(s: &str, max_len: usize) -> String {
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
    if s.width() <= max_len {
        return s.to_string();
    }
    if max_len <= 3 {
        return "...".to_string();
    }

    let target = max_len - 3;
    let mut used = 0;
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        let w = ch.width().unwrap_or(0);
        if used + w > target {
            break;
        }
        used += w;
        end = idx + ch.len_utf8();
    }
    let prefix = &s[..end];

    if let Some(pos) = prefix.rfind(' ') {
        let before = &prefix[..pos];
        if before.width() > target / 2 {
            return format!("{}...", before);
        }
    }
    format!("{}...", prefix)
}

pub(super) fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

pub(super) fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    if max_chars == 1 {
        return "…".to_string();
    }
    let truncated = truncate_chars(s, max_chars.saturating_sub(1));
    format!("{}…", truncated)
}

#[cfg(test)]
mod cell_width_tests {
    use super::truncate_smart;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn truncate_smart_keeps_fitting_text() {
        assert_eq!(truncate_smart("hello", 5), "hello");
        assert_eq!(truncate_smart("日本", 4), "日本");
    }

    #[test]
    fn truncate_smart_counts_cjk_cells() {
        let out = truncate_smart("日本語のエージェント名", 10);
        assert!(out.width() <= 10, "{out}");
        assert!(out.ends_with("..."));
    }

    #[test]
    fn truncate_smart_counts_emoji_cells() {
        let out = truncate_smart("🐝🐝🐝🐝🐝🐝🐝🐝", 9);
        assert!(out.width() <= 9, "{out}");
        assert!(out.ends_with("..."));
    }
}
