//! Frame-level tests for the composer frame (OpenSpec `add-composer-frame`):
//! accent rail + metadata row through real `ui::draw` frames with the
//! `TestState` fixture.
//!
//! The fixture defaults to the flat style, so every test here opts into the
//! rail style explicitly, and the footer is disabled to isolate composer
//! assertions from footer segments (both surfaces can show model/provider
//! facts).
//!
//! Copy-safety is additionally covered end-to-end by the issue #430 suite in
//! `app/tests/input_copy_selection.rs`, which runs the real App (rail style
//! on by default) and asserts typed-text selection stays byte-identical.

use super::*;

fn rail_cfg() -> crate::config::ComposerConfig {
    crate::config::ComposerConfig::default()
}

fn footer_off() -> crate::config::FooterConfig {
    let mut cfg = crate::config::FooterConfig::default();
    cfg.style = crate::config::FooterStyle::Off;
    cfg
}

fn composer_data() -> info_widget::InfoWidgetData {
    info_widget::InfoWidgetData {
        model: Some("claude-fable-5".to_string()),
        provider_name: Some("anthropic".to_string()),
        reasoning_effort: Some("high".to_string()),
        ..Default::default()
    }
}

fn base_state() -> TestState {
    TestState {
        info_widget_data: composer_data(),
        suppress_info_widgets: true,
        composer_config: Some(rail_cfg()),
        footer_config: Some(footer_off()),
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

fn rows_containing(buffer: &ratatui::buffer::Buffer, marker: &str) -> Vec<u16> {
    (0..buffer.area.height)
        .filter(|row| row_text(buffer, *row).contains(marker))
        .collect()
}

/// The composer input row carries the numbered prompt. The glyph follows the
/// composer mode: "1>" chat, "1$" shell, "1…" processing, "1»" skill.
fn prompt_row(buffer: &ratatui::buffer::Buffer, marker: &str) -> u16 {
    let rows = rows_containing(buffer, marker);
    assert_eq!(
        rows.len(),
        1,
        "exactly one input prompt row for {marker:?}: {rows:?}"
    );
    rows[0]
}

fn input_row(buffer: &ratatui::buffer::Buffer) -> u16 {
    prompt_row(buffer, "1>")
}

#[test]
fn rail_renders_on_every_composer_row() {
    let buffer = render(&base_state(), 80, 24);
    let input_y = input_row(&buffer);
    let row = row_text(&buffer, input_y);
    assert!(
        row.starts_with("│1>"),
        "rail then prompt on the input row: {row:?}"
    );
    // Metadata row directly below also carries the rail.
    let metadata = row_text(&buffer, input_y + 1);
    assert!(
        metadata.starts_with('│'),
        "rail on the metadata row: {metadata:?}"
    );
    assert!(
        metadata.contains("fable · anthropic · high"),
        "metadata content: {metadata:?}"
    );
}

#[test]
fn rail_color_follows_composer_mode() {
    // Chat mode.
    let buffer = render(&base_state(), 80, 24);
    let y = input_row(&buffer);
    let cell = &buffer[(0, y)];
    assert_eq!(cell.symbol(), "│");
    assert_eq!(
        cell.fg,
        jcode_tui_style::theme::user_color(),
        "chat rail color"
    );

    // Shell mode ("!" prefix input).
    let shell = TestState {
        input: "!ls -la".to_string(),
        ..base_state()
    };
    let buffer = render(&shell, 80, 24);
    let y = prompt_row(&buffer, "1$");
    assert_eq!(
        buffer[(0, y)].fg,
        crate::tui::ui::input_ui::shell_mode_color(),
        "shell rail color"
    );
    assert!(row_text(&buffer, y).starts_with("│1$ "), "shell prompt keeps glyph");

    // Queued/processing mode.
    let processing = TestState {
        status: ProcessingStatus::Streaming,
        streaming_text: "working".to_string(),
        ..base_state()
    };
    let buffer = render(&processing, 80, 24);
    let y = prompt_row(&buffer, "1…");
    assert_eq!(
        buffer[(0, y)].fg,
        jcode_tui_style::theme::queued_color(),
        "queued rail color"
    );

    // Skill mode.
    let skill = TestState {
        active_skill: Some("frontend-design".to_string()),
        ..base_state()
    };
    let buffer = render(&skill, 80, 24);
    let y = prompt_row(&buffer, "1»");
    assert_eq!(
        buffer[(0, y)].fg,
        jcode_tui_style::theme::accent_color(),
        "skill rail color"
    );
}

#[test]
fn flat_style_reserves_nothing() {
    let flat = TestState {
        composer_config: Some(crate::config::ComposerConfig {
            style: crate::config::ComposerStyle::Flat,
            metadata: true, // flat overrides: no metadata row either
        }),
        ..base_state()
    };
    let buffer = render(&flat, 80, 24);
    let y = input_row(&buffer);
    let row = row_text(&buffer, y);
    assert!(row.starts_with("1>"), "no rail column in flat: {row:?}");
    assert!(
        !buffer_text(&buffer).contains("fable"),
        "flat renders no metadata row"
    );
    // And it matches the fixture default (which is flat) exactly.
    let default_fixture = TestState {
        info_widget_data: composer_data(),
        suppress_info_widgets: true,
        footer_config: Some(footer_off()),
        ..Default::default()
    };
    assert_eq!(
        buffer_text(&buffer),
        buffer_text(&render(&default_fixture, 80, 24)),
        "explicit flat must equal the pre-frame fixture baseline"
    );
}

#[test]
fn ascii_mode_uses_pipe_rail() {
    let mut ascii_footer = footer_off();
    ascii_footer.icon_mode = crate::config::FooterIconMode::Ascii;
    let state = TestState {
        footer_config: Some(ascii_footer),
        ..base_state()
    };
    let buffer = render(&state, 80, 24);
    let y = input_row(&buffer);
    let row = row_text(&buffer, y);
    assert!(row.starts_with("|1>"), "ascii rail: {row:?}");
    assert!(!row.contains('│'), "no unicode rail: {row:?}");
    let metadata = row_text(&buffer, y + 1);
    assert!(metadata.starts_with('|'));
    assert!(
        metadata.contains("fable | anthropic | high"),
        "ascii metadata separators: {metadata:?}"
    );
}

#[test]
fn metadata_disabled_reserves_no_row_but_keeps_rail() {
    let mut cfg = rail_cfg();
    cfg.metadata = false;
    let state = TestState {
        composer_config: Some(cfg),
        ..base_state()
    };
    let buffer = render(&state, 80, 24);
    let y = input_row(&buffer);
    assert!(row_text(&buffer, y).starts_with("│1>"), "rail stays");
    assert!(
        !buffer_text(&buffer).contains("fable"),
        "no metadata content anywhere"
    );
    // The row below the input is not a railed metadata row.
    let below = row_text(&buffer, y + 1);
    assert!(
        !below.starts_with('│'),
        "no metadata row reserved: {below:?}"
    );
}

#[test]
fn metadata_survives_processing() {
    let state = TestState {
        status: ProcessingStatus::Streaming,
        streaming_text: "working on it".to_string(),
        ..base_state()
    };
    let buffer = render(&state, 80, 24);
    let y = prompt_row(&buffer, "1…");
    assert!(
        row_text(&buffer, y + 1).contains("fable · anthropic · high"),
        "metadata stays while the fact stack would stand down"
    );
}

#[test]
fn model_unavailable_keeps_composer_height_stable() {
    let mut no_model = base_state();
    no_model.info_widget_data.model = None;
    let with_model = render(&base_state(), 80, 24);
    let without_model = render(&no_model, 80, 24);
    let y_with = input_row(&with_model);
    let y_without = input_row(&without_model);
    assert_eq!(y_with, y_without, "composer height must not shift");
    // The reserved row below stays railed. The composer itself renders no
    // metadata content when the model is unknown; the row's free space may
    // still host the right fact stack (pre-existing composition over chrome
    // rows, which falls back to the provider's model name).
    let row = row_text(&without_model, y_without + 1);
    assert!(row.starts_with('│'), "rail on the empty metadata row");
    assert!(
        !row.contains("anthropic"),
        "composer metadata itself stands down: {row:?}"
    );
}

#[test]
fn metadata_drops_segments_at_narrow_widths() {
    // 60 columns: effort drops first.
    let buffer = render(&base_state(), 60, 24);
    let y = input_row(&buffer);
    let metadata = row_text(&buffer, y + 1);
    assert!(metadata.contains("fable"), "model kept at 60: {metadata:?}");
    // Rail present on both rows at every gate width.
    for width in [60u16, 80, 100, 120, 160] {
        let buffer = render(&base_state(), width, 24);
        let y = input_row(&buffer);
        assert_eq!(
            buffer[(0, y)].symbol(),
            "│",
            "rail on input row at width {width}"
        );
        assert_eq!(
            buffer[(0, y + 1)].symbol(),
            "│",
            "rail on metadata row at width {width}"
        );
    }
}

#[test]
fn packed_and_scrolling_are_byte_identical_across_renders() {
    // Packed (short transcript).
    let first = render(&base_state(), 100, 30);
    let second = render(&base_state(), 100, 30);
    assert_eq!(buffer_text(&first), buffer_text(&second));

    // Scrolling (transcript taller than the viewport): the composer sits at
    // the physical bottom, metadata row last.
    let mut messages = Vec::new();
    for idx in 0..120 {
        messages.push(DisplayMessage {
            role: "user".into(),
            content: format!("question {idx}"),
            tool_calls: vec![],
            duration_secs: None,
            title: None,
            tool_data: None,
        });
        messages.push(DisplayMessage {
            role: "assistant".into(),
            content: format!("answer {idx}"),
            tool_calls: vec![],
            duration_secs: None,
            title: None,
            tool_data: None,
        });
    }
    let scrolling = TestState {
        display_messages: messages,
        ..base_state()
    };
    let first = render(&scrolling, 100, 30);
    let second = render(&scrolling, 100, 30);
    assert_eq!(buffer_text(&first), buffer_text(&second));
    let bottom = row_text(&first, 29);
    assert!(
        bottom.starts_with('│') && bottom.contains("fable"),
        "metadata row at physical bottom when scrolling: {bottom:?}"
    );
}

#[test]
fn hint_rows_carry_the_rail() {
    // Shell mode shows the shell hint as a composer-owned hint row.
    let state = TestState {
        input: "!cargo test".to_string(),
        ..base_state()
    };
    let buffer = render(&state, 100, 24);
    let hint_rows = rows_containing(&buffer, "shell mode");
    assert_eq!(hint_rows.len(), 1, "shell hint row present");
    assert_eq!(
        buffer[(0, hint_rows[0])].symbol(),
        "│",
        "rail on the hint row: {:?}",
        row_text(&buffer, hint_rows[0])
    );
}

#[test]
fn send_mode_indicator_keeps_its_reservation() {
    // Processing + typed input shows the queue/send hint; the send-mode
    // indicator reservation must not collide with the rail or metadata.
    let state = TestState {
        input: "queued prompt".to_string(),
        status: ProcessingStatus::Streaming,
        streaming_text: "working".to_string(),
        ..base_state()
    };
    let buffer = render(&state, 100, 24);
    let y = prompt_row(&buffer, "1…");
    assert!(row_text(&buffer, y).starts_with('│'));
    assert!(row_text(&buffer, y + 1).contains("fable"));
}
