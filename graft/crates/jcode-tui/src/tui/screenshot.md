# crates/jcode-tui/src/tui/screenshot.rs

- SCREENSHOT_MODE · constant · L12-L12 — static SCREENSHOT_MODE: AtomicBool = AtomicBool::new(false);
- signal_dir · function · L15-L17 — fn signal_dir() -> PathBuf
- enable · function · L20-L25 — pub fn enable()
- disable · function · L28-L30 — pub fn disable()
- is_enabled · function · L33-L35 — pub fn is_enabled() -> bool
- signal_ready · function · L49-L70 — pub fn signal_ready(state_name: &str, metadata: serde_json::Value)
- clear_signal · function · L73-L76 — pub fn clear_signal(state_name: &str)
- clear_all_signals · function · L79-L92 — pub fn clear_all_signals()
- ScreenshotState · enum · L96-L113 — pub enum ScreenshotState
- as_str · function · L116-L127 — pub fn as_str(&self) -> &'static str
- signal · function · L130-L132 — pub fn signal(&self, metadata: serde_json::Value)
