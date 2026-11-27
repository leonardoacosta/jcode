# crates/jcode-tui/src/tui/info_widget_cache.rs

- render_kv_cache_widget · function · L4-L62 — pub(super) fn render_kv_cache_widget(data: &InfoWidgetData, inner: Rect) -> Framed
- kv_cache_pcts · function · L64-L80 — pub(super) fn kv_cache_pcts(cache: &CacheHitInfo) -> Option<(u8, Option<u8>, Option<u8>, Color)>
- render_kv_cache_title · function · L83-L105 — pub(super) fn render_kv_cache_title(cache: &CacheHitInfo) -> Line<'static>
- render_kv_cache_rates_line · function · L108-L133 — pub(super) fn render_kv_cache_rates_line(cache: &CacheHitInfo) -> Line<'static>
- render_kv_cache_summary_line · function · L136-L195 — pub(super) fn render_kv_cache_summary_line(cache: &CacheHitInfo) -> Line<'static>
- ratio_pct · function · L197-L199 — pub(super) fn ratio_pct(ratio: f32) -> u8
- kv_cache_optimal_color · function · L201-L208 — pub(super) fn kv_cache_optimal_color(pct: u8) -> Color
- format_cache_turn_label · function · L210-L216 — pub(super) fn format_cache_turn_label(turn_number: usize, call_index: u16) -> String
- compact_token_count · function · L218-L226 — pub(super) fn compact_token_count(tokens: u64) -> String
