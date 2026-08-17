#[test]
fn control_room_alt_o_opens_and_preserves_draft() {
    let mut app = create_test_app();
    app.input = "draft message".to_string();
    app.cursor_pos = app.input.len();

    app.handle_key(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::ALT).unwrap();

    assert!(app.control_room_overlay.is_some());
    assert_eq!(app.input, "draft message");
    let text = app
        .control_room_overlay
        .as_ref()
        .unwrap()
        .borrow()
        .render_text();
    assert!(text.contains("Semantic context"));
    assert!(text.contains("Execution substrate"));
    assert!(text.contains("Herdr pane"));
}

#[test]
fn control_room_alt_o_and_escape_close() {
    let mut app = create_test_app();

    app.handle_key(crossterm::event::KeyCode::Char('O'), crossterm::event::KeyModifiers::ALT).unwrap();
    assert!(app.control_room_overlay.is_some());

    app.handle_key(crossterm::event::KeyCode::Esc, crossterm::event::KeyModifiers::NONE).unwrap();
    assert!(app.control_room_overlay.is_none());

    app.handle_key(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::ALT).unwrap();
    assert!(app.control_room_overlay.is_some());
    app.handle_key(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::ALT).unwrap();
    assert!(app.control_room_overlay.is_none());
}

#[test]
fn control_room_owns_keys_without_leaking_to_prompt() {
    let mut app = create_test_app();
    app.input = "keep".to_string();
    app.cursor_pos = app.input.len();

    app.open_control_room();
    app.handle_key(crossterm::event::KeyCode::Char('x'), crossterm::event::KeyModifiers::NONE).unwrap();
    app.handle_key(crossterm::event::KeyCode::Down, crossterm::event::KeyModifiers::NONE).unwrap();

    assert_eq!(app.input, "keep");
    assert_eq!(
        app.control_room_overlay
            .as_ref()
            .unwrap()
            .borrow()
            .selected_row_index(),
        1
    );
}

#[test]
fn control_room_copy_and_non_copy_feedback() {
    let mut app = create_test_app();
    app.session.working_dir = Some("/home/nyaptor/dev/jcode/source/jcode".to_string());
    app.open_control_room();

    // Organization is unavailable and therefore not copyable.
    assert!(!app.copy_selected_control_room_value_with(|_| true));

    app.control_room_overlay.as_ref().unwrap().borrow_mut().move_next();
    let copied = std::cell::RefCell::new(String::new());
    assert!(app.copy_selected_control_room_value_with(|value| {
        copied.replace(value.to_string());
        true
    }));
    assert_eq!(copied.into_inner(), "/home/nyaptor/dev/jcode/source/jcode");
}

#[test]
fn control_room_focus_action_does_not_spawn() {
    let mut app = create_test_app();
    app.open_control_room();
    app.control_room_overlay.as_ref().unwrap().borrow_mut().last();

    app.handle_key(crossterm::event::KeyCode::Char('f'), crossterm::event::KeyModifiers::NONE).unwrap();

    assert!(app.control_room_overlay.is_some());
    assert!(app.queued_messages.is_empty());
}

#[test]
fn session_picker_keeps_ownership_over_alt_o() {
    let runtime = tokio::runtime::Runtime::new().expect("test runtime");
    let _guard = runtime.enter();
    let mut app = create_test_app();
    app.input = "/resume".to_string();
    app.submit_input();
    assert!(app.session_picker_overlay.is_some());

    app.handle_key(crossterm::event::KeyCode::Char('o'), crossterm::event::KeyModifiers::ALT).unwrap();

    assert!(app.session_picker_overlay.is_some());
    assert!(app.control_room_overlay.is_none());
}
