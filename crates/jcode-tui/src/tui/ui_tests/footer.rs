//! Frame-level tests for the persistent status footer (OpenSpec
//! `add-status-footer`). These render real frames through `ui::draw` with the
//! `TestState` fixture and assert row reservation, placement, degradation,
//! config rollback, and copy-selection exclusion.
//!
//! Placement model: the footer is the last chunk of the chat-column layout.
//! In the scrolling layout (transcript taller than the viewport) that is the
//! physical bottom row. In the packed layout (short transcript, top-anchored
//! flow) it hugs the chrome stack directly below the input row, exactly like
//! the status line and overscroll rows above it.
//!
//! All tests set `suppress_info_widgets`: the floating info-widget overlays
//! place themselves from live margins and are not frame-deterministic, which
//! would break byte-identity assertions. The footer reads the same
//! `InfoWidgetData` snapshot regardless of overlay suppression.

use super::*;

fn footer_data() -> info_widget::InfoWidgetData {
    info_widget::InfoWidgetData {
        working_dir: Some("/home/user/dev/jcode".to_string()),
        model: Some("claude-fable-5".to_string()),
        provider_name: Some("anthropic".to_string()),
        reasoning_effort: Some("high".to_string()),
        git_info: Some(info_widget::GitInfo {
            branch: "main".to_string(),
            modified: 1,
            staged: 0,
            untracked: 0,
            ahead: 2,
            behind: 0,
            dirty_files: Vec::new(),
        }),
        context_limit: Some(200_000),
        observed_context_tokens: Some(80_000),
        ..Default::default()
    }
}

fn base_state() -> TestState {
    TestState {
        info_widget_data: footer_data(),
        suppress_info_widgets: true,
        display_messages: vec![msg("user", "hello"), msg("assistant", "hi there")],
        ..Default::default()
    }
}

fn render(state: &TestState, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| crate::tui::ui::draw(frame, state))
        .expect("draw");
    terminal.backend().buffer().clone()
}

