# crates/jcode-voice/src/timing.rs

- ORIGIN · constant · L7-L7 — static ORIGIN: Mutex<Option<Instant>> = Mutex::new(None);
- RELEASE · constant · L8-L8 — static RELEASE: Mutex<Option<Instant>> = Mutex::new(None);
- enabled · function · L10-L13 — pub fn enabled() -> bool
- ON · constant · L11-L11 — static ON: OnceLock<bool> = OnceLock::new();
- begin · function · L16-L22 — pub fn begin()
- release · function · L25-L30 — pub fn release()
- mark · function · L33-L48 — pub fn mark(stage: &str)
