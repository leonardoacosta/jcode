pub mod chrome;
pub mod layout;
pub mod memory_tiles;
pub mod swarm_gallery;
pub mod swarm_tiles;

use jcode_tui_style::theme::chrome_symbol;
use ratatui::prelude::{Line, Span, Style};

pub fn render_rounded_box(
    title: &str,
    content: Vec<Line<'static>>,
    max_width: usize,
    border_style: Style,
) -> Vec<Line<'static>> {
    if content.is_empty() || max_width < 6 {
        return Vec::new();
    }

    let max_content_width = content
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(0)
        .min(max_width.saturating_sub(4));

    let truncated_title = truncate_line_with_ellipsis_to_width(
        &Line::from(Span::raw(format!(" {} ", title))),
        max_width.saturating_sub(2).max(1),
    );
    let title_text = line_plain_text(&truncated_title);
    let title_len = truncated_title.width();
    let box_content_width = max_content_width.max(title_len.saturating_sub(2));

    if box_content_width < 6 {
        return Vec::new();
    }

    let box_width = box_content_width + 4;
    let border_chars = box_width.saturating_sub(title_len + 2);
    let horizontal = chrome_symbol("─", "-");
    let left_border = horizontal.repeat(border_chars / 2);
    let right_border = horizontal.repeat(border_chars - border_chars / 2);

    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{}{}{}{}{}",
            chrome_symbol("╭", "+"),
            left_border,
            title_text,
            right_border,
            chrome_symbol("╮", "+")
        ),
        border_style,
    )));

    for line in content {
        let truncated = truncate_line_to_width(&line, box_content_width);
        let padding = box_content_width.saturating_sub(truncated.width());
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::styled(chrome_symbol("│ ", "| "), border_style));
        spans.extend(truncated.spans);
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.push(Span::styled(chrome_symbol(" │", " |"), border_style));
        lines.push(Line::from(spans));
    }

    let bottom_border = horizontal.repeat(box_width.saturating_sub(2));
    lines.push(Line::from(Span::styled(
        format!(
            "{}{}{}",
            chrome_symbol("╰", "+"),
            bottom_border,
            chrome_symbol("╯", "+")
        ),
        border_style,
    )));

    lines
}

pub fn truncate_line_to_width(line: &Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut remaining = width;
    for span in &line.spans {
        if remaining == 0 {
            break;
        }
        let text = span.content.as_ref();
        let span_width = unicode_width::UnicodeWidthStr::width(text);
        if span_width <= remaining {
            spans.push(span.clone());
            remaining -= span_width;
        } else {
            let mut clipped = String::new();
            let mut used = 0;
            for ch in text.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cw > remaining {
                    break;
                }
                clipped.push(ch);
                used += cw;
            }
            if !clipped.is_empty() {
                spans.push(Span::styled(clipped, span.style));
            }
            remaining = 0;
        }
    }

    if spans.is_empty() {
        Line::from("")
    } else {
        Line::from(spans)
    }
}

pub fn truncate_line_with_ellipsis_to_width(line: &Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    if line.width() <= width {
        return line.clone();
    }
    if width == 1 {
        return Line::from(Span::raw(chrome_symbol("…", "."))).style(line.style);
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut remaining = width.saturating_sub(1);
    let mut ellipsis_style = Style::default();

    for span in &line.spans {
        if remaining == 0 {
            break;
        }
        let text = span.content.as_ref();
        let span_width = unicode_width::UnicodeWidthStr::width(text);
        if span_width <= remaining {
            spans.push(span.clone());
            remaining -= span_width;
            ellipsis_style = span.style;
        } else {
            let mut clipped = String::new();
            let mut used = 0;
            for ch in text.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cw > remaining {
                    break;
                }
                clipped.push(ch);
                used += cw;
            }
            if !clipped.is_empty() {
                spans.push(Span::styled(clipped, span.style));
                ellipsis_style = span.style;
            }
            break;
        }
    }

    spans.push(Span::styled(chrome_symbol("…", "."), ellipsis_style));
    let mut truncated = Line::from(spans).style(line.style);
    truncated.alignment = line.alignment;
    truncated
}

