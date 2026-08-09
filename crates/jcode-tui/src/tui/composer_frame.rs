//! Composer frame (Zentui port, roadmap P2): accent rail + metadata row.
//!
//! The rail is a one-column accent strip on the left of every composer row,
//! colored by the active composer mode with the same precedence as the prompt
//! glyph (`input_prompt`): shell, processing/queued, skill, chat. The metadata
//! row is one layout-owned row at the bottom of the composer showing
//! `model · provider( · effort)`, right-aligned and muted, surviving exactly
//! the states (processing, queued, overscroll, narrow widths) where the
//! opportunistic right fact stack stands down.
//!
//! Both surfaces are pure render state: they read the per-frame
//! [`InfoWidgetData`] snapshot and config, and never mutate agent state or
//! probe the filesystem, git, or any subprocess on the render path.
//!
//! Colors resolve through the existing `[display.colors]` map (keys
//! `composerRail`, `composerRailShell`, `composerRailQueued`,
//! `composerRailSkill`, `composerMetadata`) with theme-token fallbacks.

use ratatui::{prelude::*, widgets::Paragraph};
use unicode_width::UnicodeWidthStr;

use super::TuiState;
use super::info_widget::{InfoWidgetData, model::shorten_model_name};
use jcode_tui_style::palette::parse_hex;
use super::ui::input_ui::shell_mode_color;
use jcode_tui_style::theme::{accent_color, dim_color, queued_color, user_color};

/// Composer mode for rail coloring. Mirrors the precedence of
/// `ui_input::input_prompt` so the rail and the prompt glyph always agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RailMode {
    Chat,
    Shell,
    Queued,
    Skill,
}

pub(crate) fn rail_mode(app: &dyn TuiState) -> RailMode {
    if crate::tui::app::extract_input_shell_command(app.input()).is_some() {
        RailMode::Shell
    } else if app.is_processing() {
        RailMode::Queued
    } else if app.active_skill().is_some() {
        RailMode::Skill
    } else {
        RailMode::Chat
    }
}

/// A color override from `[display.colors]`, if the key parses as `#rrggbb`.
fn configured_color(colors: &std::collections::BTreeMap<String, String>, key: &str) -> Option<Color> {
    colors
        .get(key)
        .and_then(|text| parse_hex(text))
        .map(|(r, g, b)| Color::Rgb(r, g, b))
}

/// Rail color for a mode: `display.colors` override, else the existing theme
/// mode color.
pub(crate) fn rail_color_with(
    mode: RailMode,
    colors: &std::collections::BTreeMap<String, String>,
) -> Color {
    let (key, fallback) = match mode {
        RailMode::Chat => ("composerRail", user_color()),
        RailMode::Shell => ("composerRailShell", shell_mode_color()),
        RailMode::Queued => ("composerRailQueued", queued_color()),
        RailMode::Skill => ("composerRailSkill", accent_color()),
    };
    configured_color(colors, key).unwrap_or(fallback)
}

/// Metadata text color: `display.colors.composerMetadata`, else `dim`.
pub(crate) fn metadata_color_with(colors: &std::collections::BTreeMap<String, String>) -> Color {
    configured_color(colors, "composerMetadata").unwrap_or_else(dim_color)
}

/// Rail glyph: `│` in auto icon mode, `|` in ASCII mode.
pub(crate) fn rail_glyph(ascii: bool) -> &'static str {
    if ascii { "|" } else { "│" }
}

/// Draw the accent rail down the left column of `area` (the full composer
/// chunk, including the metadata row).
pub(crate) fn draw_rail(frame: &mut Frame, app: &dyn TuiState, area: Rect, ascii: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let mode = rail_mode(app);
    let color = rail_color_with(mode, &crate::config::config().display.colors);
    let style = Style::default().fg(color);
    let glyph = rail_glyph(ascii);
    for row in 0..area.height {
        let y = area.y + row;
        if y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(glyph, style))),
                Rect::new(area.x, y, 1, 1),
            );
        }
    }
}

