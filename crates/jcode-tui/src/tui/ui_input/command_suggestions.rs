use super::*;

pub(super) fn command_suggestion_lines(
    app: &dyn TuiState,
    suggestions: &[crate::tui::CommandCandidate],
    width: usize,
) -> Vec<Line<'static>> {
    // Highlight the characters of each command that the typed query matched.
    // We only highlight the command token itself (the part before the first
    // space), matched against the corresponding leading token of the input.
    let needle = command_suggestion_needle(app.input());
    let highlight = |cmd: &str, base: Style| -> Vec<Span<'static>> {
        highlight_command_spans(cmd, needle.as_deref(), base)
    };

    let mut lines = Vec::new();
    if suggestions.len() == 1 {
        let candidate = &suggestions[0];
        let cmd = &candidate.name;
        let desc = &candidate.description;
        let base = Style::default().fg(rgb(255, 213, 128));
        let mut spans = vec![kind_span(candidate)];
        spans.extend(highlight(cmd, base));
        spans.push(Span::styled(format!("  {}", desc), base));
        lines.push(Line::from(spans));
    } else if !suggestions.is_empty() {
        let selected = app
            .command_suggestion_selected()
            .min(suggestions.len().saturating_sub(1));
        let window_start = command_suggestion_window_start(selected, suggestions.len());
        let limited: Vec<_> = suggestions
            .iter()
            .skip(window_start)
            .take(app::COMMAND_SUGGESTION_VISIBLE_LIMIT)
            .collect();
        let window_end = window_start + limited.len();
        let more_count = suggestions.len().saturating_sub(window_end);
        let selected_visible = selected.saturating_sub(window_start);

        for (i, candidate) in limited.iter().enumerate() {
            let cmd = &candidate.name;
            let desc = &candidate.description;
            let is_selected = i == selected_visible;
            let description_style = if is_selected {
                Style::default().fg(rgb(255, 213, 128))
            } else {
                Style::default().fg(dim_color())
            };
            let command_style = if is_selected {
                Style::default().fg(rgb(255, 213, 128))
            } else {
                Style::default().fg(rgb(128, 203, 196))
            };
            let mut spans = vec![kind_span(candidate)];
            spans.extend(highlight(cmd, command_style));
            spans.push(Span::styled(format!("  {}", desc), description_style));
            let mut suffix = Vec::new();
            if i == 0 && window_start > 0 {
                suffix.push(Span::styled(
                    format!("  ↑{}", window_start),
                    Style::default().fg(dim_color()),
                ));
            }
            if i + 1 == limited.len() && more_count > 0 {
                suffix.push(Span::styled(
                    format!("  +{} more", more_count),
                    Style::default().fg(dim_color()),
                ));
            }
            lines.push(super::super::truncate_line_preserving_suffix_to_width(
                &Line::from(spans),
                &Line::from(suffix),
                width,
            ));
        }
    }
    lines
        .into_iter()
        .map(|line| super::super::truncate_line_with_ellipsis_to_width(&line, width))
        .collect()
}

fn kind_span(candidate: &crate::tui::CommandCandidate) -> Span<'static> {
    use crate::tui::CommandCandidateKind;
    match candidate.kind {
        CommandCandidateKind::NativeCommand => {
            Span::styled("cmd ", Style::default().fg(rgb(128, 203, 196)))
        }
        CommandCandidateKind::Skill => {
            let source = crate::message::strip_ansi_escape_sequences(
                candidate.source.as_deref().unwrap_or("unknown"),
            )
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>();
            Span::styled(
                format!("skill[{source}] "),
                Style::default().fg(rgb(195, 170, 255)),
            )
        }
        CommandCandidateKind::Tool => {
            Span::styled("tool ", Style::default().fg(rgb(128, 203, 196)))
        }
        CommandCandidateKind::Mcp => Span::styled("mcp ", Style::default().fg(rgb(195, 170, 255))),
    }
}

pub(crate) fn candidate_line(
    candidate: &crate::tui::CommandCandidate,
    width: usize,
) -> Line<'static> {
    let clean = |text: &str| {
        crate::message::strip_ansi_escape_sequences(text)
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
    };
    super::super::truncate_line_with_ellipsis_to_width(
        &Line::from(vec![
            kind_span(candidate),
            Span::raw(clean(&candidate.name)),
            Span::raw(format!("  {}", clean(&candidate.description))),
        ]),
        width,
    )
}

