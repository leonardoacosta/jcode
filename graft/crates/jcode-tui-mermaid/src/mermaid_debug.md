# crates/jcode-tui-mermaid/src/mermaid_debug.rs

- percentile_summary · function · L3-L26 — fn percentile_summary(samples_ms: &[f64]) -> MermaidTimingSummary
- diff_counter · function · L28-L30 — fn diff_counter(after: u64, before: u64) -> u64
- debug_stats_delta · function · L32-L55 — fn debug_stats_delta(
- debug_stats · function · L57-L80 — pub fn debug_stats() -> MermaidDebugStats
- reset_debug_stats · function · L82-L86 — pub fn reset_debug_stats()
- debug_stats_json · function · L88-L90 — pub fn debug_stats_json() -> Option<serde_json::Value>
- debug_cache · function · L92-L106 — pub fn debug_cache() -> Vec<MermaidCacheEntry>
- debug_memory_profile · function · L108-L186 — pub fn debug_memory_profile() -> MermaidMemoryProfile
- debug_memory_benchmark · function · L188-L230 — pub fn debug_memory_benchmark(iterations: usize) -> MermaidMemoryBenchmark
- debug_flicker_benchmark · function · L232-L338 — pub fn debug_flicker_benchmark(steps: usize) -> MermaidFlickerBenchmark
- synth_png_b64 · function · L342-L358 — fn synth_png_b64(seed: u32, w: u32, h: u32) -> String
- debug_image_scroll_benchmark · function · L378-L501 — pub fn debug_image_scroll_benchmark(
- scan_cache_dir_png_usage · function · L503-L520 — fn scan_cache_dir_png_usage(cache_dir: &Path) -> (usize, u64)
- max_opt_u64 · function · L522-L529 — fn max_opt_u64(a: Option<u64>, b: Option<u64>) -> Option<u64>
- diff_u64 · function · L531-L537 — fn diff_u64(after: u64, before: u64) -> i64
- diff_opt_u64 · function · L539-L544 — fn diff_opt_u64(after: Option<u64>, before: Option<u64>) -> Option<i64>
- parse_proc_status_kib_line · function · L548-L552 — fn parse_proc_status_kib_line(line: &str, key: &str) -> Option<u64>
- parse_proc_status_value_bytes · function · L556-L560 — pub(super) fn parse_proc_status_value_bytes(status: &str, key: &str) -> Option<u64>
