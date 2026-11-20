//! World issue board: a pure-presentation overlay with one tab per workflow
//! state (count badge) and a table of that state's issues.
//!
//! The overlay owns no tracker logic. A caller hands it a [`BoardLoad`] and
//! reads back [`BoardAction`]s (close, refresh, confirmed move). Every chrome
//! row is a single visual row truncated by Unicode display-cell width.

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::Clear};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const PANEL_BG: Color = Color::Rgb(24, 28, 40);
const ACCENT: Color = Color::Rgb(120, 140, 190);
const SELECTED_BG: Color = Color::Rgb(38, 42, 56);
const TARGET_BG: Color = Color::Rgb(60, 78, 120);
const MUTED: Color = Color::Rgb(140, 146, 163);
const MUTED_DARK: Color = Color::Rgb(100, 106, 122);
const WARN: Color = Color::Rgb(230, 190, 90);
const ERROR: Color = Color::Rgb(230, 110, 110);

/// Row of the state tab strip.
const TAB_ROW: u16 = 2;
/// Widest identifier column in the issue table.
const MAX_ID_CELLS: usize = 14;
/// Rows used by chrome above the body (title, scope, tabs, table header).
const TOP_ROWS: u16 = 4;
/// Rows used by chrome below the body (status line, hints).
const BOTTOM_ROWS: u16 = 2;

