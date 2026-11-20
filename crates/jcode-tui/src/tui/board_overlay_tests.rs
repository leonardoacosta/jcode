use super::*;
use ratatui::{Terminal, backend::TestBackend};

const NO: KeyModifiers = KeyModifiers::NONE;

fn issue(id: &str, title: &str) -> BoardIssue {
    BoardIssue {
        id: format!("id-{id}"),
        identifier: id.to_string(),
        title: title.to_string(),
    }
}

fn column(id: &str, name: &str, issues: Vec<BoardIssue>) -> BoardColumn {
    let kind = match id {
        "c-todo" => "unstarted",
        "c-prog" => "started",
        "c-done" => "completed",
        _ => "",
    };
    kind_column(id, name, kind, issues)
}

fn kind_column(id: &str, name: &str, kind: &str, issues: Vec<BoardIssue>) -> BoardColumn {
    BoardColumn {
        id: id.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
        issues,
    }
}

fn data(read_only: bool) -> BoardData {
    BoardData {
        scope_label: "Team Core (team-wide)".to_string(),
        read_only,
        columns: vec![
            column(
                "c-todo",
                "Todo",
                vec![issue("TEAM-1", "First"), issue("TEAM-2", "Second")],
            ),
            column("c-prog", "In Progress", vec![issue("TEAM-3", "Third")]),
            column("c-done", "Done", vec![]),
        ],
    }
}

fn ready(read_only: bool) -> BoardOverlay {
    BoardOverlay::new(BoardLoad::Ready(data(read_only)))
}

fn key(o: &mut BoardOverlay, code: KeyCode) -> BoardAction {
    o.handle_key(code, NO)
}

fn ch(o: &mut BoardOverlay, c: char) -> BoardAction {
    o.handle_key(KeyCode::Char(c), NO)
}

fn rows(o: &BoardOverlay, w: u16, h: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal.draw(|f| o.render(f)).unwrap();
    let buf = terminal.backend().buffer().clone();
    // A wide glyph occupies its own cell plus continuation cells holding
    // filler; skip those so the row text has the true display width.
    (0..h)
        .map(|y| {
            let mut row = String::new();
            let mut x = 0;
            while x < w {
                let symbol = buf[(x, y)].symbol();
                row.push_str(symbol);
                x += (unicode_width::UnicodeWidthStr::width(symbol) as u16).max(1);
            }
            row
        })
        .collect()
}

fn text(o: &BoardOverlay, w: u16, h: u16) -> String {
    rows(o, w, h).join("\n")
}

/// A row's content without the frame border, selection marker or status dot.
fn bare(row: &str) -> &str {
    row.trim_matches(|c: char| c == '│' || c == '╭' || c == '╮' || c.is_whitespace())
        .trim_start_matches(['▸', '●', ' '])
}

/// Visible content of a row with trailing blanks removed.
fn trimmed(row: &str) -> &str {
    row.trim_end()
}

#[test]
fn board_overlay_loading_state_has_no_columns() {
    let o = BoardOverlay::new(BoardLoad::Loading);
    let t = text(&o, 60, 10);
    assert!(t.contains("Loading"), "{t}");
    assert!(!t.contains("Todo"));
}

#[test]
fn board_overlay_empty_state_is_distinct() {
    let o = BoardOverlay::new(BoardLoad::Empty);
    let t = text(&o, 60, 10);
    assert!(t.contains("No issues"), "{t}");
    assert!(!t.contains("Loading"));
}

#[test]
fn board_overlay_ready_without_issues_shows_empty_state() {
    let mut d = data(false);
    for c in &mut d.columns {
        c.issues.clear();
    }
    let o = BoardOverlay::new(BoardLoad::Ready(d));
    let t = text(&o, 60, 10);
    assert!(t.contains("No issues"), "{t}");
    assert!(t.contains("team-wide"), "{t}");
}

#[test]
fn board_overlay_unavailable_unauthorized_error_show_reason() {
    let cases = [
        (
            BoardLoad::Unavailable("unknown world".into()),
            "unavailable",
            "unknown world",
        ),
        (
            BoardLoad::Unavailable("Brown boards require ADO".into()),
            "unavailable",
            "require ADO",
        ),
        (
            BoardLoad::Unavailable("ADO adapter missing".into()),
            "unavailable",
            "adapter missing",
        ),
        (
            BoardLoad::Unauthorized("token rejected".into()),
            "authorized",
            "token rejected",
        ),
        (BoardLoad::Error("HTTP 500".into()), "failed", "HTTP 500"),
    ];
    for (load, label, reason) in cases {
        let o = BoardOverlay::new(load);
        let t = text(&o, 70, 10);
        assert!(
            t.contains(label) && t.contains(reason),
            "{label} {reason}\n{t}"
        );
    }
}

#[test]
fn board_overlay_no_stale_columns_after_loading_or_error() {
    for load in [
        BoardLoad::Loading,
        BoardLoad::Error("boom".into()),
        BoardLoad::Unauthorized("no".into()),
        BoardLoad::Unavailable("no".into()),
    ] {
        let mut o = ready(false);
        o.set_load(load);
        let t = text(&o, 60, 10);
        assert!(!t.contains("TEAM-1") && !t.contains("Todo"), "{t}");
    }
}

