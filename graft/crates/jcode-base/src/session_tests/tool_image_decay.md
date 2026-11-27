# crates/jcode-base/src/session_tests/tool_image_decay.rs

- SCREENSHOT_CHARS · constant · L7-L7 — const SCREENSHOT_CHARS: usize = 64 * 1024;
- add_screenshot · function · L9-L24 — fn add_screenshot(session: &mut Session, index: usize)
- inline_image_count · function · L26-L32 — fn inline_image_count(messages: &[StoredMessage]) -> usize
- computer_use_screenshots_stay_bounded_in_saved_transcript · function · L35-L82 — fn computer_use_screenshots_stay_bounded_in_saved_transcript() -> Result<()>
- loading_legacy_transcript_decays_old_screenshots · function · L85-L122 — fn loading_legacy_transcript_decays_old_screenshots() -> Result<()>