/// Build the metadata row content: `model · provider( · effort)`,
/// right-aligned within `width`, muted. Omission rules: no effort segment
/// when off/unset, no provider segment when unknown, no upstream extras when
/// absent, and no content at all when the model label is unavailable (the
/// row stays reserved so composer height is stable). Degradation drops
/// effort, then provider, then truncates the model label; the row never
/// wraps.
pub(crate) fn metadata_spans(
    data: &InfoWidgetData,
    colors: &std::collections::BTreeMap<String, String>,
    ascii: bool,
    width: u16,
) -> Vec<Span<'static>> {
    let Some(model) = data.model.as_deref().map(shorten_model_name) else {
        return Vec::new();
    };
    let effort = data
        .reasoning_effort
        .as_deref()
        .filter(|effort| {
            let normalized = effort.trim().to_ascii_lowercase();
            !normalized.is_empty() && normalized != "off" && normalized != "none"
        })
        .map(str::to_string);
    let provider = match (data.provider_name.as_ref(), data.upstream_provider.as_ref()) {
        (Some(base), Some(upstream)) if !upstream.is_empty() => Some(format!("{base}/{upstream}")),
        (Some(base), _) if !base.is_empty() => Some(base.clone()),
        (None, Some(upstream)) if !upstream.is_empty() => Some(upstream.clone()),
        _ => None,
    };

    let sep = if ascii { " | " } else { " · " };
    let style = Style::default().fg(metadata_color_with(colors));
    let width = width as usize;

    // Fixed drop order (spec): effort, then provider, then model truncation.
    let mut effort = effort;
    let mut provider = provider;
    let model = model;
    loop {
        let mut text = model.clone();
        if let Some(provider) = &provider {
            text.push_str(sep);
            text.push_str(provider);
        }
        if let Some(effort) = &effort {
            text.push_str(sep);
            text.push_str(effort);
        }
        if UnicodeWidthStr::width(text.as_str()) <= width {
            let pad = width.saturating_sub(UnicodeWidthStr::width(text.as_str()));
            let mut spans = Vec::new();
            if pad > 0 {
                spans.push(Span::raw(" ".repeat(pad)));
            }
            spans.push(Span::styled(text, style));
            return spans;
        }
        if effort.take().is_some() {
            continue;
        }
        if provider.take().is_some() {
            continue;
        }
        // Truncate the model label with an ellipsis as the last resort.
        let budget = width.saturating_sub(1);
        if budget == 0 {
            return Vec::new();
        }
        let mut truncated = String::new();
        let mut used = 0usize;
        for ch in model.chars() {
            let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if used + cw > budget {
                break;
            }
            used += cw;
            truncated.push(ch);
        }
        truncated.push('…');
        let pad = width.saturating_sub(UnicodeWidthStr::width(truncated.as_str()));
        let mut spans = Vec::new();
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad)));
        }
        spans.push(Span::styled(truncated, style));
        return spans;
    }
}