#[test]
fn board_overlay_tabs_show_states_in_order_with_count_badges() {
    let o = ready(false);
    let r = rows(&o, 60, 10);
    let tabs = &r[2];
    let todo = tabs.find("Todo (2)").unwrap();
    let prog = tabs.find("In Progress (1)").unwrap();
    let done = tabs.find("Done (0)").unwrap();
    assert!(todo < prog && prog < done, "{tabs}");
}

#[test]
fn board_overlay_table_lists_only_the_selected_state_with_full_titles() {
    let mut o = ready(false);
    let r = rows(&o, 60, 10);
    assert!(r[3].contains("ID") && r[3].contains("TITLE"), "{:?}", r[3]);
    assert!(
        r[4].contains("▸ TEAM-1") && r[4].contains("First"),
        "{:?}",
        r[4]
    );
    assert!(r[5].contains("TEAM-2") && r[5].contains("Second"));
    assert!(!text(&o, 60, 10).contains("TEAM-3"), "other state hidden");
    ch(&mut o, 'l');
    let t = text(&o, 60, 10);
    assert!(t.contains("TEAM-3") && t.contains("Third"), "{t}");
    assert!(!t.contains("TEAM-1"));
}

#[test]
fn board_overlay_long_title_gets_the_whole_row() {
    let mut d = data(false);
    d.columns[0].issues[0].title = "Reconcile the usage budget accounting across providers".into();
    let o = BoardOverlay::new(BoardLoad::Ready(d));
    let r = rows(&o, 80, 10);
    assert!(
        r[4].contains("Reconcile the usage budget accounting across providers"),
        "{:?}",
        r[4]
    );
}

#[test]
fn board_overlay_empty_state_says_so_under_its_tab() {
    let mut o = ready(false);
    ch(&mut o, 'l');
    ch(&mut o, 'l');
    let r = rows(&o, 60, 10);
    assert!(r[2].contains("Done (0)"));
    assert!(r[4].contains("No issues in this state"), "{:?}", r[4]);
}

#[test]
fn board_overlay_scope_label_is_one_row() {
    let o = ready(false);
    let r = rows(&o, 60, 10);
    assert!(r[1].contains("Team Core (team-wide)"));
}

#[test]
fn board_overlay_selection_moves_and_clamps() {
    let mut o = ready(false);
    key(&mut o, KeyCode::Left);
    assert_eq!((o.col, o.row), (0, 0));
    key(&mut o, KeyCode::Up);
    assert_eq!(o.row, 0);
    ch(&mut o, 'j');
    ch(&mut o, 'j');
    ch(&mut o, 'j');
    assert_eq!(o.row, 1, "clamped to last issue");
    ch(&mut o, 'l');
    assert_eq!((o.col, o.row), (1, 0), "row re-clamped in shorter column");
    key(&mut o, KeyCode::Right);
    key(&mut o, KeyCode::Right);
    assert_eq!(o.col, 2, "clamped to last column");
    key(&mut o, KeyCode::Down);
    assert_eq!(o.row, 0, "empty column keeps row 0");
    ch(&mut o, 'h');
    ch(&mut o, 'h');
    ch(&mut o, 'h');
    assert_eq!(o.col, 0);
    ch(&mut o, 'k');
    assert_eq!(o.row, 0);
}

#[test]
fn board_overlay_selection_valid_after_set_load() {
    let mut o = ready(false);
    ch(&mut o, 'j');
    assert_eq!(o.row, 1);
    let mut smaller = data(false);
    smaller.columns[0].issues.truncate(1);
    o.set_load(BoardLoad::Ready(smaller));
    assert_eq!((o.col, o.row), (0, 0));

    // Column selection follows the column id across a reload/Loading round trip.
    ch(&mut o, 'l');
    o.set_load(BoardLoad::Loading);
    let mut reordered = data(false);
    reordered.columns.reverse();
    o.set_load(BoardLoad::Ready(reordered));
    assert_eq!(o.col, 1);

    let mut fewer = data(false);
    fewer.columns.truncate(1);
    o.set_load(BoardLoad::Ready(fewer));
    assert_eq!(o.col, 0);
}

#[test]
fn board_overlay_esc_closes_without_other_effect() {
    let mut o = ready(false);
    assert_eq!(key(&mut o, KeyCode::Esc), BoardAction::Close);
    let mut o = BoardOverlay::new(BoardLoad::Loading);
    assert_eq!(key(&mut o, KeyCode::Esc), BoardAction::Close);
    let mut o = BoardOverlay::new(BoardLoad::Error("x".into()));
    assert_eq!(key(&mut o, KeyCode::Esc), BoardAction::Close);
}

#[test]
fn board_overlay_refresh_key_in_every_state() {
    for load in [
        BoardLoad::Loading,
        BoardLoad::Empty,
        BoardLoad::Error("x".into()),
        BoardLoad::Ready(data(false)),
    ] {
        let mut o = BoardOverlay::new(load);
        assert_eq!(ch(&mut o, 'r'), BoardAction::Refresh);
    }
}

