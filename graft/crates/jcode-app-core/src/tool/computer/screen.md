# crates/jcode-app-core/src/tool/computer/screen.rs

- png_dimensions · function · L13-L21 — pub fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)>
- PNG_SIG · constant · L14-L14 — const PNG_SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
- capture_to_temp · function · L23-L51 — fn capture_to_temp(extra_args: &[&str]) -> Result<Vec<u8>>
- screenshot · function · L53-L76 — pub fn screenshot() -> Result<ToolOutput>
- window_screenshot · function · L79-L90 — pub fn window_screenshot(window_id: i64) -> Result<ToolOutput>
- ocr · function · L100-L153 — pub fn ocr(region: Option<[f64; 4]>) -> Result<ToolOutput>
- run_vision_ocr · function · L162-L262 — fn run_vision_ocr(image_path: &str, crop: Option<[f64; 4]>) -> Result<String>
- which_swift · function · L264-L279 — fn which_swift() -> Option<String>
- frontmost_app · function · L283-L287 — pub fn frontmost_app() -> Result<String>
