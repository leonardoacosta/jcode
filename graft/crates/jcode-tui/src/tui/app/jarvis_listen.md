# crates/jcode-tui/src/tui/app/jarvis_listen.rs

- tests · module · L10-L31 — mod tests
- jarvis_listening_keeps_one_bounded_request_and_deduplicates · function · L14-L30 — fn jarvis_listening_keeps_one_bounded_request_and_deduplicates()
- AlwaysListening · struct · L34-L45 — pub(super) struct AlwaysListening
- drop · function · L47-L51 — fn drop(&mut self)
- queue_request · function · L54-L71 — fn queue_request(&mut self, text: String)
- initialize · function · L73-L111 — fn initialize(&mut self) -> anyhow::Result<()>
- suppress_speech · function · L113-L117 — pub(super) fn suppress_speech(&self)
- detach · function · L119-L126 — pub(super) fn detach(&mut self)
- toggle_always_listening · function · L130-L146 — pub(super) fn toggle_always_listening(&mut self)
- poll_jarvis_listening · function · L148-L310 — pub(super) async fn poll_jarvis_listening(&mut self, remote: &mut RemoteConnection) -> bool