#[test]
fn board_overlay_move_flow_confirms_once() {
    let mut o = ready(false);
    assert_eq!(ch(&mut o, 'm'), BoardAction::None);
    assert!(matches!(o.mode, Mode::Moving(_)));
    key(&mut o, KeyCode::Down); // In Progress -> Done
    key(&mut o, KeyCode::Up); // and back
    assert_eq!(key(&mut o, KeyCode::Enter), BoardAction::None);
    let t = text(&o, 80, 10);
    assert!(
        t.contains("Move TEAM-1 from Todo to In Progress? y/n"),
        "{t}"
    );
    // Unrelated keys do nothing and keep the prompt.
    assert_eq!(ch(&mut o, 'x'), BoardAction::None);
    assert_eq!(key(&mut o, KeyCode::Enter), BoardAction::None);
    assert!(matches!(o.mode, Mode::Confirm(_)));
    let action = ch(&mut o, 'y');
    assert_eq!(
        action,
        BoardAction::Confirmed(MoveRequest {
            issue_id: "id-TEAM-1".into(),
            from_column_id: "c-todo".into(),
            to_column_id: "c-prog".into(),
        })
    );
    assert_eq!(
        ch(&mut o, 'y'),
        BoardAction::None,
        "second y is not a second move"
    );
}

#[test]
fn board_overlay_move_dialog_groups_statuses_by_type_in_order() {
    let mut d = data(false);
    d.columns = vec![
        kind_column("c-done", "Done", "completed", vec![]),
        kind_column("c-ready", "Ready", "unstarted", vec![issue("T-1", "a")]),
        kind_column("c-triage", "Triage", "triage", vec![]),
        kind_column("c-dup", "Duplicate", "duplicate", vec![]),
        kind_column("c-prog", "In Progress", "started", vec![]),
        kind_column("c-back", "Backlog", "backlog", vec![]),
        kind_column("c-canc", "Canceled", "canceled", vec![]),
        kind_column("c-todo", "Todo", "unstarted", vec![]),
    ];
    let mut o = BoardOverlay::new(BoardLoad::Ready(d));
    ch(&mut o, 'l'); // Ready is the second tab
    ch(&mut o, 'm');
    let r = rows(&o, 60, 24);
    // Group headers are whole uppercase rows; statuses start after the indent.
    let at = |needle: &str| {
        r.iter()
            .position(|l| bare(l) == needle || bare(l).starts_with(&format!("{needle} (")))
            .unwrap_or(999)
    };
    let order = [
        "TRIAGE",
        "Triage",
        "BACKLOG",
        "Backlog",
        "UNSTARTED",
        "Ready",
        "Todo",
        "STARTED",
        "In Progress",
        "COMPLETED",
        "Done",
        "CANCELED",
        // Same group keeps the provider order (Duplicate was listed first).
        "Duplicate",
        "Canceled",
    ];
    let rows_at: Vec<_> = order.iter().map(|n| at(n)).collect();
    assert!(
        rows_at.windows(2).all(|w| w[0] < w[1]),
        "{rows_at:?}\n{r:#?}"
    );
    assert!(r[2].contains("Move T-1"), "{:?}", r[2]);
    let ready = r.iter().find(|l| l.contains("Ready")).unwrap();
    assert!(ready.contains("current"), "own status marked: {ready:?}");
}

#[test]
fn board_overlay_move_dialog_never_offers_the_current_status_or_other() {
    let mut o = with_other();
    ch(&mut o, 'm');
    for _ in 0..8 {
        let Mode::Moving(p) = o.mode else { panic!() };
        assert_ne!(p.to, p.from);
        assert!(!o.board().unwrap().columns[p.to].id.is_empty());
        key(&mut o, KeyCode::Down);
    }
    let r = text(&o, 60, 14);
    assert!(!r.contains("Other ("), "{r}");
}

#[test]
fn board_overlay_move_target_never_the_source() {
    let mut o = ready(false);
    key(&mut o, KeyCode::Right); // select In Progress
    ch(&mut o, 'm');
    let Mode::Moving(p) = o.mode else { panic!() };
    assert_ne!(p.to, p.from);
    for code in [
        KeyCode::Up,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
        KeyCode::Down,
    ] {
        key(&mut o, code);
        let Mode::Moving(p) = o.mode else { panic!() };
        assert_ne!(p.to, p.from);
    }
}

#[test]
fn board_overlay_move_declines_with_n_or_esc() {
    for decline in [KeyCode::Char('n'), KeyCode::Esc, KeyCode::Char('N')] {
        let mut o = ready(false);
        ch(&mut o, 'm');
        key(&mut o, KeyCode::Enter);
        assert_eq!(key(&mut o, decline), BoardAction::None);
        assert!(matches!(o.mode, Mode::Browse));
        assert_eq!(ch(&mut o, 'y'), BoardAction::None, "no stale confirmation");
    }
}

#[test]
fn board_overlay_move_esc_cancels_pick_without_closing() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    assert_eq!(key(&mut o, KeyCode::Esc), BoardAction::None);
    assert!(matches!(o.mode, Mode::Browse));
}

#[test]
fn board_overlay_ctrl_y_does_not_confirm() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    key(&mut o, KeyCode::Enter);
    assert_eq!(
        o.handle_key(KeyCode::Char('y'), KeyModifiers::CONTROL),
        BoardAction::None
    );
    assert!(matches!(o.mode, Mode::Confirm(_)));
}