pub const READ_ONLY_NOTICE: &str = "read-only: ADO boards cannot change status";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardLoad {
    Loading,
    Unavailable(String),
    Unauthorized(String),
    Error(String),
    Empty,
    Ready(BoardData),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardData {
    pub scope_label: String,
    pub read_only: bool,
    /// Columns in provider workflow-state order.
    pub columns: Vec<BoardColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardColumn {
    pub id: String,
    pub name: String,
    pub issues: Vec<BoardIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveRequest {
    pub issue_id: String,
    pub from_column_id: String,
    pub to_column_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardAction {
    None,
    Close,
    Refresh,
    Confirmed(MoveRequest),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingMove {
    from: usize,
    to: usize,
    issue: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Browse,
    /// Picking a target column.
    Moving(PendingMove),
    /// Waiting for y/n on a chosen target.
    Confirm(PendingMove),
}

#[derive(Debug)]
pub struct BoardOverlay {
    load: BoardLoad,
    col: usize,
    row: usize,
    mode: Mode,
    notice: Option<String>,
    /// Column id last shown on a board, kept across Loading/error states so a
    /// refresh returns to the same column.
    anchor: Option<String>,
}

impl BoardOverlay {
    pub fn new(load: BoardLoad) -> Self {
        let mut overlay = Self {
            load: BoardLoad::Loading,
            col: 0,
            row: 0,
            mode: Mode::Browse,
            notice: None,
            anchor: None,
        };
        overlay.set_load(load);
        overlay
    }

    /// Show a one-row notice (for example a move result) until the next
    /// `set_load` or key press clears it.
    pub fn set_notice(&mut self, text: impl Into<String>) {
        self.notice = Some(text.into());
    }

    /// Replace the displayed state. Any move in progress is dropped (its
    /// indices may no longer match) and the selection stays on the same column
    /// id when it still exists, clamped to valid positions otherwise.
    pub fn set_load(&mut self, load: BoardLoad) {
        if let Some(id) = self
            .board()
            .and_then(|d| d.columns.get(self.col))
            .map(|c| c.id.clone())
        {
            self.anchor = Some(id);
        }
        self.load = load;
        self.mode = Mode::Browse;
        self.notice = None;
        let anchored = self.anchor.as_ref().and_then(|id| {
            self.board()
                .and_then(|d| d.columns.iter().position(|c| &c.id == id))
        });
        if let Some(idx) = anchored {
            self.col = idx;
        }
        self.clamp_selection();
    }

    /// The board only when there is something to navigate. A successful query
    /// with no issues is shown as the empty state, never as bare columns.
    fn board(&self) -> Option<&BoardData> {
        match &self.load {
            BoardLoad::Ready(data) if has_issues(data) => Some(data),
            _ => None,
        }
    }

    fn clamp_selection(&mut self) {
        let Some(data) = self.board() else {
            return;
        };
        let col = self.col.min(data.columns.len().saturating_sub(1));
        let row = self
            .row
            .min(data.columns[col].issues.len().saturating_sub(1));
        self.col = col;
        self.row = row;
    }

    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> BoardAction {
        // Ctrl/Alt chords never act as plain letters (y must not confirm on Ctrl+Y).
        let chord = modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let code = match code {
            KeyCode::Char(_) if chord => return BoardAction::None,
            KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
            other => other,
        };
        self.notice = None;
        match self.mode {
            Mode::Browse => self.handle_browse_key(code),
            Mode::Moving(pending) => self.handle_moving_key(code, pending),
            Mode::Confirm(pending) => self.handle_confirm_key(code, pending),
        }
    }

    fn handle_browse_key(&mut self, code: KeyCode) -> BoardAction {
        match code {
            KeyCode::Esc => return BoardAction::Close,
            KeyCode::Char('r') => return BoardAction::Refresh,
            _ => {}
        }
        let Some(data) = self.board() else {
            return BoardAction::None;
        };
        let columns = data.columns.len();
        let read_only = data.read_only;
        match code {
            KeyCode::Left | KeyCode::Char('h') => self.col = self.col.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => self.col = (self.col + 1).min(columns - 1),
            KeyCode::Up | KeyCode::Char('k') => self.row = self.row.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.row += 1,
            KeyCode::Char('m') => self.start_move(read_only),
            _ => {}
        }
        self.clamp_selection();
        BoardAction::None
    }

    fn start_move(&mut self, read_only: bool) {
        if read_only {
            self.notice = Some(READ_ONLY_NOTICE.to_string());
            return;
        }
        let has_issue = self
            .board()
            .and_then(|d| d.columns.get(self.col))
            .is_some_and(|c| self.row < c.issues.len());
        if !has_issue {
            self.notice = Some("no issue selected".to_string());
            return;
        }
        // The synthetic "Other" column (empty id) is display-only: it has no
        // workflow state to move from or to.
        let movable: Vec<usize> = self
            .board()
            .map(|d| {
                d.columns
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| !c.id.is_empty())
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        if !movable.contains(&self.col) {
            self.notice = Some("Other column is display-only".to_string());
            return;
        }
        let Some(to) = movable
            .iter()
            .copied()
            .find(|i| *i > self.col)
            .or_else(|| movable.iter().rev().copied().find(|i| *i < self.col))
        else {
            self.notice = Some("no other column to move to".to_string());
            return;
        };
        self.mode = Mode::Moving(PendingMove {
            from: self.col,
            to,
            issue: self.row,
        });
    }

    fn handle_moving_key(&mut self, code: KeyCode, mut pending: PendingMove) -> BoardAction {
        let columns = self.board().map_or(0, |d| d.columns.len());
        match code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Left | KeyCode::Char('h') => {
                let board = self.board();
                let movable = |i: usize| {
                    board.is_some_and(|d| d.columns.get(i).is_some_and(|c| !c.id.is_empty()))
                };
                pending.to = step_target(pending.to, pending.from, columns, false, &movable);
                self.mode = Mode::Moving(pending);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let board = self.board();
                let movable = |i: usize| {
                    board.is_some_and(|d| d.columns.get(i).is_some_and(|c| !c.id.is_empty()))
                };
                pending.to = step_target(pending.to, pending.from, columns, true, &movable);
                self.mode = Mode::Moving(pending);
            }
            KeyCode::Enter => {
                // Defensive: a read-only board never reaches a confirm prompt.
                let read_only = self.board().is_none_or(|d| d.read_only);
                if read_only {
                    self.notice = Some(READ_ONLY_NOTICE.to_string());
                    self.mode = Mode::Browse;
                } else {
                    self.mode = Mode::Confirm(pending);
                }
            }
            _ => {}
        }
        BoardAction::None
    }

    fn handle_confirm_key(&mut self, code: KeyCode, pending: PendingMove) -> BoardAction {
        match code {
            KeyCode::Char('y') => {
                self.mode = Mode::Browse;
                self.move_request(pending)
                    .map_or(BoardAction::None, BoardAction::Confirmed)
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::Browse;
                BoardAction::None
            }
            _ => BoardAction::None,
        }
    }

    fn move_request(&self, pending: PendingMove) -> Option<MoveRequest> {
        let data = self.board()?;
        if data.read_only {
            return None;
        }
        let from = data.columns.get(pending.from)?;
        let to = data.columns.get(pending.to)?;
        let issue = from.issues.get(pending.issue)?;
        Some(MoveRequest {
            issue_id: issue.id.clone(),
            from_column_id: from.id.clone(),
            to_column_id: to.id.clone(),
        })
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        self.render_to(frame.buffer_mut(), area);
    }

    pub fn render_to(&self, buf: &mut Buffer, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        Clear.render(area, buf);
        buf.set_style(area, Style::default().bg(PANEL_BG));

        let h = area.height;
        let full = h >= TOP_ROWS + BOTTOM_ROWS;
        put(
            buf,
            area,
            0,
            &self.title(),
            Style::default().fg(Color::White).bold(),
        );

        if full {
            self.render_body(buf, area, h - TOP_ROWS - BOTTOM_ROWS);
        }
        // On terminals too short for the full layout the last row carries the
        // status line when there is one, otherwise the hints.
        let status = self.status_line();
        if full {
            if let Some((text, style)) = &status {
                put(buf, area, h - 2, text, *style);
            }
            put(
                buf,
                area,
                h - 1,
                &self.hints(),
                Style::default().fg(MUTED_DARK),
            );
        } else if h > 1 {
            match &status {
                Some((text, style)) => put(buf, area, h - 1, text, *style),
                None => put(
                    buf,
                    area,
                    h - 1,
                    &self.hints(),
                    Style::default().fg(MUTED_DARK),
                ),
            }
        }
    }

    fn title(&self) -> String {
        match self.board() {
            Some(d) => format!("Issue board [{}/{}]", self.col + 1, d.columns.len()),
            None => "Issue board".to_string(),
        }
    }

    fn status_line(&self) -> Option<(String, Style)> {
        match self.mode {
            Mode::Confirm(p) => {
                let data = self.board()?;
                let issue = data.columns.get(p.from)?.issues.get(p.issue)?;
                let text = format!(
                    "Move {} from {} to {}? y/n",
                    clean(&issue.identifier),
                    clean(&data.columns[p.from].name),
                    clean(&data.columns[p.to].name)
                );
                Some((text, Style::default().fg(WARN).bold()))
            }
            Mode::Moving(p) => {
                let data = self.board()?;
                let text = format!("Moving: target {}", clean(&data.columns.get(p.to)?.name));
                Some((text, Style::default().fg(ACCENT)))
            }
            Mode::Browse => self
                .notice
                .as_ref()
                .map(|n| (clean(n), Style::default().fg(WARN))),
        }
    }

    fn hints(&self) -> String {
        match self.mode {
            Mode::Confirm(_) => "y confirm  n cancel".to_string(),
            Mode::Moving(_) => "Left/Right target  Enter move  Esc cancel".to_string(),
            Mode::Browse => match self.board() {
                Some(d) if d.read_only => {
                    "Arrows move  r refresh  read-only  Esc close".to_string()
                }
                Some(_) => "Arrows move  m move  r refresh  Esc close".to_string(),
                None => "r refresh  Esc close".to_string(),
            },
        }
    }

    fn render_body(&self, buf: &mut Buffer, area: Rect, body_h: u16) {
        // Rows 1 (scope) and 2 (tabs / state message).
        let (message, style) = match &self.load {
            BoardLoad::Loading => (Some("Loading issue board...".to_string()), MUTED),
            BoardLoad::Empty => (Some("No issues on this board".to_string()), MUTED),
            BoardLoad::Unavailable(r) => (Some(format!("Board unavailable: {}", clean(r))), WARN),
            BoardLoad::Unauthorized(r) => (Some(format!("Not authorized: {}", clean(r))), WARN),
            BoardLoad::Error(r) => (Some(format!("Board query failed: {}", clean(r))), ERROR),
            BoardLoad::Ready(d) if !has_issues(d) => {
                put(
                    buf,
                    area,
                    1,
                    &clean(&d.scope_label),
                    Style::default().fg(MUTED),
                );
                (Some("No issues on this board".to_string()), MUTED)
            }
            BoardLoad::Ready(_) => (None, MUTED),
        };
        if let Some(text) = message {
            put(buf, area, 2, &text, Style::default().fg(style));
            return;
        }
        let Some(data) = self.board() else { return };
        put(
            buf,
            area,
            1,
            &clean(&data.scope_label),
            Style::default().fg(MUTED),
        );

        let target = match self.mode {
            Mode::Moving(p) | Mode::Confirm(p) => Some(p),
            Mode::Browse => None,
        };
        self.render_tabs(buf, area, data, target.map(|p| p.to));

        let Some(column) = data.columns.get(self.col) else {
            return;
        };
        let id_w = column
            .issues
            .iter()
            .map(|i| clean(&i.identifier).width())
            .max()
            .unwrap_or(0)
            .clamp(2, MAX_ID_CELLS);
        let header = format!("  {}  TITLE", pad_cells("ID", id_w));
        put_filled(
            buf,
            area,
            TAB_ROW + 1,
            &header,
            Style::default().fg(MUTED_DARK),
        );
        if column.issues.is_empty() {
            put(
                buf,
                area,
                TOP_ROWS,
                "No issues in this state",
                Style::default().fg(MUTED),
            );
            return;
        }
        let top = (self.row + 1).saturating_sub(body_h as usize);
        for (i, issue) in column
            .issues
            .iter()
            .enumerate()
            .skip(top)
            .take(body_h as usize)
        {
            let y = TOP_ROWS + (i - top) as u16;
            let selected = i == self.row;
            let marker = if selected { "> " } else { "  " };
            let text = format!(
                "{marker}{}  {}",
                pad_cells(&truncate_cells(&clean(&issue.identifier), id_w), id_w),
                clean(&issue.title)
            );
            let style = if selected {
                Style::default().fg(Color::White).bg(SELECTED_BG)
            } else {
                Style::default().fg(MUTED)
            };
            put_filled(buf, area, y, &text, style);
        }
    }

    /// One row of state tabs, each `Name (count)`. The strip scrolls so the
    /// selected tab is always visible; `‹`/`›` mark hidden tabs.
    fn render_tabs(&self, buf: &mut Buffer, area: Rect, data: &BoardData, target: Option<usize>) {
        let width = area.width as usize;
        let labels: Vec<String> = data
            .columns
            .iter()
            .map(|c| format!(" {} ({}) ", clean(&c.name), c.issues.len()))
            .collect();
        let widths: Vec<usize> = labels.iter().map(|l| l.width()).collect();
        // First tab to draw: the earliest start from which the selected tab
        // still fits within the width.
        let mut start = 0;
        while start < self.col && widths[start..=self.col].iter().sum::<usize>() > width {
            start += 1;
        }
        let mut x = 0usize;
        for (idx, label) in labels.iter().enumerate().skip(start) {
            if x >= width {
                break;
            }
            let mut style = Style::default().fg(MUTED);
            if data.columns[idx].issues.is_empty() {
                style = Style::default().fg(MUTED_DARK);
            }
            if idx == self.col {
                style = Style::default().fg(ACCENT).bg(SELECTED_BG).bold();
            }
            if target == Some(idx) {
                style = Style::default().fg(Color::White).bg(TARGET_BG).bold();
            }
            let cell = Rect::new(area.x + x as u16, area.y, (width - x) as u16, area.height);
            let text = truncate_cells(label, width - x);
            let drawn = text.width();
            buf.set_stringn(cell.x, cell.y + TAB_ROW, &text, cell.width as usize, style);
            x += drawn;
        }
        if start > 0 {
            buf.set_stringn(area.x, area.y + TAB_ROW, "‹", 1, Style::default().fg(WARN));
        }
        if x >= width && start + 1 < labels.len() && widths[start..].iter().sum::<usize>() > width {
            buf.set_stringn(
                area.x + area.width - 1,
                area.y + TAB_ROW,
                "›",
                1,
                Style::default().fg(WARN),
            );
        }
    }
}

fn has_issues(data: &BoardData) -> bool {
    data.columns.iter().any(|c| !c.issues.is_empty())
}

/// Next target column in a direction, never landing on the source and
/// clamped to the board edges.
fn step_target(
    current: usize,
    source: usize,
    columns: usize,
    forward: bool,
    allowed: &dyn Fn(usize) -> bool,
) -> usize {
    let mut next = current;
    loop {
        let candidate = if forward {
            (next + 1).min(columns.saturating_sub(1))
        } else {
            next.saturating_sub(1)
        };
        if candidate == next {
            return current;
        }
        if candidate != source && allowed(candidate) {
            return candidate;
        }
        next = candidate;
    }
}

/// Replace control characters so one logical row stays one visual row.
fn clean(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Truncate to `width` terminal cells, ending in `…` when cut.
fn truncate_cells(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w + 1 > width {
            break;
        }
        used += w;
        out.push(ch);
    }
    out.push('…');
    out
}

/// Pad with spaces to `width` terminal cells (no truncation).
fn pad_cells(text: &str, width: usize) -> String {
    let used = text.width();
    format!("{text}{}", " ".repeat(width.saturating_sub(used)))
}

/// Draw one truncated row at `y` within `area`.
fn put(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    if y >= area.height {
        return;
    }
    let text = truncate_cells(text, area.width as usize);
    buf.set_stringn(area.x, area.y + y, &text, area.width as usize, style);
}

/// Like [`put`], but also paints the background across the whole cell width.
fn put_filled(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    if y >= area.height || area.width == 0 {
        return;
    }
    buf.set_style(Rect::new(area.x, area.y + y, area.width, 1), style);
    put(buf, area, y, text, style);
}

#[cfg(test)]
#[path = "board_overlay_tests.rs"]
mod tests;
