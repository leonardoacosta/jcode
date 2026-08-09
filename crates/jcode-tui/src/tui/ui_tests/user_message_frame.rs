//! Frame-level tests for user-message framing (OpenSpec
//! `add-user-message-framing`) through real `ui::draw` frames with the
//! `TestState` fixture.
//!
//! The fixture defaults to `off`, so every test opts into a style explicitly;
//! the pre-existing golden suite doubles as the byte-identical rollback
//! proof. Copy-safety is additionally covered end-to-end by an App-level
//! drag-selection test in `app/tests/user_message_frame_copy.rs`.

use super::*;

fn style_cfg(style: crate::config::UserMessageStyle) -> crate::config::UserMessagesConfig {
    crate::config::UserMessagesConfig { style }
}

fn footer_off() -> crate::config::FooterConfig {
    let mut cfg = crate::config::FooterConfig::default();
    cfg.style = crate::config::FooterStyle::Off;
    cfg
}

fn composer_flat() -> crate::config::ComposerConfig {
    crate::config::ComposerConfig {
        style: crate::config::ComposerStyle::Flat,
        metadata: false,
    }
}

fn user_msg(content: &str) -> DisplayMessage {
    DisplayMessage {
        role: "user".into(),
        content: content.into(),
        tool_calls: vec![],
        duration_secs: None,
        title: None,
        tool_data: None,
    }
}

fn assistant_msg(content: &str) -> DisplayMessage {
    DisplayMessage {
        role: "assistant".into(),
        content: content.into(),
        tool_calls: vec![],
        duration_secs: None,
        title: None,
        tool_data: None,
    }
}

