# crates/jcode-app-core/src/server/mobile_local.rs

- IDLE_START_WAIT · constant · L25-L25 — const IDLE_START_WAIT: Duration = Duration::from_secs(4);
- IDLE_START_RETRY · constant · L26-L26 — const IDLE_START_RETRY: Duration = Duration::from_millis(100);
- DaemonSessions · struct · L28-L32 — pub(super) struct DaemonSessions
- member_state · function · L36-L42 — async fn member_state(&self, session_id: &str) -> Option<bool>
- live · function · L44-L68 — async fn live(&self) -> Vec<LiveSession>
- deliver_message · function · L70-L141 — async fn deliver_message(
- BoxFuture · type · L144-L144 — type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
- live_sessions · function · L147-L149 — fn live_sessions(&self) -> BoxFuture<'_, Vec<LiveSession>>
- deliver · function · L151-L153 — fn deliver<'a>(&'a self, session_id: &'a str, text: &'a str) -> BoxFuture<'a, Result<()>>
- deliver_images · function · L155-L162 — fn deliver_images<'a>(
- spawn · function · L165-L167 — pub(super) fn spawn(host: DaemonSessions)