/// Build spans for a suggestion command, recoloring the characters that the
/// fuzzy matcher aligned with the typed query. Falls back to a single
/// unhighlighted span when there is nothing (useful) to highlight.
pub(super) fn highlight_command_spans(
    cmd: &str,
    needle: Option<&str>,
    base: Style,
) -> Vec<Span<'static>> {
    let positions: Vec<usize> = match needle {
        Some(n) if !n.is_empty() && n != "/" => crate::tui::fuzzy::fuzzy_match_positions(n, cmd),
        _ => Vec::new(),
    };
    if positions.is_empty() {
        return vec![Span::styled(cmd.to_string(), base)];
    }

    // Recolor (rather than underline) the matched characters so they stay in
    // the command palette's own hue: matched chars are a brighter version of
    // the line's base color, while unmatched chars are dimmed so the match
    // visually pops.
    let highlight_style = base
        .fg(brighten_command_color(base.fg))
        .add_modifier(Modifier::BOLD);
    let rest_style = base.fg(dim_command_color(base.fg));
    let chars: Vec<char> = cmd.chars().collect();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run_start = 0usize;
    let mut run_is_match = !chars.is_empty() && positions.contains(&0);
    for i in 1..=chars.len() {
        let cur_is_match = i < chars.len() && positions.contains(&i);
        if cur_is_match != run_is_match || i == chars.len() {
            let chunk: String = chars[run_start..i].iter().collect();
            spans.push(Span::styled(
                chunk,
                if run_is_match {
                    highlight_style
                } else {
                    rest_style
                },
            ));
            run_start = i;
            run_is_match = cur_is_match;
        }
    }
    spans
}

#[cfg(test)]
mod skill_selector_metadata_tests {
    use super::*;

    #[test]
    fn skill_provenance_source_labels_fit_compact_rows() {
        let state = crate::tui::ui::tests::TestState::default();
        let mut candidate = crate::tui::CommandCandidate::skill("/界-skill".into(), "Description");
        candidate.source = Some("project".into());
        for width in [0, 1, 20, 40, 80] {
            let lines = command_suggestion_lines(&state, &[candidate.clone()], width);
            assert_eq!(lines.len(), 1);
            assert!(lines[0].width() <= width);
            if width >= 40 {
                assert!(lines[0].to_string().contains("skill[project]"));
            }
        }
    }

    #[test]
    fn skill_selector_metadata_rows_fit_unicode_cell_widths_and_style_kind() {
        let state = crate::tui::ui::tests::TestState::default();
        let candidates = vec![
            crate::tui::CommandCandidate::skill(
                "/界-foo".into(),
                "Does foo 界 with a long explanation",
            ),
            crate::tui::CommandCandidate::from(("/help".into(), "Show help")),
        ];
        for width in [0, 1, 20, 40, 80] {
            let lines = command_suggestion_lines(&state, &candidates, width);
            assert_eq!(lines.len(), 2);
            for line in &lines {
                assert!(line.width() <= width, "{line:?} exceeds {width}");
                let text: String = line
                    .spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect();
                assert!(!text.chars().any(char::is_control));
                if width == 0 {
                    assert!(text.is_empty());
                }
                if width == 1 {
                    assert_eq!(text, "…");
                }
            }
            if width == 80 {
                assert_eq!(lines[0].spans[0].content, "skill[unknown] ");
                assert_eq!(lines[1].spans[0].content, "cmd ");
                assert_ne!(lines[0].spans[0].style, lines[1].spans[0].style);
                assert!(lines[0].to_string().contains("Does foo 界"));
            }
            if width == 20 {
                assert!(lines[0].to_string().ends_with('…'));
            }
        }
    }
}

#[cfg(test)]
mod navigation_metadata_tests {
    use super::*;

    #[test]
    fn skill_selector_metadata_long_descriptions_keep_more_indicator_visible() {
        let state = crate::tui::ui::tests::TestState::default();
        let candidates: Vec<_> = (0..app::COMMAND_SUGGESTION_VISIBLE_LIMIT + 2)
            .map(|i| {
                crate::tui::CommandCandidate::skill(
                    format!("/skill-{i}"),
                    &"界 description ".repeat(30),
                )
            })
            .collect();
        for width in [0, 1, 20, 40, 80] {
            let lines = command_suggestion_lines(&state, &candidates, width);
            assert!(lines.iter().all(|line| line.width() <= width));
            if width >= 20 {
                assert!(lines.last().unwrap().to_string().ends_with("  +2 more"));
            }
        }
    }
}