#[test]
fn board_overlay_read_only_never_prompts() {
    let mut o = ready(true);
    assert_eq!(ch(&mut o, 'm'), BoardAction::None);
    assert!(matches!(o.mode, Mode::Browse));
    let r = rows(&o, 70, 10);
    assert!(r.iter().any(|l| l.contains(READ_ONLY_NOTICE)), "{r:?}");
    assert!(!r.iter().any(|l| l.contains("y/n")));
    for c in ['y', 'n', 'm'] {
        assert_eq!(ch(&mut o, c), BoardAction::None);
        assert!(matches!(o.mode, Mode::Browse));
    }
    assert_eq!(key(&mut o, KeyCode::Enter), BoardAction::None);
}

#[test]
fn board_overlay_move_ignored_without_selected_issue_or_target() {
    let mut o = ready(false);
    key(&mut o, KeyCode::Right);
    key(&mut o, KeyCode::Right); // empty Done column
    ch(&mut o, 'm');
    assert!(matches!(o.mode, Mode::Browse));

    let one = BoardData {
        scope_label: "s".into(),
        read_only: false,
        columns: vec![column("a", "A", vec![issue("X-1", "t")])],
    };
    let mut o = BoardOverlay::new(BoardLoad::Ready(one));
    ch(&mut o, 'm');
    assert!(matches!(o.mode, Mode::Browse));
}

#[test]
fn board_overlay_set_load_cancels_pending_move() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    key(&mut o, KeyCode::Enter);
    o.set_load(BoardLoad::Ready(data(false)));
    assert!(matches!(o.mode, Mode::Browse));
    assert_eq!(ch(&mut o, 'y'), BoardAction::None);
}

#[test]
fn board_overlay_set_load_to_read_only_cannot_confirm() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    key(&mut o, KeyCode::Enter);
    o.set_load(BoardLoad::Ready(data(true)));
    assert_eq!(ch(&mut o, 'y'), BoardAction::None);
}

/// Every row's display width must fit the buffer and no row may spill: the
/// TestBackend buffer is exactly w cells wide, so check the *symbols* we put.
fn assert_rows_fit(o: &BoardOverlay, w: u16, h: u16) {
    use unicode_width::UnicodeWidthStr;
    for (y, row) in rows(o, w, h).iter().enumerate() {
        assert!(
            row.width() <= w as usize,
            "row {y} width {} > {w}: {row:?}",
            row.width()
        );
    }
}

#[test]
fn board_overlay_narrow_widths_keep_one_row_per_chrome_line() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    key(&mut o, KeyCode::Enter);
    for w in [1u16, 2, 5, 8, 20, 40] {
        for h in [1u16, 2, 4, 5, 6, 12] {
            assert_rows_fit(&o, w, h);
        }
    }
    // The confirm line is one row, truncated with an ellipsis at 20 cells.
    let r = rows(&o, 20, 8);
    let confirm: Vec<_> = r
        .iter()
        .filter(|l| l.starts_with("Move TEAM-1 from"))
        .collect();
    assert_eq!(confirm.len(), 1, "{r:?}");
    assert!(confirm[0].contains('…'), "{confirm:?}");
    assert!(
        !r.iter().any(|l| l.contains("y/n")),
        "tail cut, not wrapped: {r:?}"
    );
}

#[test]
fn board_overlay_chrome_rows_truncate_with_ellipsis() {
    let mut d = data(false);
    d.scope_label = "A very long scope label that cannot possibly fit".into();
    let o = BoardOverlay::new(BoardLoad::Ready(d));
    let r = rows(&o, 20, 8);
    assert!(trimmed(&r[1]).ends_with('…'), "{:?}", r[1]);
    let o = BoardOverlay::new(BoardLoad::Error(
        "a very long provider failure message here".into(),
    ));
    let r = rows(&o, 20, 8);
    assert!(trimmed(&r[2]).ends_with('…'), "{:?}", r[2]);
    assert!(
        !r[3].trim().chars().any(|c| c.is_alphanumeric()),
        "no wrap: {:?}",
        r[3]
    );
}

#[test]
fn board_overlay_notice_row_is_single_row_at_narrow_width() {
    let mut o = ready(true);
    ch(&mut o, 'm');
    let r = rows(&o, 20, 8);
    let hits = r.iter().filter(|l| l.contains("read-only")).count();
    assert!(hits >= 1);
    assert_rows_fit(&o, 20, 8);
    assert!(trimmed(&r[6]).ends_with('…'), "{:?}", r[6]);
}

#[test]
fn board_overlay_long_titles_truncate_by_cells() {
    let mut d = data(false);
    d.columns[0].issues[0].title = "x".repeat(200);
    d.columns[0].issues[1].title = "日本語のとても長いタイトルです".repeat(4);
    d.columns[1].issues[0].title = "🚀🎉 launch party 🚀🎉 launch party".into();
    let mut o = BoardOverlay::new(BoardLoad::Ready(d));
    for w in [8u16, 20, 31, 60, 120] {
        assert_rows_fit(&o, w, 10);
    }
    let r = rows(&o, 60, 10);
    assert!(r[4].contains('…'), "{:?}", r[4]);
    // CJK row: a wide char is never split across the cut (no half cell glyph).
    assert!(r[5].contains('…'), "{:?}", r[5]);
    ch(&mut o, 'l');
    assert_rows_fit(&o, 20, 10);
}

