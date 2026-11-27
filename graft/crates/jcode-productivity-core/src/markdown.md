# crates/jcode-productivity-core/src/markdown.rs

- human · function · L6-L16 — pub fn human(n: u64) -> String
- hour_label · function · L18-L25 — fn hour_label(h: u8) -> String
- hour_label_pub · function · L28-L30 — pub fn hour_label_pub(h: u8) -> String
- render_markdown · function · L33-L157 — pub fn render_markdown(r: &ProductivityReport) -> String
- bar_for · function · L159-L170 — fn bar_for(count: u64, max: u64, width: usize) -> String
- sparkline · function · L173-L188 — pub fn sparkline(hist: &[u32; 24]) -> String
- BLOCKS · constant · L174-L174 — const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
