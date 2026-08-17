//! User-message framing (Zentui port, roadmap P2): transcript prompt frames.
//!
//! Five styles (`display.user_messages.style`):
//! - `framed` (default): full-width top/bottom border rows around each user
//!   prompt plus an accent rail on every prompt row.
//! - `framed-copy-friendly`: identical borders and band, a one-cell leading
//!   gutter instead of rail glyphs.
//! - `compact`: accent rail only, zero added height.
//! - `labeled`: rounded box with a fixed `User` label in the top border.
//! - `off`: the pre-change flat numbered band, byte-identical.
//!
//! Frames derive from the prepared-line pipeline's user prompt row anchors so
//! they always span exactly the prompt's wrapped rows at the current width.
//! Decoration is chrome: border rows carry a zero-width copy map and prompt
//! rows extend their copy offsets past the rail/gutter, so selection never
//! copies a border, rail, gutter, or label glyph.
//!
//! Colors resolve through `[display.colors]` (keys `userMessageBorder`,
//! `userMessageRail`, `userMessageLabel`) with theme-token fallbacks. All
//! helpers are pure render state: no agent mutation, no filesystem/git/
//! subprocess probing.

use ratatui::prelude::*;
use unicode_width::UnicodeWidthStr;

use jcode_tui_style::palette::parse_hex;
use jcode_tui_style::theme::{dim_color, user_color};

use crate::config::{UserMessageStyle, UserMessagesConfig};

/// A color override from `[display.colors]`, if the key parses as `#rrggbb`.
fn configured_color(
    colors: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<Color> {
    colors
        .get(key)
        .and_then(|text| parse_hex(text))
        .map(|(r, g, b)| Color::Rgb(r, g, b))
}

/// Border color: `display.colors.userMessageBorder`, else the user accent.
pub(crate) fn border_color_with(colors: &std::collections::BTreeMap<String, String>) -> Color {
    configured_color(colors, "userMessageBorder").unwrap_or_else(user_color)
}

/// Rail color: `display.colors.userMessageRail`, else the user accent.
pub(crate) fn rail_color_with(colors: &std::collections::BTreeMap<String, String>) -> Color {
    configured_color(colors, "userMessageRail").unwrap_or_else(user_color)
}

/// Label color: `display.colors.userMessageLabel`, else `dim`.
pub(crate) fn label_color_with(colors: &std::collections::BTreeMap<String, String>) -> Color {
    configured_color(colors, "userMessageLabel").unwrap_or_else(dim_color)
}

/// Border/rail glyph set per capability mode.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FrameGlyphs {
    pub horizontal: &'static str,
    pub vertical: &'static str,
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
}

pub(crate) fn glyphs(ascii: bool) -> FrameGlyphs {
    if ascii {
        FrameGlyphs {
            horizontal: "-",
            vertical: "|",
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
        }
    } else {
        FrameGlyphs {
            horizontal: "─",
            vertical: "│",
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
        }
    }
}

/// Straight (non-rounded) top corners for the framed styles.
fn straight_top(g: FrameGlyphs) -> (&'static str, &'static str) {
    if g.top_left == "╭" {
        ("┌", "┐")
    } else {
        (g.top_left, g.top_right)
    }
}

/// Rail span prepended to every prompt row (framed/compact/labeled), styled
/// with the rail color. The band background stays on the prompt's own spans;
/// the rail deliberately stands on the default background like the composer
/// rail.
pub(crate) fn rail_span(
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
) -> Span<'static> {
    Span::styled(
        glyphs(ascii).vertical,
        Style::default().fg(rail_color_with(colors)),
    )
}

/// Gutter span for the copy-friendly style: one unstyled cell, no glyph.
pub(crate) fn gutter_span() -> Span<'static> {
    Span::raw(" ")
}

/// The fixed label for the labeled style.
pub(crate) const USER_LABEL: &str = "User";

