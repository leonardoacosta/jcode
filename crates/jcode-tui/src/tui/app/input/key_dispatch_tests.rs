//! Characterization tests for key dispatch in `input.rs`.
use super::shell_commands::combine_shell_output;
use crate::tui::app::tests::create_test_app;
use crossterm::event::{KeyCode, KeyModifiers};

fn type_text(app: &mut crate::tui::app::App, text: &str) {
    for c in text.chars() {
        app.handle_key(KeyCode::Char(c), KeyModifiers::empty())
            .unwrap();
    }
}

#[test]
fn key_dispatch_enter_submits_typed_text() {
    let mut app = create_test_app();
    type_text(&mut app, "hi");
    app.handle_key(KeyCode::Enter, KeyModifiers::empty())
        .unwrap();
    assert!(app.is_processing);
    assert!(app.input.is_empty());
    assert_eq!(app.display_messages.last().unwrap().content, "hi");
}

#[test]
fn key_dispatch_enter_on_empty_input_does_nothing() {
    let mut app = create_test_app();
    let before = app.display_messages.len();
    app.handle_key(KeyCode::Enter, KeyModifiers::empty())
        .unwrap();
    assert!(!app.is_processing);
    assert_eq!(app.display_messages.len(), before);
}

#[test]
fn key_dispatch_shift_enter_inserts_newline() {
    let mut app = create_test_app();
    type_text(&mut app, "a");
    app.handle_key(KeyCode::Enter, KeyModifiers::SHIFT).unwrap();
    type_text(&mut app, "b");
    assert_eq!(app.input, "a\nb");
    assert!(!app.is_processing);
}

#[test]
fn key_dispatch_up_recalls_previous_prompt_on_empty_input() {
    let mut app = create_test_app();
    app.display_messages = vec![crate::tui::DisplayMessage::user("previous prompt")];
    app.handle_key(KeyCode::Up, KeyModifiers::empty()).unwrap();
    assert_eq!(app.input, "previous prompt");
}

#[test]
fn key_dispatch_paste_inlines_small_text_at_cursor() {
    let mut app = create_test_app();
    super::handle_paste(&mut app, "pasted".to_string());
    assert_eq!(app.input, "pasted");
    assert_eq!(app.cursor_pos, 6);
    assert!(!app.is_processing);
}

#[test]
fn key_dispatch_shell_output_combines_stderr_section() {
    let (out, truncated) = combine_shell_output(b"ok\n", b"bad");
    assert_eq!(out, "ok\n[stderr]\nbad");
    assert!(!truncated);
}
