// Copy-safety tests for user-message framing (OpenSpec
// `add-user-message-framing`): frame decoration (border rows, rail/gutter
// columns, the `User` label) must never contribute copied text, and
// prompt-text selection stays byte-identical to the pre-change behavior.
//
// These tests follow the proven chat-pane copy pattern from
// scroll_copy_02/part_01.rs: render through `render_and_snap`, resolve
// screen coordinates through the `copy_viewport_*` APIs, then drive the
// mouse Down/Drag/Up sequence.

/// Build a transcript app in the shape create_copy_test_app uses, with the
/// given messages and user-message style.
fn framed_transcript_app(
    messages: Vec<DisplayMessage>,
    style: crate::config::UserMessageStyle,
) -> App {
    let mut app = create_test_app();
    app.display_messages = messages;
    app.bump_display_messages_version();
    app.scroll_offset = 0;
    app.auto_scroll_paused = false;
    app.is_processing = false;
    app.streaming.streaming_text.clear();
    app.status = ProcessingStatus::Idle;
    app.session.short_name = Some("test".to_string());
    app.user_messages_config = crate::config::UserMessagesConfig { style };
    app
}

/// Drive a Down/Drag/Up mouse selection and return the copied text.
fn mouse_drag_copy(app: &mut App, start: (u16, u16), end: (u16, u16)) -> String {
    let copied = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let copied_for_closure = copied.clone();
    app.handle_copy_selection_mouse_with(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: start.0,
            row: start.1,
            modifiers: KeyModifiers::empty(),
        },
        |_| true,
    );
    app.handle_copy_selection_mouse_with(
        MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: end.0,
            row: end.1,
            modifiers: KeyModifiers::empty(),
        },
        |_| true,
    );
    app.handle_copy_selection_mouse_with(
        MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: end.0,
            row: end.1,
            modifiers: KeyModifiers::empty(),
        },
        |text| {
            *copied_for_closure.lock().unwrap() = text.to_string();
            true
        },
    );
    copied.lock().unwrap().clone()
}

/// Resolve `(abs_line, visible_start, messages_area)` after rendering.
fn chat_copy_layout(app: &App) -> (std::ops::Range<usize>, ratatui::layout::Rect) {
    let _ = app;
    let layout = crate::tui::ui::last_layout_snapshot().expect("layout snapshot");
    let (visible_start, visible_end) =
        crate::tui::ui::copy_viewport_visible_range().expect("visible copy range");
    (visible_start..visible_end, layout.messages_area)
}