/// Build a border row at `width` cells. `labeled` embeds the `User` label in
/// the top border; `rounded` selects the rounded corner glyphs (labeled) or
/// straight corners (framed styles).
fn border_line(
    width: usize,
    top: bool,
    rounded: bool,
    label: bool,
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
) -> Line<'static> {
    let g = glyphs(ascii);
    let (left, right) = if top {
        if rounded {
            (g.top_left, g.top_right)
        } else {
            straight_top(g)
        }
    } else if rounded {
        (g.bottom_left, g.bottom_right)
    } else if g.top_left == "╭" {
        ("└", "┘")
    } else {
        (g.bottom_left, g.bottom_right)
    };
    let border_style = Style::default().fg(border_color_with(colors));

    // Width budget: corners plus fill, label embedded after the opening run.
    // The label drops when it cannot fit between the corners.
    let label_text = if label && top {
        let text = format!(" {USER_LABEL} ");
        if UnicodeWidthStr::width(text.as_str()) + 2 <= width {
            text
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let label_width = UnicodeWidthStr::width(label_text.as_str());
    let fill = width.saturating_sub(2 + label_width);
    if width < 2 {
        return Line::from(Span::styled(left.repeat(width), border_style));
    }
    let mut spans = vec![
        Span::styled(left, border_style),
        Span::styled(g.horizontal.repeat(fill), border_style),
    ];
    if !label_text.is_empty() {
        spans.push(Span::styled(
            label_text,
            Style::default().fg(label_color_with(colors)),
        ));
    }
    spans.push(Span::styled(right, border_style));
    Line::from(spans)
}

/// Top border row for a prompt frame.
pub(crate) fn border_top(
    width: usize,
    style: UserMessageStyle,
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
) -> Line<'static> {
    let labeled = matches!(style, UserMessageStyle::Labeled);
    border_line(width, true, labeled, labeled, colors, ascii)
}

/// Bottom border row for a prompt frame.
pub(crate) fn border_bottom(
    width: usize,
    style: UserMessageStyle,
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
) -> Line<'static> {
    let rounded = matches!(style, UserMessageStyle::Labeled);
    border_line(width, false, rounded, false, colors, ascii)
}

