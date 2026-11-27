# crates/jcode-app-core/src/server/mobile_local_tests.rs

- Fixture · struct · L8-L14 — struct Fixture
- fixture · function · L16-L47 — async fn fixture() -> Fixture
- until_done · function · L50-L64 — async fn until_done(events: &mut mpsc::UnboundedReceiver<ServerEvent>) -> Vec<ServerEvent>
- transcript · function · L66-L71 — async fn transcript(agent: &Arc<Mutex<Agent>>) -> String
- idle_session_is_reported_live_and_a_phone_message_starts_a_turn · function · L74-L101 — async fn idle_session_is_reported_live_and_a_phone_message_starts_a_turn()
- message_for_a_session_mid_turn_is_queued_behind_that_turn · function · L104-L140 — async fn message_for_a_session_mid_turn_is_queued_behind_that_turn()
- idle_session_that_is_briefly_locked_still_gets_a_turn · function · L143-L159 — async fn idle_session_that_is_briefly_locked_still_gets_a_turn()
- idle_session_that_stays_locked_refuses_instead_of_stranding_the_message · function · L162-L182 — async fn idle_session_that_stays_locked_refuses_instead_of_stranding_the_message()
- sessions_without_a_window_are_not_live_and_refuse_messages · function · L185-L206 — async fn sessions_without_a_window_are_not_live_and_refuse_messages()
- headless_sessions_are_live · function · L209-L220 — async fn headless_sessions_are_live()
- phone_photo_starts_same_session_turn_with_image_content · function · L223-L241 — async fn phone_photo_starts_same_session_turn_with_image_content()
- phone_photo_is_preserved_in_busy_session_queue · function · L244-L270 — async fn phone_photo_is_preserved_in_busy_session_queue()
