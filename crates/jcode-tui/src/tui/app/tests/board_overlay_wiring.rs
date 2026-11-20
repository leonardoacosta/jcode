#[test]
fn board_overlay_wiring_local_keys_route_first_and_esc_closes() {
    use crate::tui::board_overlay::{BoardAction, BoardLoad};
    let mut app = create_test_app();
    app.input = "draft".to_string();
    app.open_board_overlay(BoardLoad::Loading);
    assert!(app.board_overlay.is_some());

    // Keys go to the overlay, never the composer.
    app.handle_key(KeyCode::Char('x'), KeyModifiers::empty())
        .unwrap();
    assert_eq!(app.input, "draft");

    app.handle_key(KeyCode::Char('r'), KeyModifiers::empty())
        .unwrap();
    assert_eq!(app.take_board_action(), Some(BoardAction::Refresh));
    assert_eq!(app.take_board_action(), None);

    app.update_board_overlay(BoardLoad::Empty);
    app.handle_key(KeyCode::Esc, KeyModifiers::empty()).unwrap();
    assert!(app.board_overlay.is_none());
    assert_eq!(app.input, "draft");

    // A late result after close must not reopen the overlay.
    app.update_board_overlay(BoardLoad::Loading);
    assert!(app.board_overlay.is_none());
}

#[test]
fn board_overlay_wiring_remote_keys_route_first_and_esc_closes() {
    use crate::tui::board_overlay::{BoardAction, BoardLoad};
    let mut app = create_test_app();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let mut remote = crate::tui::backend::RemoteConnection::dummy();
    app.input = "draft".to_string();
    app.open_board_overlay(BoardLoad::Loading);

    rt.block_on(app.handle_remote_key(KeyCode::Char('r'), KeyModifiers::empty(), &mut remote))
        .unwrap();
    assert_eq!(app.take_board_action(), Some(BoardAction::Refresh));
    assert_eq!(app.input, "draft");

    rt.block_on(app.handle_remote_key(KeyCode::Esc, KeyModifiers::empty(), &mut remote))
        .unwrap();
    assert!(app.board_overlay.is_none());
}
