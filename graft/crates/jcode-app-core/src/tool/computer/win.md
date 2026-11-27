# crates/jcode-app-core/src/tool/computer/win.rs

- list_apps · function · L7-L17 — pub fn list_apps() -> Result<ToolOutput>
- activate_app · function · L19-L25 — pub fn activate_app(app: &str) -> Result<ToolOutput>
- hide_app · function · L27-L33 — pub fn hide_app(app: &str) -> Result<ToolOutput>
- quit_app · function · L35-L49 — pub fn quit_app(app: &str) -> Result<ToolOutput>
- list_windows · function · L52-L86 — pub fn list_windows() -> Result<ToolOutput>
- focus_window · function · L90-L98 — pub fn focus_window(app: &str) -> Result<ToolOutput>
- move_window · function · L100-L110 — pub fn move_window(app: &str, x: f64, y: f64) -> Result<ToolOutput>
- resize_window · function · L112-L122 — pub fn resize_window(app: &str, w: f64, h: f64) -> Result<ToolOutput>
- minimize_window · function · L124-L130 — pub fn minimize_window(app: &str) -> Result<ToolOutput>
- close_window · function · L132-L138 — pub fn close_window(app: &str) -> Result<ToolOutput>
