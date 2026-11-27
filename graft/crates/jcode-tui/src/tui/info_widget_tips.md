# crates/jcode-tui/src/tui/info_widget_tips.rs

- TIP_CYCLE_SECONDS · constant · L3-L3 — const TIP_CYCLE_SECONDS: u64 = 15;
- STATUS_TIP_PERIOD_SECONDS · constant · L4-L4 — const STATUS_TIP_PERIOD_SECONDS: u64 = 90;
- STATUS_TIP_OFFSET_SECONDS · constant · L5-L5 — const STATUS_TIP_OFFSET_SECONDS: u64 = 28;
- STATUS_TIP_SHOW_SECONDS · constant · L6-L6 — const STATUS_TIP_SHOW_SECONDS: u64 = 12;
- Tip · struct · L8-L10 — struct Tip
- all_tips · function · L12-L35 — fn all_tips() -> Vec<Tip>
- TIP_STATE · constant · L37-L37 — static TIP_STATE: Mutex<Option<(usize, Instant)>> = Mutex::new(None);
- current_tip · function · L39-L56 — fn current_tip(_max_width: usize) -> Tip
- occasional_status_tip · function · L58-L77 — pub(crate) fn occasional_status_tip(max_width: usize, elapsed_secs: u64) -> Option<String>
- wrap_tip_text · function · L79-L100 — fn wrap_tip_text(text: &str, width: usize) -> Vec<String>
- render_tips_widget · function · L103-L124 — pub(super) fn render_tips_widget(inner: Rect) -> super::frame::Framed
- footer_tests · module · L127-L136 — mod footer_tests
- tip_footer_explains_rotation_not_content · function · L131-L135 — fn tip_footer_explains_rotation_not_content()