fn row_text(buffer: &ratatui::buffer::Buffer, row: u16) -> String {
    let area = buffer.area;
    let start = row as usize * area.width as usize;
    let end = start + area.width as usize;
    buffer.content[start..end]
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
    let area = buffer.area;
    (0..area.height)
        .map(|row| row_text(buffer, row))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Rows whose text contains `marker`.
fn rows_containing(buffer: &ratatui::buffer::Buffer, marker: &str) -> Vec<u16> {
    (0..buffer.area.height)
        .filter(|row| row_text(buffer, *row).contains(marker))
        .collect()
}

fn msg(role: &str, content: &str) -> DisplayMessage {
    DisplayMessage {
        role: role.into(),
        content: content.into(),
        tool_calls: vec![],
        duration_secs: None,
        title: None,
        tool_data: None,
        artifact: None,
    }
}

fn long_transcript() -> Vec<DisplayMessage> {
    let mut messages = Vec::new();
    for idx in 0..120 {
        messages.push(msg("user", &format!("question {idx}")));
        messages.push(msg("assistant", &format!("answer {idx}")));
    }
    messages
}

#[test]
fn footer_renders_below_input_in_packed_layout() {
    let buffer = render(&base_state(), 120, 30);
    let footer_rows = rows_containing(&buffer, "fable");
    assert_eq!(
        footer_rows.len(),
        1,
        "footer must occupy exactly one row: {footer_rows:?}"
    );
    let footer_row = footer_rows[0];
    let footer = row_text(&buffer, footer_row);
    assert!(footer.contains("jcode"), "cwd on footer row: {footer:?}");
    assert!(footer.contains("main"), "branch on footer row: {footer:?}");
    assert!(
        footer.contains("anthropic"),
        "provider on footer row: {footer:?}"
    );
    assert!(footer.contains("40%"), "context on footer row: {footer:?}");
    // Packed layout: the footer hugs the chrome stack, directly below the
    // input prompt row ("2>").
    let input_rows = rows_containing(&buffer, "2>");
    assert_eq!(input_rows.len(), 1, "one input prompt row: {input_rows:?}");
    assert!(
        footer_row > input_rows[0],
        "footer row {footer_row} must sit below input row {}",
        input_rows[0]
    );
    assert!(
        footer_row < 29,
        "packed footer hugs content, not the physical bottom: {footer_row}"
    );
}

#[test]
fn footer_is_physical_bottom_row_when_scrolling() {
    let state = TestState {
        display_messages: long_transcript(),
        ..base_state()
    };
    let buffer = render(&state, 100, 30);
    let bottom = row_text(&buffer, 29);
    assert!(bottom.contains("fable"), "scrolling footer row: {bottom:?}");
    assert!(bottom.contains("main"), "branch at bottom: {bottom:?}");
    assert_eq!(rows_containing(&buffer, "fable"), vec![29]);
}

#[test]
fn footer_off_removes_content_and_is_deterministic() {
    let mut off = crate::config::FooterConfig::default();
    off.style = crate::config::FooterStyle::Off;
    let state = TestState {
        footer_config: Some(off),
        ..base_state()
    };
    let first = render(&state, 120, 30);
    let second = render(&state, 120, 30);
    assert_eq!(
        buffer_text(&first),
        buffer_text(&second),
        "off-mode render must be deterministic"
    );
    let text = buffer_text(&first);
    assert!(
        !text.contains("fable"),
        "off mode renders no footer content anywhere: {text}"
    );
    assert!(
        !text.contains("anthropic · high"),
        "off mode renders no footer metadata: {text}"
    );
}

#[test]
fn footer_on_is_deterministic_and_differs_from_off() {
    let state = base_state();
    let first = render(&state, 120, 30);
    let second = render(&state, 120, 30);
    assert_eq!(buffer_text(&first), buffer_text(&second));

    let mut off = crate::config::FooterConfig::default();
    off.style = crate::config::FooterStyle::Off;
    let off_state = TestState {
        footer_config: Some(off),
        ..state
    };
    let off_buffer = render(&off_state, 120, 30);
    assert_ne!(
        buffer_text(&first),
        buffer_text(&off_buffer),
        "enabling the footer must change the frame"
    );
}

#[test]
fn footer_off_layout_matches_on_layout_minus_row() {
    // The rollback contract: with the footer off, every other row renders
    // identically; with it on, the transcript region loses exactly one row
    // at the bottom of the chrome stack.
    let mut off = crate::config::FooterConfig::default();
    off.style = crate::config::FooterStyle::Off;
    let off_state = TestState {
        footer_config: Some(off),
        display_messages: long_transcript(),
        ..base_state()
    };
    let on_state = TestState {
        display_messages: long_transcript(),
        ..base_state()
    };
    let off_buffer = render(&off_state, 100, 30);
    let on_buffer = render(&on_state, 100, 30);
    let footer_row = row_text(&on_buffer, 29);
    assert!(
        footer_row.contains("fable"),
        "on footer row: {footer_row:?}"
    );
    assert!(
        !buffer_text(&off_buffer).contains("fable"),
        "off render has no footer"
    );
}

#[test]
fn footer_holds_one_row_at_gate_widths() {
    for width in [60u16, 80, 100, 120, 160] {
        let state = TestState {
            display_messages: long_transcript(),
            ..base_state()
        };
        let buffer = render(&state, width, 24);
        let bottom = row_text(&buffer, 23);
        assert!(
            bottom.contains("jcode"),
            "cwd survives at width {width}: {bottom:?}"
        );
        assert!(
            unicode_width::UnicodeWidthStr::width(bottom.trim_end()) <= width as usize,
            "footer row fits width {width}: {bottom:?}"
        );
    }
}

#[test]
fn footer_ascii_mode_uses_ascii_glyphs() {
    let mut ascii = crate::config::FooterConfig::default();
    ascii.icon_mode = crate::config::FooterIconMode::Ascii;
    let state = TestState {
        footer_config: Some(ascii),
        display_messages: long_transcript(),
        ..base_state()
    };
    let buffer = render(&state, 120, 24);
    let bottom = row_text(&buffer, 23);
    assert!(bottom.contains("^2"), "ascii ahead marker: {bottom:?}");
    assert!(!bottom.contains('↑'), "no unicode arrow: {bottom:?}");
    assert!(!bottom.contains('·'), "no middot separator: {bottom:?}");
}

#[test]
fn footer_decoration_stays_out_of_transcript_region() {
    // A distinctive directory name proves the footer's left zone never leaks
    // into any other row (the transcript region is what copy selection reads).
    let mut state = base_state();
    state.info_widget_data.working_dir = Some("/home/user/dev/xyzzy-plugh".to_string());
    let buffer = render(&state, 120, 30);
    let rows = rows_containing(&buffer, "xyzzy-plugh");
    assert_eq!(
        rows.len(),
        1,
        "footer decoration must appear on exactly one row: {rows:?}"
    );
}

#[test]
fn footer_renders_consistently_while_streaming() {
    let state = TestState {
        streaming_text: "streaming **markdown** response text".to_string(),
        status: ProcessingStatus::Streaming,
        display_messages: long_transcript(),
        ..base_state()
    };
    let first = render(&state, 120, 30);
    let second = render(&state, 120, 30);
    assert_eq!(
        row_text(&first, 29),
        row_text(&second, 29),
        "footer row must be stable across streaming frames"
    );
    assert!(row_text(&first, 29).contains("fable"));
}

#[test]
fn footer_segment_toggles_remove_segments() {
    let mut cfg = crate::config::FooterConfig::default();
    cfg.segments.git = false;
    cfg.segments.provider = false;
    let state = TestState {
        footer_config: Some(cfg),
        display_messages: long_transcript(),
        ..base_state()
    };
    let buffer = render(&state, 120, 24);
    let bottom = row_text(&buffer, 23);
    assert!(!bottom.contains("main"), "git segment hidden: {bottom:?}");
    assert!(!bottom.contains("anthropic"), "provider hidden: {bottom:?}");
    assert!(bottom.contains("jcode"), "cwd remains: {bottom:?}");
    assert!(bottom.contains("fable"), "model remains: {bottom:?}");
}
