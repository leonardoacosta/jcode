//! World issue board: a pure-presentation overlay with one tab per workflow
//! state (count badge) and a table of that state's issues.
//!
//! The overlay owns no tracker logic. A caller hands it a [`BoardLoad`] and
//! reads back [`BoardAction`]s (close, refresh, confirmed move). Every chrome
//! row is a single visual row truncated by Unicode display-cell width.

use crossterm::event::{KeyCode, KeyModifiers};
use jcode_tui_style::{
    Role, rgb, role_color,
    theme::{accent_color, ai_text, dim_color, file_link_color, pending_color, queued_color},
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Row of the state tab strip.
const TAB_ROW: u16 = 2;
/// Widest identifier column in the issue table.
const MAX_ID_CELLS: usize = 14;
/// Rows used by chrome above the body (title, scope, tabs, table header).
/// When framed, row 0 is the top border carrying the title.
const TOP_ROWS: u16 = 4;
/// Rows used by chrome below the body (status line, hints). When framed, the
/// last row is the bottom border carrying the hints.
const BOTTOM_ROWS: u16 = 2;
/// Narrower areas skip the border and use the frameless layout.
const MIN_FRAME_WIDTH: u16 = 24;
/// Shorter areas skip the border (frame plus at least three body rows).
const MIN_FRAME_HEIGHT: u16 = TOP_ROWS + BOTTOM_ROWS + 3;

/// A run of text with one style. A row is a list of these.
type Seg = (String, Style);

fn panel_bg() -> Color {
    rgb(18, 18, 26)
}

fn seg(text: impl Into<String>, style: Style) -> Seg {
    (text.into(), style)
}

fn dim() -> Style {
    Style::default().fg(dim_color())
}

fn accent() -> Style {
    Style::default().fg(accent_color())
}

fn selection_bg() -> Color {
    role_color(Role::SelectionBg)
}

/// Color of the status-kind dot shown before a state name.
fn kind_dot_color(kind: &str) -> Color {
    match kind {
        "triage" => pending_color(),
        "unstarted" => role_color(Role::Info),
        "started" => queued_color(),
        "completed" => role_color(Role::Success),
        // backlog, canceled, cancelled, duplicate and unknown kinds
        _ => dim_color(),
    }
}

fn kind_dot(kind: &str) -> Seg {
    seg("●", Style::default().fg(kind_dot_color(kind)))
}

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
    /// Provider status type, lowercase (`triage`, `backlog`, `unstarted`,
    /// `started`, `completed`, `canceled`, `duplicate`). Empty when unknown.
    pub kind: String,
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

/// How a requested move ended, as far as the tracker can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveVerdict {
    /// The tracker applied it.
    Moved,
    /// Nothing was written (refused, conflict, failure): undo the optimistic card.
    NotMoved,
    /// The write may or may not have landed: keep the card, mark it unconfirmed.
    Unknown,
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

/// A move already shown on the board while the tracker call is in flight.
#[derive(Debug)]
struct InFlight {
    identifier: String,
    to_name: String,
    /// The board as it was before the optimistic update.
    rollback: BoardData,
}

#[derive(Debug)]
pub struct BoardOverlay {
    load: BoardLoad,
    col: usize,
    row: usize,
    mode: Mode,
    notice: Option<String>,
    notice_ok: bool,
    in_flight: Option<InFlight>,
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
            notice_ok: false,
            in_flight: None,
            anchor: None,
        };
        overlay.set_load(load);
        overlay
    }

    /// Show a one-row notice (for example a move result) until the next
    /// `set_load` or key press clears it.
    pub fn set_notice(&mut self, text: impl Into<String>) {
        self.notice = Some(text.into());
        self.notice_ok = false;
    }

    /// The tracker call for an optimistic move finished. `fresh` is the
    /// re-read board when it succeeded and is the truth. Without it, the
    /// verdict decides: undo a move known not to have happened, keep one that
    /// happened or may have.
    pub fn finish_move(&mut self, verdict: MoveVerdict, text: &str, fresh: Option<BoardLoad>) {
        let flight = self.in_flight.take();
        let label = flight.as_ref().map(|f| clean(&f.identifier));
        let with_label = |t: &str| match &label {
            Some(l) => format!("{l}: {t}"),
            None => t.to_string(),
        };
        let ok = verdict == MoveVerdict::Moved;
        let notice = match fresh {
            Some(load) => {
                self.set_load(load);
                with_label(text)
            }
            None => match (verdict, flight) {
                (MoveVerdict::NotMoved, Some(f)) => {
                    self.set_load(BoardLoad::Ready(f.rollback));
                    with_label(text)
                }
                (MoveVerdict::Unknown, _) => {
                    with_label(&format!("{text}; shown as moved, unconfirmed"))
                }
                _ => with_label(&format!("{text}; board not refreshed, press r")),
            },
        };
        self.notice = Some(notice);
        self.notice_ok = ok;
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
        self.in_flight = None;
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
            KeyCode::Char('r') if self.in_flight.is_some() => {
                self.notice = Some("a move is in progress".to_string());
                self.notice_ok = false;
                return BoardAction::None;
            }
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
        if self.in_flight.is_some() {
            self.notice = Some("a move is in progress".to_string());
            return;
        }
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
        let Some(data) = self.board() else { return };
        if data.columns[self.col].id.is_empty() {
            self.notice = Some("Other column is display-only".to_string());
            return;
        }
        let order = picker_order(data);
        let Some(at) = order.iter().position(|i| *i == self.col) else {
            return;
        };
        let Some(to) = order
            .get(at + 1)
            .or_else(|| at.checked_sub(1).and_then(|p| order.get(p)))
            .copied()
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
        match code {
            KeyCode::Esc => self.mode = Mode::Browse,
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => {
                let forward = matches!(code, KeyCode::Down | KeyCode::Char('j'));
                if let Some(data) = self.board() {
                    let targets: Vec<usize> = picker_order(data)
                        .into_iter()
                        .filter(|i| *i != pending.from)
                        .collect();
                    if let Some(at) = targets.iter().position(|i| *i == pending.to) {
                        let next = if forward {
                            (at + 1).min(targets.len() - 1)
                        } else {
                            at.saturating_sub(1)
                        };
                        pending.to = targets[next];
                        self.mode = Mode::Moving(pending);
                    }
                }
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
                self.begin_move(pending)
                    .map_or(BoardAction::None, BoardAction::Confirmed)
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = Mode::Browse;
                BoardAction::None
            }
            _ => BoardAction::None,
        }
    }

    /// Show the move on the board at once and remember how to undo it. The
    /// caller then performs the tracker call and reports back through
    /// [`BoardOverlay::finish_move`].
    fn begin_move(&mut self, pending: PendingMove) -> Option<MoveRequest> {
        let request = self.move_request(pending)?;
        let rollback = self.board()?.clone();
        let BoardLoad::Ready(data) = &mut self.load else {
            return None;
        };
        let issue = data.columns[pending.from].issues.remove(pending.issue);
        let flight = InFlight {
            identifier: issue.identifier.clone(),
            to_name: data.columns[pending.to].name.clone(),
            rollback,
        };
        data.columns[pending.to].issues.insert(0, issue);
        self.in_flight = Some(flight);
        self.notice = None;
        self.clamp_selection();
        Some(request)
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
        buf.set_style(area, Style::default().bg(panel_bg()));

        let framed = area.width >= MIN_FRAME_WIDTH && area.height >= MIN_FRAME_HEIGHT;
        // Row numbers below are relative to `content`. Framed, its first and
        // last rows are the border rows (title and hints); its columns exclude
        // the side borders plus one padding column on each side. Frameless,
        // it is the whole area.
        let content = if framed {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(role_color(Role::Border)))
                .style(Style::default().bg(panel_bg()))
                .render(area, buf);
            Rect::new(area.x + 2, area.y, area.width - 4, area.height)
        } else {
            area
        };

        let h = content.height;
        let full = h >= TOP_ROWS + BOTTOM_ROWS;
        if framed {
            self.render_frame_title(buf, content);
        } else {
            put(buf, content, 0, &self.title(), accent().bold());
        }

        if full {
            self.render_body(buf, content, h - TOP_ROWS - BOTTOM_ROWS);
        }
        // On terminals too short for the full layout the last row carries the
        // status line when there is one, otherwise the hints.
        let status = self.status_line();
        if full {
            if let Some((text, style)) = &status {
                put(buf, content, h - 2, text, *style);
            }
            self.render_hints(buf, content, h - 1, framed);
        } else if h > 1 {
            match &status {
                Some((text, style)) => put(buf, content, h - 1, text, *style),
                None => self.render_hints(buf, content, h - 1, false),
            }
        }
    }

    /// Frameless title row text.
    fn title(&self) -> String {
        match self.board() {
            Some(d) => format!("Issue board [{}/{}]", self.col + 1, d.columns.len()),
            None => "Issue board".to_string(),
        }
    }

    /// Title and column position embedded in the top border row.
    fn render_frame_title(&self, buf: &mut Buffer, content: Rect) {
        let width = content.width as usize;
        let title = seg(" ◆ Issue board ", accent().bold());
        let position = self
            .board()
            .map(|d| format!(" {}/{} ", self.col + 1, d.columns.len()));
        let right_w = position.as_ref().map_or(0, |p| p.width());
        let right = position.filter(|_| title.0.width() + right_w <= width);
        let used = title.0.width().min(width);
        put_segs(buf, content.x, content.y, width, &[title]);
        if let Some(p) = right {
            let x = content.x + (width - p.width()) as u16;
            if (x - content.x) as usize >= used {
                put_segs(buf, x, content.y, p.width(), &[seg(p, dim())]);
            }
        }
    }

    fn render_hints(&self, buf: &mut Buffer, content: Rect, y: u16, framed: bool) {
        let mut segs: Vec<Seg> = Vec::new();
        if framed {
            segs.push(seg(" ", Style::default()));
        }
        for (i, (key, label)) in self.hints().iter().enumerate() {
            if i > 0 {
                segs.push(seg(" · ", dim()));
            }
            if !key.is_empty() {
                segs.push(seg(*key, accent().bold()));
                segs.push(seg(" ", dim()));
            }
            segs.push(seg(*label, dim()));
        }
        if framed {
            segs.push(seg(" ", Style::default()));
        }
        put_segs(buf, content.x, content.y + y, content.width as usize, &segs);
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
                Some((text, Style::default().fg(role_color(Role::Warning)).bold()))
            }
            Mode::Moving(p) => {
                let data = self.board()?;
                let text = format!("Move to: {}", clean(&data.columns.get(p.to)?.name));
                Some((text, accent()))
            }
            Mode::Browse => match (&self.in_flight, &self.notice) {
                (_, Some(n)) => {
                    let role = if self.notice_ok {
                        Role::Success
                    } else {
                        Role::Warning
                    };
                    Some((clean(n), Style::default().fg(role_color(role))))
                }
                (Some(f), None) => Some((
                    format!("Moving {} to {}…", clean(&f.identifier), clean(&f.to_name)),
                    accent(),
                )),
                (None, None) => None,
            },
        }
    }

    /// `(key, label)` pairs; an empty key is a plain label.
    fn hints(&self) -> Vec<(&'static str, &'static str)> {
        match self.mode {
            Mode::Confirm(_) => vec![("y", "confirm"), ("n", "cancel")],
            Mode::Moving(_) => vec![
                ("↑↓", "choose status"),
                ("Enter", "select"),
                ("Esc", "cancel"),
            ],
            Mode::Browse => match self.board() {
                Some(d) if d.read_only => vec![
                    ("←→", "state"),
                    ("↑↓", "move"),
                    ("r", "refresh"),
                    ("", "read-only"),
                    ("Esc", "close"),
                ],
                Some(_) => vec![
                    ("←→", "state"),
                    ("↑↓", "move"),
                    ("m", "move status"),
                    ("r", "refresh"),
                    ("Esc", "close"),
                ],
                None => vec![("r", "refresh"), ("Esc", "close")],
            },
        }
    }

    fn render_body(&self, buf: &mut Buffer, area: Rect, body_h: u16) {
        // Rows 1 (scope) and 2 (tabs / state message).
        let warning = role_color(Role::Warning);
        let (message, color) = match &self.load {
            BoardLoad::Loading => (Some("Loading issue board...".to_string()), dim_color()),
            BoardLoad::Empty => (Some("No issues on this board".to_string()), dim_color()),
            BoardLoad::Unavailable(r) => {
                (Some(format!("Board unavailable: {}", clean(r))), warning)
            }
            BoardLoad::Unauthorized(r) => (Some(format!("Not authorized: {}", clean(r))), warning),
            BoardLoad::Error(r) => (
                Some(format!("Board query failed: {}", clean(r))),
                role_color(Role::Error),
            ),
            BoardLoad::Ready(d) if !has_issues(d) => {
                put(buf, area, 1, &clean(&d.scope_label), dim());
                (Some("No issues on this board".to_string()), dim_color())
            }
            BoardLoad::Ready(_) => (None, dim_color()),
        };
        if let Some(text) = message {
            put(buf, area, 2, &text, Style::default().fg(color));
            return;
        }
        let Some(data) = self.board() else { return };
        put(buf, area, 1, &clean(&data.scope_label), dim());

        if let Mode::Moving(p) | Mode::Confirm(p) = self.mode {
            render_picker(buf, area, data, p, body_h);
            return;
        }
        self.render_tabs(buf, area, data);

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
        put_filled(buf, area, TAB_ROW + 1, &header, dim());
        if column.issues.is_empty() {
            put(buf, area, TOP_ROWS, "No issues in this state", dim());
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
            let segs = [
                if selected {
                    seg("▸ ", accent())
                } else {
                    seg("  ", Style::default())
                },
                seg(
                    pad_cells(&truncate_cells(&clean(&issue.identifier), id_w), id_w),
                    Style::default().fg(file_link_color()),
                ),
                seg("  ", Style::default()),
                seg(clean(&issue.title), Style::default().fg(ai_text())),
            ];
            let fill = selected.then(|| Style::default().bg(selection_bg()));
            put_row(buf, area, y, &segs, fill);
        }
    }

    /// One row of state tabs, each `● Name (count)`. The strip scrolls so the
    /// selected tab is always visible; `‹`/`›` mark hidden tabs.
    fn render_tabs(&self, buf: &mut Buffer, area: Rect, data: &BoardData) {
        let width = area.width as usize;
        let tabs: Vec<Vec<Seg>> = data
            .columns
            .iter()
            .enumerate()
            .map(|(idx, c)| tab_segs(c, idx == self.col))
            .collect();
        let widths: Vec<usize> = tabs.iter().map(|t| segs_width(t)).collect();
        // First tab to draw: the earliest start from which the selected tab
        // still fits within the width.
        let mut start = 0;
        while start < self.col && widths[start..=self.col].iter().sum::<usize>() > width {
            start += 1;
        }
        let y = area.y + TAB_ROW;
        let mut x = 0usize;
        for tab in tabs.iter().skip(start) {
            if x >= width {
                break;
            }
            x += put_segs(buf, area.x + x as u16, y, width - x, tab);
        }
        if start > 0 {
            buf.set_stringn(area.x, y, "‹", 1, dim());
        }
        if x >= width && start + 1 < tabs.len() && widths[start..].iter().sum::<usize>() > width {
            buf.set_stringn(area.x + area.width - 1, y, "›", 1, dim());
        }
    }
}

