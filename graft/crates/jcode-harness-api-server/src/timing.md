# crates/jcode-harness-api-server/src/timing.rs

- MAX_BYTES · constant · L12-L12 — const MAX_BYTES: u64 = 4 * 1024 * 1024;
- SLOW_TRANSLATE · constant · L14-L14 — pub const SLOW_TRANSLATE: Duration = Duration::from_millis(20);
- path · function · L16-L21 — fn path() -> Option<std::path::PathBuf>
- log · function · L23-L60 — pub fn log(line: &str)
- FILE · constant · L24-L24 — static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
- ms · function · L62-L64 — pub fn ms(d: Duration) -> f64
- Pending · struct · L68-L71 — pub struct Pending
- Instant_ · struct · L73-L73 — struct Instant_(Instant);
- default · function · L75-L77 — fn default() -> Self
- connection_age · function · L81-L83 — pub fn connection_age(&self) -> Duration
- begin · function · L86-L92 — pub fn begin(&mut self, id: u64, kind: &str, received: Instant, translate: Duration)
- finish · function · L96-L105 — pub fn finish(&mut self, reply_to: u64, event: &str, bytes: usize, source: &str)
