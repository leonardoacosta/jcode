# crates/jcode-base/src/mobile_runner_events.rs

- tests · module · L15-L15 — mod tests;
- Source · type · L17-L17 — type Source = Arc<dyn Fn() -> Option<(String, String)> + Send + Sync>;
- Wakeups · struct · L19-L22 — pub(super) struct Wakeups
- drop · function · L25-L27 — fn drop(&mut self)
- wait · function · L31-L46 — pub(super) async fn wait(&mut self, delay: Duration)
- spawn · function · L49-L57 — pub(super) fn spawn() -> Wakeups
- event_request · function · L59-L85 — fn event_request(
- available · function · L87-L95 — fn available(message: &str) -> bool
- Hint · struct · L90-L93 — struct Hint
- spawn_with · function · L97-L176 — fn spawn_with(source: Source, retry: Duration) -> Wakeups
