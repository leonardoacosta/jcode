# crates/jcode-base/src/mobile_runner_local.rs

- MAX_REPORTED_SESSIONS · constant · L25-L25 — pub const MAX_REPORTED_SESSIONS: usize = 90;
- LiveSession · struct · L28-L32 — pub struct LiveSession
- BoxFuture · type · L34-L34 — type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
- LocalSessionHost · interface · L37-L58 — pub trait LocalSessionHost: Send + Sync + 'static
- live_sessions · function · L39-L39 — fn live_sessions(&self) -> BoxFuture<'_, Vec<LiveSession>>;
- deliver · function · L42-L42 — fn deliver<'a>(&'a self, session_id: &'a str, text: &'a str) -> BoxFuture<'a, Result<()>>;
- deliver_images · function · L44-L57 — fn deliver_images<'a>(
- enabled · function · L61-L67 — pub fn enabled() -> bool
- report · function · L71-L92 — fn report(sessions: Vec<LiveSession>) -> Vec<LiveSession>
- ClaimLocalRequest · struct · L96-L100 — struct ClaimLocalRequest<'a>
- ClaimLocalSessions · struct · L102-L102 — struct ClaimLocalSessions<'a>(&'a [LiveSession]);
- ClaimLocalSession · struct · L105-L108 — struct ClaimLocalSession<'a>
- serialize · function · L111-L120 — fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
- claim_local · function · L123-L147 — async fn claim_local(
- run_local_once · function · L151-L190 — pub async fn run_local_once(
- machine_name · function · L192-L194 — fn machine_name() -> String
- spawn_local_runner · function · L197-L229 — pub fn spawn_local_runner(host: Arc<dyn LocalSessionHost>)
- tests · module · L233-L233 — mod tests;