/// Leading span for a prompt row under `style` (rail or gutter). Returns
/// `None` for `off`.
pub(crate) fn leading_span(
    style: UserMessageStyle,
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
) -> Option<Span<'static>> {
    let cfg = UserMessagesConfig { style };
    if cfg.rail() {
        Some(rail_span(colors, ascii))
    } else if matches!(style, UserMessageStyle::FramedCopyFriendly) {
        Some(gutter_span())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_colors() -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::new()
    }

    fn line_text(line: &Line) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn framed_borders_span_the_width_with_straight_corners() {
        let top = border_top(20, UserMessageStyle::Framed, &no_colors(), false);
        let bottom = border_bottom(20, UserMessageStyle::Framed, &no_colors(), false);
        let top_text = line_text(&top);
        let bottom_text = line_text(&bottom);
        assert_eq!(UnicodeWidthStr::width(top_text.as_str()), 20);
        assert_eq!(UnicodeWidthStr::width(bottom_text.as_str()), 20);
        assert!(top_text.starts_with('┌') && top_text.ends_with('┐'));
        assert!(bottom_text.starts_with('└') && bottom_text.ends_with('┘'));
        assert!(!top_text.contains(USER_LABEL), "framed has no label");
    }

    #[test]
    fn labeled_borders_are_rounded_with_user_label_in_top() {
        let top = border_top(24, UserMessageStyle::Labeled, &no_colors(), false);
        let bottom = border_bottom(24, UserMessageStyle::Labeled, &no_colors(), false);
        let top_text = line_text(&top);
        let bottom_text = line_text(&bottom);
        assert_eq!(UnicodeWidthStr::width(top_text.as_str()), 24);
        assert!(top_text.starts_with('╭') && top_text.ends_with('╮'));
        assert!(top_text.contains(" User "), "label embedded: {top_text:?}");
        assert!(bottom_text.starts_with('╰') && bottom_text.ends_with('╯'));
        assert!(!bottom_text.contains(USER_LABEL), "label only on top");
    }

    #[test]
    fn ascii_borders_use_plain_glyphs_and_label() {
        let top = border_top(16, UserMessageStyle::Labeled, &no_colors(), true);
        let bottom = border_bottom(16, UserMessageStyle::Framed, &no_colors(), true);
        let top_text = line_text(&top);
        let bottom_text = line_text(&bottom);
        assert_eq!(UnicodeWidthStr::width(top_text.as_str()), 16);
        assert!(top_text.starts_with('+') && top_text.ends_with('+'));
        assert!(top_text.contains('-'), "ascii fill: {top_text:?}");
        assert!(top_text.contains(" User "));
        assert!(!top_text.contains('╭') && !bottom_text.contains('└'));
        assert_eq!(bottom_text, format!("+{}+", "-".repeat(14)));
    }

    #[test]
    fn borders_never_exceed_the_available_width() {
        for width in [0usize, 1, 2, 3, 8] {
            let top = border_top(width, UserMessageStyle::Labeled, &no_colors(), false);
            assert!(
                UnicodeWidthStr::width(line_text(&top).as_str()) <= width,
                "width {width}: {:?}",
                line_text(&top)
            );
        }
    }

    #[test]
    fn leading_span_matches_style() {
        let colors = no_colors();
        assert_eq!(
            leading_span(UserMessageStyle::Framed, &colors, false).map(|s| s.content.to_string()),
            Some("│".to_string())
        );
        assert_eq!(
            leading_span(UserMessageStyle::Compact, &colors, true).map(|s| s.content.to_string()),
            Some("|".to_string())
        );
        assert_eq!(
            leading_span(UserMessageStyle::FramedCopyFriendly, &colors, false)
                .map(|s| s.content.to_string()),
            Some(" ".to_string())
        );
        assert!(leading_span(UserMessageStyle::Off, &colors, false).is_none());
        assert!(leading_span(UserMessageStyle::Labeled, &colors, false).is_some());
    }

    #[test]
    fn rail_span_color_resolves_override_then_fallback() {
        let mut colors = no_colors();
        let fallback = rail_span(&colors, false);
        assert_eq!(fallback.style.fg, Some(user_color()));
        colors.insert("userMessageRail".to_string(), "#ff8800".to_string());
        let overridden = rail_span(&colors, false);
        assert_eq!(overridden.style.fg, Some(Color::Rgb(255, 136, 0)));
    }

    #[test]
    fn border_and_label_colors_resolve_overrides() {
        let mut colors = no_colors();
        assert_eq!(border_color_with(&colors), user_color());
        assert_eq!(label_color_with(&colors), dim_color());
        colors.insert("userMessageBorder".to_string(), "#112233".to_string());
        colors.insert("userMessageLabel".to_string(), "#445566".to_string());
        assert_eq!(border_color_with(&colors), Color::Rgb(17, 34, 51));
        assert_eq!(label_color_with(&colors), Color::Rgb(68, 85, 102));
    }

    #[test]
    fn style_helpers_match_spec() {
        for (style, borders, rail, leading) in [
            (UserMessageStyle::Framed, true, true, 1),
            (UserMessageStyle::FramedCopyFriendly, true, false, 1),
            (UserMessageStyle::Compact, false, true, 1),
            (UserMessageStyle::Labeled, true, true, 1),
            (UserMessageStyle::Off, false, false, 0),
        ] {
            let cfg = UserMessagesConfig { style };
            assert_eq!(cfg.borders(), borders, "{style:?} borders");
            assert_eq!(cfg.rail(), rail, "{style:?} rail");
            assert_eq!(cfg.leading_width(), leading, "{style:?} leading");
        }
    }
}