/// One tab label: ` ● Name (count) `. Only the name is highlighted when
/// selected, so the dot keeps the status-kind color.
fn tab_segs(column: &BoardColumn, selected: bool) -> Vec<Seg> {
    let name_style = if selected {
        accent().bold().underlined()
    } else if column.issues.is_empty() {
        dim()
    } else {
        Style::default().fg(ai_text())
    };
    vec![
        seg(" ", Style::default()),
        kind_dot(&column.kind),
        seg(" ", Style::default()),
        seg(clean(&column.name), name_style),
        seg(format!(" ({}) ", column.issues.len()), dim()),
    ]
}

fn has_issues(data: &BoardData) -> bool {
    data.columns.iter().any(|c| !c.issues.is_empty())
}

/// Sort key of a status type: the order groups appear in the move dialog.
fn kind_rank(kind: &str) -> u8 {
    match kind {
        "triage" => 0,
        "backlog" => 1,
        "unstarted" => 2,
        "started" => 3,
        "completed" => 4,
        "canceled" | "cancelled" | "duplicate" => 5,
        _ => 6,
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind_rank(kind) {
        0 => "Triage",
        1 => "Backlog",
        2 => "Unstarted",
        3 => "Started",
        4 => "Completed",
        5 => "Canceled",
        _ => "Other statuses",
    }
}

/// Real columns (never the synthetic "Other") in the order the move dialog
/// lists them: by status type, then provider order.
fn picker_order(data: &BoardData) -> Vec<usize> {
    let mut order: Vec<usize> = (0..data.columns.len())
        .filter(|i| !data.columns[*i].id.is_empty())
        .collect();
    order.sort_by_key(|i| (kind_rank(&data.columns[*i].kind), *i));
    order
}

/// The move dialog: statuses grouped by type, the target highlighted and the
/// issue's current status shown dim. Replaces the tab strip and table.
fn render_picker(buf: &mut Buffer, area: Rect, data: &BoardData, p: PendingMove, body_h: u16) {
    let issue = data.columns.get(p.from).and_then(|c| c.issues.get(p.issue));
    let head_style = Style::default().fg(ai_text()).bold();
    let head = match issue {
        Some(i) => vec![
            seg("Move ", head_style),
            seg(
                clean(&i.identifier),
                Style::default().fg(file_link_color()).bold(),
            ),
            seg(format!("  {}", clean(&i.title)), head_style),
        ],
        None => vec![seg("Move issue", head_style)],
    };
    put_row(buf, area, 2, &head, Some(Style::default()));
    enum Line {
        Group(&'static str),
        Status(usize),
    }
    let mut lines = Vec::new();
    let mut last = None;
    for idx in picker_order(data) {
        let label = kind_label(&data.columns[idx].kind);
        if last != Some(label) {
            lines.push(Line::Group(label));
            last = Some(label);
        }
        lines.push(Line::Status(idx));
    }
    let selected = lines
        .iter()
        .position(|l| matches!(l, Line::Status(i) if *i == p.to))
        .unwrap_or(0);
    let top = (selected + 1).saturating_sub(body_h as usize);
    let plain = Style::default();
    for (n, line) in lines.iter().enumerate().skip(top).take(body_h as usize) {
        let y = TOP_ROWS - 1 + (n - top) as u16;
        match line {
            Line::Group(label) => put(buf, area, y, &label.to_uppercase(), dim()),
            Line::Status(idx) => {
                let column = &data.columns[*idx];
                let name = clean(&column.name);
                let count = column.issues.len();
                let dot = kind_dot(&column.kind);
                if *idx == p.from {
                    let segs = [
                        seg("    ", plain),
                        dot,
                        seg(format!(" {name} ({count})  current"), dim()),
                    ];
                    put_row(buf, area, y, &segs, None);
                } else if *idx == p.to {
                    let segs = [
                        seg("  ", plain),
                        seg("▸", accent()),
                        seg(" ", plain),
                        dot,
                        seg(" ", plain),
                        seg(name, accent().bold()),
                        seg(format!(" ({count})"), Style::default().fg(ai_text())),
                    ];
                    let fill = Style::default().bg(selection_bg());
                    put_row(buf, area, y, &segs, Some(fill));
                } else {
                    let segs = [
                        seg("    ", plain),
                        dot,
                        seg(" ", plain),
                        seg(name, Style::default().fg(ai_text())),
                        seg(format!(" ({count})"), dim()),
                    ];
                    put_row(buf, area, y, &segs, None);
                }
            }
        }
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

fn segs_width(segs: &[Seg]) -> usize {
    segs.iter().map(|(t, _)| t.width()).sum()
}

/// Draw styled runs at `(x, y)` within `width` cells. When the runs do not
/// fit, the tail is cut and replaced by `…` in the style of the last run
/// drawn. Returns the cells used.
fn put_segs(buf: &mut Buffer, x: u16, y: u16, width: usize, segs: &[Seg]) -> usize {
    if width == 0 {
        return 0;
    }
    let cut = segs_width(segs) > width;
    let limit = if cut { width - 1 } else { width };
    let mut used = 0;
    let mut last_style = Style::default();
    for (text, style) in segs {
        if used >= limit {
            break;
        }
        let mut part = String::new();
        let mut part_w = 0;
        for ch in text.chars() {
            let w = ch.width().unwrap_or(0);
            if used + part_w + w > limit {
                break;
            }
            part_w += w;
            part.push(ch);
        }
        buf.set_stringn(x + used as u16, y, &part, part_w, *style);
        used += part_w;
        last_style = *style;
    }
    if cut {
        buf.set_stringn(x + used as u16, y, "…", 1, last_style);
        used += 1;
    }
    used
}

/// Draw one row of styled runs at `y` within `area`, optionally filling the
/// whole row with `fill` first.
fn put_row(buf: &mut Buffer, area: Rect, y: u16, segs: &[Seg], fill: Option<Style>) {
    if y >= area.height || area.width == 0 {
        return;
    }
    if let Some(fill) = fill {
        buf.set_style(Rect::new(area.x, area.y + y, area.width, 1), fill);
    }
    put_segs(buf, area.x, area.y + y, area.width as usize, segs);
}

/// Draw one truncated row at `y` within `area`.
fn put(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    put_row(buf, area, y, &[seg(text, style)], None);
}

/// Like [`put`], but also paints the background across the whole cell width.
fn put_filled(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    put_row(buf, area, y, &[seg(text, style)], Some(style));
}

#[cfg(test)]
#[path = "board_overlay_tests.rs"]
mod tests;
