# crates/jcode-tui/src/tui/backend_screen.rs

- NEXT_SCREEN_ID · constant · L8-L8 — static NEXT_SCREEN_ID: AtomicU64 = AtomicU64::new(1 << 63);
- request · function · L9-L15 — fn request(session: &str, action: ScreenContextAction) -> Request
- ScreenFeed · struct · L17-L22 — pub(super) struct ScreenFeed
- drop · function · L24-L26 — fn drop(&mut self)
- write_control · function · L29-L51 — async fn write_control(
- wait_ack · function · L53-L64 — async fn wait_ack(acknowledged: &mut watch::Receiver<u64>, id: u64) -> Result<()>
- forward_latest · function · L66-L93 — async fn forward_latest(
- prepare_screen_context · function · L98-L148 — pub(crate) async fn prepare_screen_context(&mut self) -> Result<()>
- handle_screen_control_event · function · L152-L169 — pub(crate) fn handle_screen_control_event(&mut self, event: &ServerEvent) -> bool
- stop_screen_context · function · L171-L173 — pub(crate) fn stop_screen_context(&mut self)
- screen_context_call · function · L175-L219 — pub(crate) fn screen_context_call(
- tests · module · L223-L340 — mod tests
- screen_control_responses_are_separate_from_chat_requests · function · L226-L251 — async fn screen_control_responses_are_separate_from_chat_requests()
- screen_transport_retains_latest_when_a_hundred_updates_arrive · function · L254-L319 — async fn screen_transport_retains_latest_when_a_hundred_updates_arrive()
- screen_callback_rejects_unowned_session_without_capture · function · L322-L339 — async fn screen_callback_rejects_unowned_session_without_capture()