pub fn truncate_line_preserving_suffix_to_width(
    prefix: &Line<'static>,
    suffix: &Line<'static>,
    width: usize,
) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }

    if suffix.width() == 0 {
        return truncate_line_with_ellipsis_to_width(prefix, width);
    }

    let mut combined_spans = prefix.spans.clone();
    combined_spans.extend(suffix.spans.clone());
    let mut combined = Line::from(combined_spans);
    combined.alignment = prefix.alignment;
    if combined.width() <= width {
        return combined;
    }

    let suffix_width = suffix.width();
    if suffix_width >= width {
        let mut truncated = truncate_line_with_ellipsis_to_width(suffix, width);
        truncated.alignment = prefix.alignment;
        return truncated;
    }

    let prefix_budget = width.saturating_sub(suffix_width);
    let mut prefix_part = truncate_line_with_ellipsis_to_width(prefix, prefix_budget);
    prefix_part.spans.extend(suffix.spans.clone());
    prefix_part.alignment = prefix.alignment;
    prefix_part
}

pub fn line_plain_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>()
}

#[cfg(test)]
mod ascii_tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend, widgets::Paragraph};

    #[test]
    fn ascii_chrome_environment_matrix() {
        if std::env::var_os("JCODE_TEST_ASCII_CHROME_CHILD").is_none() {
            for value in [
                None,
                Some("on"),
                Some(" TRUE "),
                Some("1"),
                Some("yes"),
                Some("off"),
                Some("0"),
                Some("false"),
                Some("no"),
                Some(""),
                Some("invalid"),
            ] {
                let mut command = std::process::Command::new(std::env::current_exe().unwrap());
                command
                    .args([
                        "--exact",
                        "ascii_tests::ascii_chrome_environment_matrix",
                        "--nocapture",
                    ])
                    .env("JCODE_TEST_ASCII_CHROME_CHILD", "1")
                    .env_remove("JCODE_ASCII_SAFE");
                if let Some(value) = value {
                    command.env("JCODE_ASCII_SAFE", value);
                }
                let output = command.output().unwrap();
                assert!(
                    output.status.success(),
                    "{value:?}: {}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            return;
        }
        let ascii = matches!(
            std::env::var("JCODE_ASCII_SAFE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "on" | "true" | "1" | "yes"
        );
        let border = Style::default().fg(ratatui::prelude::Color::Red);
        let content = "中文 α /tmp/路径 https://例子.test/✓ tool→output";
        let lines = render_rounded_box("标题 ✓", vec![Line::from(content)], 100, border);
        let text = lines
            .iter()
            .map(line_plain_text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains(content), "content changed: {text}");
        assert!(text.contains("标题 ✓"), "title changed: {text}");
        assert_eq!(text.contains('╭'), !ascii);
        assert_eq!(text.contains('+'), ascii);
        for width in 0..25 {
            let line = truncate_line_with_ellipsis_to_width(&Line::from("中文abcdefghi"), width);
            assert!(line.width() <= width);
            if width > 0 && width < 13 {
                assert!(line_plain_text(&line).ends_with(if ascii { "." } else { "…" }));
            }
            let box_lines = render_rounded_box(
                "title",
                vec![Line::from("long generated chrome")],
                width,
                border,
            );
            assert!(box_lines.iter().all(|line| line.width() <= width));
        }
        let spinner = jcode_tui_style::theme::activity_indicator(0.0, 12.5, false);
        assert_eq!(spinner.is_ascii(), ascii);
        assert!(jcode_tui_style::theme::is_activity_indicator_frame(spinner));
        let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
        terminal
            .draw(|frame| {
                crate::chrome::draw_right_rail_chrome(
                    frame,
                    frame.area(),
                    Line::from("中文/title"),
                    border,
                );
            })
            .unwrap();
        assert_eq!(
            terminal.backend().buffer()[(0, 1)].symbol(),
            if ascii { "|" } else { "│" }
        );
        let lines = render_rounded_box("title", vec![Line::from("content")], 20, border);
        terminal
            .draw(|frame| frame.render_widget(Paragraph::new(lines), frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), if ascii { "+" } else { "╭" });
        assert!(
            buffer
                .content
                .iter()
                .all(|cell| !ascii || cell.symbol().is_ascii())
        );
    }
}
