# crates/jcode-tui/src/tui/ui/todo_completion_flash.rs

- TODO_COMPLETION_FLASH_MS · constant · L11-L11 — pub(crate) const TODO_COMPLETION_FLASH_MS: u64 = 1200;
- TITLE · constant · L12-L12 — const TITLE: &str = "Todo completion";
- FlashState · struct · L15-L19 — struct FlashState
- with_state · function · L21-L39 — fn with_state<R>(f: impl FnOnce(&mut FlashState) -> R) -> R
- STATE · constant · L31-L32 — static STATE: std::sync::OnceLock<std::sync::Mutex<FlashState>> =
- update · function · L43-L67 — pub(crate) fn update(messages: &[DisplayMessage], now_ms: u64) -> Option<(usize, f32)>
- active · function · L70-L76 — pub(crate) fn active(now_ms: u64) -> bool
- apply · function · L81-L95 — pub(crate) fn apply(line: &mut Line<'static>, t: f32)
- tests · module · L98-L133 — mod tests
- report · function · L101-L105 — fn report(content: &str) -> DisplayMessage
- flashes_only_new_reports_and_expires · function · L108-L120 — fn flashes_only_new_reports_and_expires()
- apply_changes_text_color_but_not_background · function · L123-L132 — fn apply_changes_text_color_but_not_background()