#[test]
fn board_overlay_wide_char_boundary_never_exceeds_cells() {
    use unicode_width::UnicodeWidthStr;
    for w in 1..12usize {
        let cut = truncate_cells("日本語日本語", w);
        assert!(cut.width() <= w, "{cut:?} {w}");
    }
    assert_eq!(truncate_cells("日本語", 4), "日…");
    assert_eq!(truncate_cells("abc", 3), "abc");
    assert_eq!(truncate_cells("abcd", 3), "ab…");
    assert_eq!(truncate_cells("abc", 0), "");
}

#[test]
fn board_overlay_control_chars_cannot_break_rows() {
    let mut d = data(false);
    d.columns[0].issues[0].title = "line1\nline2\ttab\r".into();
    d.scope_label = "scope\nnext".into();
    let o = BoardOverlay::new(BoardLoad::Ready(d));
    let r = rows(&o, 120, 10);
    assert!(r[1].contains("scope next"));
    assert!(r[4].contains("line1 line2 tab"));
}

#[test]
fn board_overlay_tab_strip_scrolls_to_keep_the_selected_tab_visible() {
    let mut o = ready(false);
    let r = rows(&o, 20, 10);
    assert!(r[2].contains("Todo"));
    assert!(!r[2].contains("Done"));
    ch(&mut o, 'l');
    ch(&mut o, 'l');
    let r = rows(&o, 20, 10);
    assert!(r[2].contains("Done"), "{:?}", r[2]);
    assert!(r[2].starts_with('‹'), "hidden tabs to the left: {:?}", r[2]);
    for w in [1u16, 2, 5, 8, 12, 20, 40] {
        assert_rows_fit(&o, w, 10);
    }
}

#[test]
fn board_overlay_selected_issue_scrolls_into_view() {
    let many = BoardData {
        scope_label: "s".into(),
        read_only: false,
        columns: vec![column(
            "a",
            "A",
            (0..30).map(|i| issue(&format!("N-{i}"), "t")).collect(),
        )],
    };
    let mut o = BoardOverlay::new(BoardLoad::Ready(many));
    for _ in 0..25 {
        ch(&mut o, 'j');
    }
    let t = text(&o, 40, 10);
    assert!(t.contains("▸ N-25"), "{t}");
    assert!(!t.contains("N-0 "));
}

#[test]
fn board_overlay_zero_area_render_is_noop() {
    let o = ready(false);
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));
    o.render_to(&mut buf, Rect::new(0, 0, 0, 0));
    o.render_to(&mut buf, Rect::new(0, 0, 10, 0));
}

fn with_other() -> BoardOverlay {
    let mut d = data(false);
    d.columns
        .push(column("", "Other", vec![issue("TEAM-9", "Odd state")]));
    BoardOverlay::new(BoardLoad::Ready(d))
}

#[test]
fn board_overlay_other_column_is_never_a_move_source() {
    let mut o = with_other();
    for _ in 0..3 {
        key(&mut o, KeyCode::Right);
    }
    ch(&mut o, 'm');
    assert!(text(&o, 70, 12).contains("display-only"));
    // No confirm prompt appears and Enter produces no action.
    assert_eq!(key(&mut o, KeyCode::Enter), BoardAction::None);
    assert!(!text(&o, 70, 12).contains("y/n"));
}

#[test]
fn board_overlay_other_column_is_never_a_move_target() {
    let mut o = with_other();
    // Select the issue in the last real column's neighbour: start from Done is
    // empty, so move TEAM-3 (In Progress) right twice, past Done into Other.
    key(&mut o, KeyCode::Right);
    ch(&mut o, 'm');
    for _ in 0..4 {
        key(&mut o, KeyCode::Down);
    }
    key(&mut o, KeyCode::Enter);
    match ch(&mut o, 'y') {
        BoardAction::Confirmed(m) => {
            assert!(
                !m.to_column_id.is_empty(),
                "target must be a real column: {m:?}"
            );
            assert_eq!(m.to_column_id, "c-done");
        }
        other => panic!("expected a confirmed move, got {other:?}"),
    }
}

// ---- optimistic move -------------------------------------------------------

fn confirmed_move(o: &mut BoardOverlay) -> MoveRequest {
    ch(o, 'm');
    key(o, KeyCode::Enter);
    match ch(o, 'y') {
        BoardAction::Confirmed(m) => m,
        other => panic!("expected a confirmed move, got {other:?}"),
    }
}

fn names_in(o: &BoardOverlay, col: usize) -> Vec<String> {
    o.board().unwrap().columns[col]
        .issues
        .iter()
        .map(|i| i.identifier.clone())
        .collect()
}

#[test]
fn board_overlay_confirm_moves_the_card_at_once_and_shows_progress() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    assert_eq!(names_in(&o, 0), ["TEAM-2"]);
    assert_eq!(names_in(&o, 1), ["TEAM-1", "TEAM-3"]);
    let t = text(&o, 80, 12);
    assert!(t.contains("Moving TEAM-1 to In Progress…"), "{t}");
    assert!(t.contains("In Progress (2)"), "{t}");
}

#[test]
fn board_overlay_failed_move_puts_the_card_back_with_the_reason() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    o.finish_move(
        MoveVerdict::NotMoved,
        "not moved: cannot read live state",
        None,
    );
    assert_eq!(names_in(&o, 0), ["TEAM-1", "TEAM-2"]);
    assert_eq!(names_in(&o, 1), ["TEAM-3"]);
    let t = text(&o, 90, 12);
    assert!(
        t.contains("TEAM-1: not moved: cannot read live state"),
        "{t}"
    );
    assert!(!t.contains("Moving TEAM-1"), "{t}");
}