/// Draw the metadata row into `area` (a single-row rect at the bottom of the
/// composer, already inset for the rail).
pub(crate) fn draw_metadata(
    frame: &mut Frame,
    data: &InfoWidgetData,
    area: Rect,
    ascii: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let spans = metadata_spans(data, &crate::config::config().display.colors, ascii, area.width);
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_colors() -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::new()
    }

    fn metadata_data() -> InfoWidgetData {
        InfoWidgetData {
            model: Some("claude-fable-5".to_string()),
            provider_name: Some("anthropic".to_string()),
            reasoning_effort: Some("high".to_string()),
            ..Default::default()
        }
    }

    fn text_of(spans: &[Span<'static>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn metadata_full_render_is_right_aligned() {
        let spans = metadata_spans(&metadata_data(), &no_colors(), false, 30);
        let text = text_of(&spans);
        assert!(text.ends_with("fable · anthropic · high"), "{text:?}");
        assert_eq!(UnicodeWidthStr::width(text.as_str()), 30);
        assert!(text.starts_with(' '), "right-aligned pad: {text:?}");
    }

    #[test]
    fn metadata_omits_effort_when_off_or_unset() {
        for effort in [None, Some("off"), Some("none"), Some("")] {
            let mut data = metadata_data();
            data.reasoning_effort = effort.map(str::to_string);
            let spans = metadata_spans(&data, &no_colors(), false, 40);
            let text = text_of(&spans);
            assert!(text.ends_with("fable · anthropic"), "{effort:?}: {text:?}");
            assert!(!text.contains("off"), "no off placeholder: {text:?}");
        }
    }

    #[test]
    fn metadata_omits_provider_when_absent() {
        let mut data = metadata_data();
        data.provider_name = None;
        let spans = metadata_spans(&data, &no_colors(), false, 40);
        assert!(text_of(&spans).ends_with("fable · high"));
    }

    #[test]
    fn metadata_empty_when_model_unavailable() {
        let data = InfoWidgetData::default();
        let spans = metadata_spans(&data, &no_colors(), false, 40);
        assert!(spans.is_empty(), "empty row keeps height stable");
    }

    #[test]
    fn metadata_drops_in_documented_order() {
        let data = metadata_data();
        // Width 24 fits "fable · anthropic · high" exactly (24 cols).
        let full = text_of(&metadata_spans(&data, &no_colors(), false, 24));
        assert_eq!(full, "fable · anthropic · high");
        // One narrower drops effort.
        let no_effort = text_of(&metadata_spans(&data, &no_colors(), false, 23));
        assert_eq!(no_effort.trim_start(), "fable · anthropic");
        // Narrower still drops provider.
        let model_only = text_of(&metadata_spans(&data, &no_colors(), false, 10));
        assert_eq!(model_only.trim_start(), "fable");
        // Below the model length, truncate with ellipsis.
        let truncated = text_of(&metadata_spans(&data, &no_colors(), false, 4));
        assert_eq!(truncated.trim_start(), "fab…");
    }

    #[test]
    fn metadata_ascii_separator() {
        let spans = metadata_spans(&metadata_data(), &no_colors(), true, 40);
        let text = text_of(&spans);
        assert!(text.contains("fable | anthropic | high"), "{text:?}");
        assert!(!text.contains('·'));
    }

    #[test]
    fn metadata_includes_upstream_when_present() {
        let mut data = metadata_data();
        data.upstream_provider = Some("fireworks".to_string());
        let spans = metadata_spans(&data, &no_colors(), false, 60);
        assert!(text_of(&spans).contains("anthropic/fireworks"));
    }

    #[test]
    fn rail_colors_fall_back_to_mode_colors() {
        let colors = no_colors();
        assert_eq!(rail_color_with(RailMode::Chat, &colors), user_color());
        assert_eq!(rail_color_with(RailMode::Shell, &colors), shell_mode_color());
        assert_eq!(rail_color_with(RailMode::Queued, &colors), queued_color());
        assert_eq!(rail_color_with(RailMode::Skill, &colors), accent_color());
    }

    #[test]
    fn rail_colors_honor_display_colors_overrides() {
        let mut colors = no_colors();
        colors.insert("composerRailShell".to_string(), "#112233".to_string());
        assert_eq!(
            rail_color_with(RailMode::Shell, &colors),
            Color::Rgb(0x11, 0x22, 0x33)
        );
        // Unset keys keep the theme fallback.
        assert_eq!(rail_color_with(RailMode::Chat, &colors), user_color());
    }

    #[test]
    fn metadata_color_honors_override() {
        let mut colors = no_colors();
        colors.insert("composerMetadata".to_string(), "#a0b0c0".to_string());
        assert_eq!(
            metadata_color_with(&colors),
            Color::Rgb(0xa0, 0xb0, 0xc0)
        );
        assert_eq!(metadata_color_with(&no_colors()), dim_color());
    }

    #[test]
    fn rail_glyph_modes() {
        assert_eq!(rail_glyph(false), "│");
        assert_eq!(rail_glyph(true), "|");
    }

    #[test]
    fn metadata_deterministic() {
        let first = metadata_spans(&metadata_data(), &no_colors(), false, 30);
        let second = metadata_spans(&metadata_data(), &no_colors(), false, 30);
        assert_eq!(text_of(&first), text_of(&second));
    }
}
