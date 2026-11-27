# crates/jcode-tui-mermaid/tests/layout_cache_memory_probe.rs

- vm_rss_bytes · function · L20-L34 — fn vm_rss_bytes() -> Option<u64>
- probe_diagram · function · L41-L51 — fn probe_diagram(idx: usize, nonce: u128) -> String
- layout_cache_stays_bounded_under_50_distinct_renders · function · L55-L169 — fn layout_cache_stays_bounded_under_50_distinct_renders()
- RENDERS · constant · L56-L56 — const RENDERS: usize = 50;
- APPROX_BYTES_CAP · constant · L58-L58 — const APPROX_BYTES_CAP: u64 = 2_500_000;
- RSS_DELTA_CAP · constant · L61-L61 — const RSS_DELTA_CAP: u64 = 50 * 1024 * 1024;
