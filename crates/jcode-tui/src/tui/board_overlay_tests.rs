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
    BoardColumn {
        id: id.to_string(),
        name: name.to_string(),
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
fn board_overlay_columns_render_left_to_right_in_order_with_issues() {
    let o = ready(false);
    let r = rows(&o, 60, 10);
    let head = &r[2];
    let todo = head.find("Todo").unwrap();
    let prog = head.find("In Progress").unwrap();
    let done = head.find("Done").unwrap();
    assert!(todo < prog && prog < done, "{head}");
    // Issues sit under their own column.
    let first_issue_row = &r[3];
    assert!(first_issue_row.find("TEAM-1").unwrap() < first_issue_row.find("TEAM-3").unwrap());
    assert!(r[4].contains("TEAM-2"));
    assert!(head.contains("Todo (2)"));
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
    key(&mut o, KeyCode::Right); // skip past In Progress to Done
    key(&mut o, KeyCode::Left); // and back
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
fn board_overlay_move_target_never_the_source() {
    let mut o = ready(false);
    key(&mut o, KeyCode::Right); // select In Progress
    ch(&mut o, 'm');
    let Mode::Moving(p) = o.mode else { panic!() };
    assert_ne!(p.to, p.from);
    // Walk left then right across the source: never lands on it.
    for code in [
        KeyCode::Left,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Right,
        KeyCode::Right,
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
    let confirm: Vec<_> = r.iter().filter(|l| l.contains("Move")).collect();
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
    assert!(r[3].contains('…'), "{:?}", r[3]);
    // CJK row: a wide char is never split across the cut (no half cell glyph).
    assert!(r[4].contains('…'), "{:?}", r[4]);
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
    assert!(r[3].contains("line1 line2 tab"));
}

#[test]
fn board_overlay_pages_columns_on_narrow_widths_and_follows_selection() {
    let mut o = ready(false);
    let r = rows(&o, 20, 10);
    assert!(r[2].contains("Todo"));
    assert!(!r[2].contains("Done"));
    ch(&mut o, 'l');
    ch(&mut o, 'l');
    let r = rows(&o, 20, 10);
    assert!(r[2].contains("Done"), "{:?}", r[2]);
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
    assert!(t.contains("> N-25"), "{t}");
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
        key(&mut o, KeyCode::Right);
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