#[test]
fn board_overlay_unknown_result_keeps_the_card_and_says_unconfirmed() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    o.finish_move(MoveVerdict::Unknown, "move result unknown: timeout", None);
    assert_eq!(names_in(&o, 1), ["TEAM-1", "TEAM-3"], "never rolled back");
    let t = text(&o, 100, 12);
    assert!(t.contains("unconfirmed"), "{t}");
}

#[test]
fn board_overlay_success_adopts_the_fresh_board() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    let mut fresh = data(false);
    let moved = fresh.columns[0].issues.remove(0);
    fresh.columns[1].issues.push(moved);
    o.finish_move(MoveVerdict::Moved, "moved", Some(BoardLoad::Ready(fresh)));
    assert_eq!(names_in(&o, 1), ["TEAM-3", "TEAM-1"], "tracker order wins");
    assert!(o.in_flight.is_none());
    let t = text(&o, 80, 12);
    assert!(t.contains("TEAM-1: moved"), "{t}");
}

#[test]
fn board_overlay_success_without_a_fresh_read_keeps_the_card_and_hints_refresh() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    o.finish_move(MoveVerdict::Moved, "moved", None);
    assert_eq!(names_in(&o, 1), ["TEAM-1", "TEAM-3"]);
    assert!(text(&o, 90, 12).contains("press r"));
}

#[test]
fn board_overlay_a_second_move_waits_for_the_first() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    ch(&mut o, 'm');
    assert!(matches!(o.mode, Mode::Browse));
    assert!(text(&o, 80, 12).contains("in progress"));
    assert_eq!(ch(&mut o, 'r'), BoardAction::None, "no refresh mid-move");
    o.finish_move(MoveVerdict::Moved, "moved", None);
    assert_eq!(ch(&mut o, 'r'), BoardAction::Refresh);
}

#[test]
fn board_overlay_set_load_clears_a_stale_in_flight_move() {
    let mut o = ready(false);
    confirmed_move(&mut o);
    o.set_load(BoardLoad::Ready(data(false)));
    assert!(o.in_flight.is_none());
    o.finish_move(MoveVerdict::NotMoved, "late", None);
    assert_eq!(
        names_in(&o, 0),
        ["TEAM-1", "TEAM-2"],
        "late rollback is a no-op"
    );
}

#[test]
fn board_overlay_status_rows_stay_single_row_at_narrow_widths_during_a_move() {
    let mut o = ready(false);
    ch(&mut o, 'm');
    for w in [1u16, 5, 12, 20, 40] {
        for h in [1u16, 4, 6, 12] {
            assert_rows_fit(&o, w, h);
        }
    }
    key(&mut o, KeyCode::Enter);
    ch(&mut o, 'y');
    for w in [1u16, 5, 12, 20, 40] {
        assert_rows_fit(&o, w, 8);
    }
}

// ---- palette styling ---------------------------------------------------------

use jcode_tui_style::{Palette, Role, role_color, set_palette};

fn render_buf(o: &BoardOverlay, w: u16, h: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal.draw(|f| o.render(f)).unwrap();
    terminal.backend().buffer().clone()
}

/// Column of the first cell of `needle` on buffer row `y`.
fn col_of(buf: &Buffer, y: u16, needle: &str) -> u16 {
    let row: Vec<&str> = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
    let first = needle.chars().next().unwrap().to_string();
    (0..row.len())
        .find(|&x| row[x] == first && row[x..].concat().starts_with(needle))
        .unwrap_or_else(|| panic!("{needle:?} not on row {y}: {row:?}")) as u16
}

/// Serialize palette access and restore the default palette afterwards.
struct PaletteGuard(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);

impl PaletteGuard {
    fn new() -> Self {
        let guard = crate::storage::lock_test_env();
        set_palette(Palette::default());
        Self(guard)
    }
}

impl Drop for PaletteGuard {
    fn drop(&mut self) {
        set_palette(Palette::default());
    }
}

#[test]
fn board_overlay_selected_tab_and_row_use_the_accent_and_selection_roles() {
    let _g = PaletteGuard::new();
    let o = ready(false);
    let buf = render_buf(&o, 60, 12);
    let accent = jcode_tui_style::theme::accent_color();

    let tab = &buf[(col_of(&buf, 2, "Todo"), 2)];
    assert_eq!(tab.fg, accent);
    assert!(tab.modifier.contains(Modifier::BOLD | Modifier::UNDERLINED));
    // Unselected tabs are not accent-colored and not underlined.
    let other = &buf[(col_of(&buf, 2, "In Progress"), 2)];
    assert_ne!(other.fg, accent);
    assert!(!other.modifier.contains(Modifier::UNDERLINED));

    let marker = &buf[(col_of(&buf, 4, "▸"), 4)];
    assert_eq!(marker.fg, accent);
    let selection = role_color(Role::SelectionBg);
    for x in 2..58 {
        assert_eq!(buf[(x, 4)].bg, selection, "selected row fill at x={x}");
    }
    // The unselected row keeps the panel background.
    assert_ne!(buf[(5, 5)].bg, selection);
    // Identifier and title use their own roles.
    let id = &buf[(col_of(&buf, 4, "TEAM-1"), 4)];
    assert_eq!(id.fg, jcode_tui_style::theme::file_link_color());
    let title = &buf[(col_of(&buf, 4, "First"), 4)];
    assert_eq!(title.fg, jcode_tui_style::theme::ai_text());
}