/// Find the abs_line whose copy text contains `needle`.
fn find_abs_line(range: &std::ops::Range<usize>, needle: &str) -> usize {
    range
        .clone()
        .find(|abs_line| {
            crate::tui::ui::copy_viewport_line_text(*abs_line)
                .map(|text| text.contains(needle))
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("expected visible line containing {needle:?}"))
}

/// Screen row for an abs_line inside the messages area.
fn screen_row(area: ratatui::layout::Rect, visible_start: usize, abs_line: usize) -> u16 {
    area.y + (abs_line - visible_start) as u16
}

/// First screen column inside the messages area that hit-tests to `abs_line`.
fn screen_col_for_abs(area: ratatui::layout::Rect, row: u16, abs_line: usize) -> u16 {
    (area.x..area.x + area.width)
        .find(|&column| {
            crate::tui::ui::copy_viewport_point_from_screen(column, row)
                .map(|point| point.abs_line == abs_line)
                .unwrap_or(false)
        })
        .expect("screen x for selection point")
}

/// Drag across every row the framed prompt occupies: top border through
/// bottom border. Returns the copied text.
fn drag_full_framed_prompt(
    app: &mut App,
    prompt_text: &str,
    style: crate::config::UserMessageStyle,
) -> String {
    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
    render_and_snap(app, &mut terminal);

    let (range, area) = chat_copy_layout(app);
    let prompt_abs = find_abs_line(&range, prompt_text);
    // Framed/labeled styles emit a top border directly before the prompt and
    // a bottom border directly after (single-line prompts).
    let top_abs = prompt_abs - 1;
    let bottom_abs = prompt_abs + 1;

    let start_row = screen_row(area, range.start, top_abs);
    let end_row = screen_row(area, range.start, bottom_abs);
    let start_x = screen_col_for_abs(area, start_row, top_abs);
    let end_x = screen_col_for_abs(area, end_row, bottom_abs);

    mouse_drag_copy(app, (start_x, start_row), (end_x, end_row))
}

#[test]
fn framed_prompt_drag_never_copies_borders_or_rail() {
    let _render_lock = scroll_render_test_lock();
    let mut app = framed_transcript_app(
        vec![
            DisplayMessage::user("copy only this prompt"),
            DisplayMessage::assistant("assistant answer row"),
        ],
        crate::config::UserMessageStyle::Framed,
    );

    let copied = drag_full_framed_prompt(
        &mut app,
        "copy only this prompt",
        crate::config::UserMessageStyle::Framed,
    );
    assert!(
        copied.contains("copy only this prompt"),
        "prompt text copied: {copied:?}"
    );
    for glyph in ["─", "│", "┌", "┐", "└", "┘", "›"] {
        assert!(
            !copied.contains(glyph),
            "decoration glyph {glyph:?} never copied: {copied:?}"
        );
    }
}

#[test]
fn framed_prompt_selection_text_is_byte_identical_to_off() {
    let _render_lock = scroll_render_test_lock();
    let select_prompt = |style: crate::config::UserMessageStyle| -> String {
        let mut app = framed_transcript_app(
            vec![
                DisplayMessage::user("byte identical selection target"),
                DisplayMessage::assistant("answer"),
            ],
            style,
        );
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
        render_and_snap(&app, &mut terminal);

        let (range, area) = chat_copy_layout(&app);
        let prompt_abs = find_abs_line(&range, "byte identical selection target");
        let row = screen_row(area, range.start, prompt_abs);
        // Span the full prompt row: from the first cell that hit-tests to the
        // prompt line through the last.
        let start_x = screen_col_for_abs(area, row, prompt_abs);
        let end_x = (area.x..area.x + area.width)
            .filter(|&column| {
                crate::tui::ui::copy_viewport_point_from_screen(column, row)
                    .map(|point| point.abs_line == prompt_abs)
                    .unwrap_or(false)
            })
            .max()
            .expect("end x for prompt row");
        mouse_drag_copy(&mut app, (start_x, row), (end_x, row))
    };

    let framed = select_prompt(crate::config::UserMessageStyle::Framed);
    let off = select_prompt(crate::config::UserMessageStyle::Off);
    assert_eq!(framed, "byte identical selection target");
    assert_eq!(
        framed, off,
        "prompt-text selection byte-identical across styles"
    );
}

#[test]
fn labeled_border_label_is_never_copied() {
    let _render_lock = scroll_render_test_lock();
    let mut app = framed_transcript_app(
        vec![
            DisplayMessage::user("labeled prompt text"),
            DisplayMessage::assistant("answer"),
        ],
        crate::config::UserMessageStyle::Labeled,
    );

    let copied = drag_full_framed_prompt(
        &mut app,
        "labeled prompt text",
        crate::config::UserMessageStyle::Labeled,
    );
    assert!(copied.contains("labeled prompt text"), "{copied:?}");
    assert!(
        !copied.contains("User"),
        "the User label is chrome, never copied: {copied:?}"
    );
    for glyph in ["╭", "╮", "╰", "╯", "─", "│"] {
        assert!(!copied.contains(glyph), "glyph {glyph:?}: {copied:?}");
    }
}

#[test]
fn compact_style_keeps_selection_clean() {
    let _render_lock = scroll_render_test_lock();
    let mut app = framed_transcript_app(
        vec![
            DisplayMessage::user("compact prompt text"),
            DisplayMessage::assistant("answer"),
        ],
        crate::config::UserMessageStyle::Compact,
    );
    let backend = ratatui::backend::TestBackend::new(100, 30);
    let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
    render_and_snap(&app, &mut terminal);

    let (range, area) = chat_copy_layout(&app);
    let prompt_abs = find_abs_line(&range, "compact prompt text");
    let row = screen_row(area, range.start, prompt_abs);
    let start_x = screen_col_for_abs(area, row, prompt_abs);
    let end_x = (area.x..area.x + area.width)
        .filter(|&column| {
            crate::tui::ui::copy_viewport_point_from_screen(column, row)
                .map(|point| point.abs_line == prompt_abs)
                .unwrap_or(false)
        })
        .max()
        .expect("end x for prompt row");
    let copied = mouse_drag_copy(&mut app, (start_x, row), (end_x, row));
    assert!(copied.contains("compact prompt text"), "{copied:?}");
    assert!(
        !copied.contains('│'),
        "compact rail never copied: {copied:?}"
    );
}
