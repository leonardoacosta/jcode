## 1. Cell-width truncation

- [x] 1.1 Rewrite `truncate_smart` in `info_widget_text.rs` to measure and cut with `unicode-width`; tests for CJK, emoji and fitting text
- [x] 1.2 In `info_widget_swarm_background.rs`, subtract role-prefix and icon cell widths from the member-row budget; test full row width at narrow widths
- [x] 1.3 Make `clamp_line_to_width` in `info_widget_swarm_gallery.rs` append `…` when content is dropped; test hint row at width 30

## 2. Verification

- [x] 2.1 Apply Resolved decisions: document compact count wording; record caching deferral
- [x] 2.2 `cargo test -p jcode-tui info_widget`
- [x] 2.3 `openspec validate swarm-widget-cell-width --strict`
