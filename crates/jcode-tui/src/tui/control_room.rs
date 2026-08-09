use jcode_session_types::{ContextRow, ContextSnapshot};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Debug, Clone)]
pub struct ControlRoomOverlay {
    snapshot: ContextSnapshot,
    selected_row: usize,
    scroll: u16,
    feedback: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlRoomAction {
    Close,
    Copy,
    Focus,
    None,
}

impl ControlRoomOverlay {
    pub fn new(snapshot: ContextSnapshot) -> Self {
        Self {
            snapshot,
            selected_row: 0,
            scroll: 0,
            feedback: None,
        }
    }

    pub fn snapshot(&self) -> &ContextSnapshot {
        &self.snapshot
    }

    pub fn selected_row_index(&self) -> usize {
        self.selected_row
    }

    pub fn selected_row(&self) -> Option<&ContextRow> {
        self.rows().get(self.selected_row).copied()
    }

    pub fn set_feedback(&mut self, feedback: impl Into<String>) {
        self.feedback = Some(feedback.into());
    }

    pub fn move_next(&mut self) {
        let max = self.rows().len().saturating_sub(1);
        self.selected_row = (self.selected_row + 1).min(max);
        self.ensure_selection_visible();
    }

    pub fn move_previous(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);
        self.ensure_selection_visible();
    }

    pub fn page_down(&mut self) {
        let max = self.rows().len().saturating_sub(1);
        self.selected_row = (self.selected_row + 5).min(max);
        self.ensure_selection_visible();
    }

    pub fn page_up(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(5);
        self.ensure_selection_visible();
    }

    pub fn first(&mut self) {
        self.selected_row = 0;
        self.ensure_selection_visible();
    }

    pub fn last(&mut self) {
        self.selected_row = self.rows().len().saturating_sub(1);
        self.ensure_selection_visible();
    }

    pub fn selected_copy_value(&self) -> Option<&str> {
        self.selected_row()
            .filter(|row| row.copyable)
            .and_then(ContextRow::copy_value)
    }

    pub fn selected_focus_label(&self) -> Option<&str> {
        self.selected_row()
            .filter(|row| row.focusable)
            .map(|row| row.label.as_str())
    }

    pub fn handle_key(&mut self, code: crossterm::event::KeyCode) -> ControlRoomAction {
        use crossterm::event::KeyCode;
        match code {
            KeyCode::Esc => ControlRoomAction::Close,
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_previous();
                ControlRoomAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_next();
                ControlRoomAction::None
            }
            KeyCode::PageUp => {
                self.page_up();
                ControlRoomAction::None
            }
            KeyCode::PageDown => {
                self.page_down();
                ControlRoomAction::None
            }
            KeyCode::Home => {
                self.first();
                ControlRoomAction::None
            }
            KeyCode::End => {
                self.last();
                ControlRoomAction::None
            }
            KeyCode::Enter | KeyCode::Char('c') | KeyCode::Char('y') => ControlRoomAction::Copy,
            KeyCode::Char('f') => ControlRoomAction::Focus,
            _ => ControlRoomAction::None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let area = centered_rect(area, 84, 78);
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Context Control Room · Alt+O ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(inner);

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "Jcode is authoritative",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" · Herdr is execution substrate · no spawning from this overlay"),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let lines = self.render_lines();
        let visible_height = chunks[1].height.saturating_sub(1).max(1) as usize;
        if self.selected_row + 6 > self.scroll as usize + visible_height {
            self.scroll = (self.selected_row + 6).saturating_sub(visible_height) as u16;
        }
        let body = Paragraph::new(lines)
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(body, chunks[1]);

        let footer_text = self.feedback.as_deref().unwrap_or(
            "↑/↓ navigate · Enter/C copy selected value · F focus existing surface · Esc close",
        );
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, chunks[2]);
    }

    pub fn render_text(&self) -> String {
        self.render_lines()
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut row_index = 0usize;
        self.push_section_lines(
            &mut lines,
            &self.snapshot.semantic.title,
            &self.snapshot.semantic.rows,
            &mut row_index,
        );
        lines.push(Line::raw(""));
        self.push_section_lines(
            &mut lines,
            &self.snapshot.execution.title,
            &self.snapshot.execution.rows,
            &mut row_index,
        );
        lines
    }

    fn push_section_lines(
        &self,
        lines: &mut Vec<Line<'static>>,
        title: &str,
        rows: &[ContextRow],
        row_index: &mut usize,
    ) {
        lines.push(Line::from(Span::styled(
            title.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        for row in rows {
            let selected = *row_index == self.selected_row;
            let marker = if selected { "▶" } else { " " };
            let actions = match (row.copyable, row.focusable) {
                (true, true) => "copy focus",
                (true, false) => "copy",
                (false, true) => "focus",
                (false, false) => "view",
            };
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{marker} {:<14}", row.label), style),
                Span::styled(row.display_value().to_string(), style),
                Span::styled(
                    format!(
                        "  [{} · {} · {actions}]",
                        row.provenance.label(),
                        row.confidence.label()
                    ),
                    style.fg(if selected {
                        Color::Black
                    } else {
                        Color::DarkGray
                    }),
                ),
            ]));
            *row_index += 1;
        }
    }

    fn rows(&self) -> Vec<&ContextRow> {
        self.snapshot.rows().collect()
    }

    fn ensure_selection_visible(&mut self) {
        let selected_line = self.selected_row.saturating_add(1);
        if selected_line < self.scroll as usize {
            self.scroll = selected_line as u16;
        }
    }
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_session_types::{ContextRow, ContextSection, ContextSnapshot};

    #[test]
    fn render_text_contains_sections_and_unavailable_rows() {
        let overlay = ControlRoomOverlay::new(ContextSnapshot {
            semantic: ContextSection {
                title: "Semantic context".into(),
                rows: vec![ContextRow::persisted("Project", "jcode", "project:jcode")],
            },
            execution: ContextSection {
                title: "Execution substrate".into(),
                rows: vec![ContextRow::unavailable("Herdr pane", "Herdr not detected")],
            },
        });
        let text = overlay.render_text();
        assert!(text.contains("Semantic context"));
        assert!(text.contains("Execution substrate"));
        assert!(text.contains("persisted"));
        assert!(text.contains("unavailable"));
        assert!(text.contains("Herdr not detected"));
    }
}