#[test]
fn board_overlay_frame_uses_the_border_role_and_embeds_title_and_hints() {
    let _g = PaletteGuard::new();
    let o = ready(false);
    let buf = render_buf(&o, 60, 12);
    assert_eq!(buf[(0, 0)].symbol(), "╭");
    assert_eq!(buf[(0, 0)].fg, role_color(Role::Border));
    assert_eq!(buf[(59, 11)].symbol(), "╯");
    let r = rows(&o, 60, 12);
    assert!(r[0].contains("◆ Issue board"), "{:?}", r[0]);
    assert!(
        r[0].trim_end_matches(['─', '╮'])
            .trim_end()
            .ends_with("1/3"),
        "{:?}",
        r[0]
    );
    assert!(r[11].contains("↑↓ move · m move status"), "{:?}", r[11]);
    let title = &buf[(col_of(&buf, 0, "Issue"), 0)];
    assert_eq!(title.fg, jcode_tui_style::theme::accent_color());
    let key = &buf[(col_of(&buf, 11, "↑↓"), 11)];
    assert_eq!(key.fg, jcode_tui_style::theme::accent_color());
    let label = &buf[(col_of(&buf, 11, "move status"), 11)];
    assert_eq!(label.fg, jcode_tui_style::theme::dim_color());
}

#[test]
fn board_overlay_read_only_hints_keep_read_only() {
    let _g = PaletteGuard::new();
    let o = ready(true);
    assert!(rows(&o, 80, 12)[11].contains("read-only"));
}

#[test]
fn board_overlay_configured_accent_is_used() {
    let _g = PaletteGuard::new();
    let o = ready(false);
    let default_accent = jcode_tui_style::theme::accent_color();
    let mut palette = Palette::default();
    palette.set(Role::Accent, (250, 40, 200));
    set_palette(palette);
    // Accessors return role defaults; the per-frame display pass maps them
    // onto the configured palette, as the real draw path does.
    let mut buf = render_buf(&o, 60, 12);
    jcode_tui_style::adapt_buffer_for_display(&mut buf);
    let want = Palette::from_pairs([("accent", "#fa28c8")])
        .0
        .color(Role::Accent);
    assert_ne!(want, default_accent);
    assert_eq!(buf[(col_of(&buf, 2, "Todo"), 2)].fg, want);
    assert_eq!(buf[(col_of(&buf, 4, "▸"), 4)].fg, want);
    assert_eq!(buf[(col_of(&buf, 0, "Issue"), 0)].fg, want);
}

#[test]
fn board_overlay_move_target_row_uses_accent_and_selection_fill() {
    let _g = PaletteGuard::new();
    let mut o = ready(false);
    ch(&mut o, 'm');
    let buf = render_buf(&o, 60, 16);
    let accent = jcode_tui_style::theme::accent_color();
    let y = (0..16)
        .find(|&y| {
            let row: String = (0..60).map(|x| buf[(x, y)].symbol()).collect();
            row.contains("▸ ● In Progress")
        })
        .expect("target row");
    assert_eq!(buf[(col_of(&buf, y, "▸"), y)].fg, accent);
    let name = &buf[(col_of(&buf, y, "In Progress"), y)];
    assert_eq!(name.fg, accent);
    assert!(name.modifier.contains(Modifier::BOLD));
    assert_eq!(buf[(30, y)].bg, role_color(Role::SelectionBg));
    // Group header is dim and the current status says so.
    let group = (0..16)
        .find(|&y| {
            (0..60)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .contains("UNSTARTED")
        })
        .expect("group header");
    assert_eq!(
        buf[(col_of(&buf, group, "UNSTARTED"), group)].fg,
        jcode_tui_style::theme::dim_color()
    );
    let r = rows(&o, 60, 16);
    assert!(r.iter().any(|l| l.contains("Todo (2)  current")), "{r:#?}");
}

#[test]
fn board_overlay_status_line_colors_follow_roles() {
    let _g = PaletteGuard::new();
    let mut o = ready(true);
    ch(&mut o, 'm'); // read-only notice
    let buf = render_buf(&o, 70, 12);
    assert_eq!(buf[(2, 10)].fg, role_color(Role::Warning));

    let mut o = ready(false);
    confirmed_move(&mut o);
    let buf = render_buf(&o, 70, 12);
    assert_eq!(buf[(2, 10)].fg, jcode_tui_style::theme::accent_color());
    o.finish_move(
        MoveVerdict::Moved,
        "moved",
        Some(BoardLoad::Ready(data(false))),
    );
    let buf = render_buf(&o, 70, 12);
    assert_eq!(buf[(2, 10)].fg, role_color(Role::Success));

    let mut o = ready(false);
    ch(&mut o, 'm');
    key(&mut o, KeyCode::Enter);
    let buf = render_buf(&o, 70, 12);
    assert_eq!(buf[(2, 10)].fg, role_color(Role::Warning));
    assert!(buf[(2, 10)].modifier.contains(Modifier::BOLD));

    let o = BoardOverlay::new(BoardLoad::Error("boom".into()));
    let buf = render_buf(&o, 70, 12);
    assert_eq!(buf[(2, 2)].fg, role_color(Role::Error));
}