fn base_state(style: crate::config::UserMessageStyle) -> TestState {
    TestState {
        display_messages: vec![
            user_msg("Fix the flaky login test"),
            assistant_msg("Looking at the failure now."),
        ],
        suppress_info_widgets: true,
        user_messages_config: Some(style_cfg(style)),
        footer_config: Some(footer_off()),
        composer_config: Some(composer_flat()),
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
    (0..buffer.area.height)
        .map(|row| row_text(buffer, row))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rows_starting_with(buffer: &ratatui::buffer::Buffer, prefix: char) -> Vec<u16> {
    (0..buffer.area.height)
        .filter(|row| row_text(buffer, *row).trim_start().starts_with(prefix))
        .collect()
}

#[test]
fn framed_surrounds_prompt_with_borders_and_rail() {
    let buffer = render(&base_state(crate::config::UserMessageStyle::Framed), 80, 20);
    let text = buffer_text(&buffer);
    let tops = rows_starting_with(&buffer, '┌');
    let bottoms = rows_starting_with(&buffer, '└');
    assert_eq!(tops.len(), 1, "one top border: {text}");
    assert_eq!(bottoms.len(), 1, "one bottom border: {text}");
    assert_eq!(tops[0] + 2, bottoms[0], "borders enclose exactly the prompt row");
    let prompt_row = row_text(&buffer, tops[0] + 1);
    assert!(
        prompt_row.trim_start().starts_with("│1› Fix the flaky login test"),
        "rail, numbering, and text inside the frame: {prompt_row:?}"
    );
    // Border spans the chat column width.
    let top = row_text(&buffer, tops[0]);
    assert!(top.trim_end().ends_with('┐'), "top border closed: {top:?}");
    assert_eq!(
        top.trim_end().chars().count(),
        79,
        "border spans the transcript column (width minus scrollbar gutter): {top:?}"
    );
}

#[test]
fn framed_copy_friendly_has_gutter_instead_of_rail() {
    let buffer = render(
        &base_state(crate::config::UserMessageStyle::FramedCopyFriendly),
        80,
        20,
    );
    let tops = rows_starting_with(&buffer, '┌');
    let bottoms = rows_starting_with(&buffer, '└');
    assert_eq!(tops.len(), 1);
    assert_eq!(bottoms.len(), 1);
    let prompt_row = row_text(&buffer, tops[0] + 1);
    assert!(
        prompt_row.trim_start().starts_with("1›"),
        "no rail glyph, plain gutter: {prompt_row:?}"
    );
    // The trailing │ is the pre-existing right-edge user bar; the leading
    // decoration must be a plain gutter, not a rail.
    assert!(
        !prompt_row.trim_start().starts_with('│'),
        "no leading rail on the prompt row: {prompt_row:?}"
    );
}

#[test]
fn compact_adds_no_rows() {
    let compact = render(
        &base_state(crate::config::UserMessageStyle::Compact),
        80,
        20,
    );
    let off = render(
        &base_state(crate::config::UserMessageStyle::Off),
        80,
        20,
    );
    let compact_text = buffer_text(&compact);
    assert!(
        !compact_text.contains('┌') && !compact_text.contains('└'),
        "compact has no borders"
    );
    assert!(
        compact_text.contains("│1› Fix the flaky login test"),
        "compact keeps the rail: {compact_text}"
    );
    // Zero added height: the transcript rows align exactly with off apart
    // from the rail column itself.
    let off_prompt = rows_starting_with(&off, '1');
    let compact_prompt_rows: Vec<u16> = (0..compact.area.height)
        .filter(|row| row_text(&compact, *row).contains("1›"))
        .collect();
    assert_eq!(off_prompt.len(), 1);
    assert_eq!(compact_prompt_rows.len(), 1);
    assert_eq!(
        off_prompt[0], compact_prompt_rows[0],
        "compact does not shift row positions"
    );
}

#[test]
fn labeled_draws_rounded_box_with_user_label() {
    let buffer = render(
        &base_state(crate::config::UserMessageStyle::Labeled),
        80,
        20,
    );
    let text = buffer_text(&buffer);
    let tops = rows_starting_with(&buffer, '╭');
    let bottoms = rows_starting_with(&buffer, '╰');
    assert_eq!(tops.len(), 1, "rounded top: {text}");
    assert_eq!(bottoms.len(), 1, "rounded bottom: {text}");
    let top = row_text(&buffer, tops[0]);
    assert!(top.contains(" User "), "label in top border: {top:?}");
    let prompt_row = row_text(&buffer, tops[0] + 1);
    assert!(prompt_row.trim_start().starts_with("│1›"));
}

#[test]
fn off_is_byte_identical_to_fixture_default() {
    let explicit_off = render(
        &base_state(crate::config::UserMessageStyle::Off),
        80,
        20,
    );
    let default_fixture = TestState {
        display_messages: vec![
            user_msg("Fix the flaky login test"),
            assistant_msg("Looking at the failure now."),
        ],
        suppress_info_widgets: true,
        footer_config: Some(footer_off()),
        composer_config: Some(composer_flat()),
        ..Default::default()
    };
    let default_render = render(&default_fixture, 80, 20);
    assert_eq!(
        buffer_text(&explicit_off),
        buffer_text(&default_render),
        "explicit off must equal the pre-change fixture baseline"
    );
}

#[test]
fn multi_line_prompt_frames_every_wrapped_row() {
    let mut state = base_state(crate::config::UserMessageStyle::Framed);
    state.display_messages[0] = user_msg("First line of the ask\nsecond line of the ask\nthird line");
    let buffer = render(&state, 60, 20);
    let tops = rows_starting_with(&buffer, '┌');
    let bottoms = rows_starting_with(&buffer, '└');
    assert_eq!(tops.len(), 1);
    assert_eq!(bottoms.len(), 1);
    // Three content lines (possibly more after wrapping) between the borders.
    let enclosed = bottoms[0] - tops[0] - 1;
    assert!(enclosed >= 3, "all prompt lines framed, got {enclosed}");
    for row in (tops[0] + 1)..bottoms[0] {
        let text = row_text(&buffer, row);
        assert!(
            text.trim_start().starts_with('│'),
            "rail on every prompt row {row}: {text:?}"
        );
    }
}

#[test]
fn frames_at_every_gate_width_and_byte_identical() {
    for width in [60u16, 80, 100, 120, 160] {
        for style in [
            crate::config::UserMessageStyle::Framed,
            crate::config::UserMessageStyle::FramedCopyFriendly,
            crate::config::UserMessageStyle::Compact,
            crate::config::UserMessageStyle::Labeled,
            crate::config::UserMessageStyle::Off,
        ] {
            crate::tui::ui::clear_test_render_state_for_tests();
            let first = render(&base_state(style), width, 20);
            // Reset global render state (flicker history records a same-state
            // redraw for the identical second render and would inject a
            // notification row, shifting the whole frame).
            crate::tui::ui::clear_test_render_state_for_tests();
            let second = render(&base_state(style), width, 20);
            assert_eq!(
                buffer_text(&first),
                buffer_text(&second),
                "repeated renders identical at {width} for {style:?}"
            );
            let text = buffer_text(&first);
            match style {
                crate::config::UserMessageStyle::Framed => {
                    assert!(text.contains('┌'), "framed border at {width}");
                }
                crate::config::UserMessageStyle::Labeled => {
                    assert!(text.contains('╭'), "labeled border at {width}");
                }
                crate::config::UserMessageStyle::Compact => {
                    assert!(!text.contains('┌'), "compact has no border at {width}");
                    assert!(text.contains("│1›"), "compact rail at {width}");
                }
                _ => {}
            }
        }
    }
}

#[test]
fn ascii_mode_draws_plain_borders_and_label() {
    let mut footer = footer_off();
    footer.icon_mode = crate::config::FooterIconMode::Ascii;
    let state = TestState {
        footer_config: Some(footer),
        ..base_state(crate::config::UserMessageStyle::Labeled)
    };
    let buffer = render(&state, 80, 20);
    let text = buffer_text(&buffer);
    let tops = rows_starting_with(&buffer, '+');
    assert!(!tops.is_empty(), "ascii corners: {text}");
    assert!(text.contains(" User "), "plain label: {text}");
    assert!(text.contains('-'), "ascii fill: {text}");
    assert!(!text.contains('╭') && !text.contains('┌'), "no unicode corners");
    let prompt_rows: Vec<u16> = (0..buffer.area.height)
        .filter(|row| row_text(&buffer, *row).contains("1›"))
        .collect();
    assert_eq!(prompt_rows.len(), 1);
    assert!(
        row_text(&buffer, prompt_rows[0]).trim_start().starts_with('|'),
        "ascii rail on the prompt row"
    );
}

#[test]
fn scrolling_layout_keeps_frames_attached_to_prompts() {
    let mut messages = Vec::new();
    for idx in 0..30 {
        messages.push(user_msg(&format!("question number {idx}")));
        messages.push(assistant_msg(&format!(
            "answer {idx} with enough text to wrap around the transcript width for sure"
        )));
    }
    let state = TestState {
        display_messages: messages,
        ..base_state(crate::config::UserMessageStyle::Framed)
    };
    let first = render(&state, 100, 24);
    crate::tui::ui::clear_test_render_state_for_tests();
    let second = render(&state, 100, 24);
    assert_eq!(buffer_text(&first), buffer_text(&second));
    let text = buffer_text(&first);
    // Bottom-anchored: the last prompt (30) is framed near the bottom.
    assert!(text.contains("30›"), "latest prompt visible: {text}");
    // Every fully visible frame is balanced: rails only on rows between a
    // top and bottom border. A frame may straddle the viewport edge (its
    // border scrolled off, or its prompt replaced by the sticky preview
    // row); only paired borders are asserted.
    let mut tops = rows_starting_with(&first, '┌');
    let mut bottoms = rows_starting_with(&first, '└');
    if let (Some(&first_bottom), Some(&first_top)) = (bottoms.first(), tops.first()) {
        if first_bottom < first_top {
            bottoms.remove(0);
        }
    }
    if let (Some(&last_top), Some(&last_bottom)) = (tops.last(), bottoms.last()) {
        if last_top > last_bottom {
            tops.pop();
        }
    }
    assert_eq!(
        tops.len(),
        bottoms.len(),
        "balanced visible borders: {text}"
    );
    for (top, bottom) in tops.iter().zip(bottoms.iter()) {
        assert!(bottom > top, "bottom below top");
        for row in (*top + 1)..*bottom {
            assert!(
                row_text(&first, row).trim_start().starts_with('│'),
                "rail between borders at row {row}"
            );
        }
    }
}

#[test]
fn frames_do_not_rewrap_prompt_text() {
    // A prompt that exactly fills the user width must keep its wrapped row
    // count with the rail added (the rail replaces one gutter column).
    let long = "word ".repeat(30).trim().to_string();
    let mut off_state = base_state(crate::config::UserMessageStyle::Off);
    off_state.display_messages[0] = user_msg(&long);
    let mut framed_state = base_state(crate::config::UserMessageStyle::Framed);
    framed_state.display_messages[0] = user_msg(&long);
    let off = render(&off_state, 60, 24);
    let framed = render(&framed_state, 60, 24);
    let off_rows: Vec<u16> = (0..off.area.height)
        .filter(|row| row_text(&off, *row).contains("word"))
        .collect();
    let tops = rows_starting_with(&framed, '┌');
    let bottoms = rows_starting_with(&framed, '└');
    assert_eq!(tops.len(), 1);
    assert_eq!(bottoms.len(), 1);
    let framed_content_rows = bottoms[0] - tops[0] - 1;
    assert_eq!(
        off_rows.len() as u16,
        framed_content_rows,
        "decoration must not add wrapping: off {:?} framed {}",
        off_rows,
        framed_content_rows
    );
}

#[test]
fn streaming_keeps_static_user_frames_stable() {
    let state = TestState {
        status: ProcessingStatus::Streaming,
        streaming_text: "Working on the fix now.".to_string(),
        ..base_state(crate::config::UserMessageStyle::Framed)
    };
    let first = render(&state, 80, 20);
    let second = render(&state, 80, 20);
    assert_eq!(
        buffer_text(&first),
        buffer_text(&second),
        "frames are static during streaming"
    );
    let text = buffer_text(&first);
    assert!(text.contains('┌') && text.contains('└'));
}