#[test]
fn board_overlay_status_kind_dot_colors() {
    let _g = PaletteGuard::new();
    use jcode_tui_style::theme::{dim_color, pending_color, queued_color};
    let expected = [
        ("triage", pending_color()),
        ("backlog", dim_color()),
        ("unstarted", role_color(Role::Info)),
        ("started", queued_color()),
        ("completed", role_color(Role::Success)),
        ("canceled", dim_color()),
        ("cancelled", dim_color()),
        ("duplicate", dim_color()),
        ("", dim_color()),
        ("whatever", dim_color()),
    ];
    for (kind, color) in expected {
        assert_eq!(kind_dot_color(kind), color, "{kind}");
    }
    // Rendered tab dots: after the border and one space of padding.
    let mut d = data(false);
    d.columns[2].issues.push(issue("TEAM-4", "x"));
    let o = BoardOverlay::new(BoardLoad::Ready(d));
    let buf = render_buf(&o, 70, 12);
    for (name, kind) in [
        ("Todo", "unstarted"),
        ("In Progress", "started"),
        ("Done", "completed"),
    ] {
        let x = col_of(&buf, 2, name) - 2;
        assert_eq!(buf[(x, 2)].symbol(), "●", "{name}");
        assert_eq!(buf[(x, 2)].fg, kind_dot_color(kind), "{name}");
    }
}

#[test]
fn board_overlay_tab_width_counts_the_dot() {
    // " ● Todo (2) " is 12 cells; at width 12 the strip fits only Todo and the
    // scroll arrow appears, at 11 the label itself is cut with an ellipsis.
    let o = ready(false);
    let r = rows(&o, 12, 10);
    assert!(r[2].contains("● Todo (2)"), "{:?}", r[2]);
    assert!(r[2].ends_with('›'), "{:?}", r[2]);
    assert_rows_fit(&o, 11, 10);
}

#[test]
fn board_overlay_small_areas_fall_back_to_the_frameless_layout() {
    let _g = PaletteGuard::new();
    let o = ready(false);
    // Framed when large enough.
    assert!(rows(&o, 24, MIN_FRAME_HEIGHT)[0].starts_with('╭'));
    // Too narrow or too short: no border glyphs, title on row 0.
    for (w, h) in [
        (23u16, 12u16),
        (20, 12),
        (40, MIN_FRAME_HEIGHT - 1),
        (40, 6),
        (40, 3),
    ] {
        let r = rows(&o, w, h);
        assert!(r[0].starts_with("Issue board"), "{w}x{h}: {:?}", r[0]);
        assert!(
            !r.iter().any(|l| l.contains(['╭', '╮', '╰', '╯', '│'])),
            "{w}x{h}: {r:?}"
        );
    }
    // Short-height fallback still shows hints, or the status when present.
    let r = rows(&o, 40, 3);
    assert!(r[2].contains("Esc") || r[2].contains('…'), "{:?}", r[2]);
    let mut o = ready(true);
    ch(&mut o, 'm');
    let r = rows(&o, 40, 3);
    assert!(r[2].contains("read-only"), "{:?}", r[2]);
}

#[test]
fn board_overlay_no_row_wraps_or_overflows_at_any_size() {
    let _g = PaletteGuard::new();
    let mut long = data(false);
    long.scope_label = "scope ".repeat(30);
    long.columns[0].issues[0].title = "日本語のとても長いタイトルです".repeat(6);
    long.columns[0].issues[1].identifier = "VERYLONGIDENTIFIER-12345".into();
    let mut overlays = vec![
        BoardOverlay::new(BoardLoad::Ready(long.clone())),
        ready(true),
        BoardOverlay::new(BoardLoad::Error("e ".repeat(60))),
        BoardOverlay::new(BoardLoad::Loading),
    ];
    let mut moving = BoardOverlay::new(BoardLoad::Ready(long));
    ch(&mut moving, 'm');
    let mut confirm = ready(false);
    ch(&mut confirm, 'm');
    key(&mut confirm, KeyCode::Enter);
    overlays.push(moving);
    overlays.push(confirm);
    for o in &overlays {
        for w in [20u16, 40, 80] {
            for h in [1u16, 2, 3, 5, 6, 8, 9, 10, 12, 20, 40] {
                let r = rows(o, w, h);
                assert_eq!(r.len(), h as usize);
                for (y, row) in r.iter().enumerate() {
                    assert!(!row.contains('\n'), "{w}x{h} row {y}");
                    assert!(
                        unicode_width::UnicodeWidthStr::width(row.as_str()) <= w as usize,
                        "{w}x{h} row {y}: {row:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn board_overlay_tab_overflow_markers_are_dim() {
    let _g = PaletteGuard::new();
    let mut o = ready(false);
    ch(&mut o, 'l');
    ch(&mut o, 'l');
    let buf = render_buf(&o, 20, 10);
    assert_eq!(buf[(0, 2)].symbol(), "‹");
    assert_eq!(buf[(0, 2)].fg, jcode_tui_style::theme::dim_color());
}
