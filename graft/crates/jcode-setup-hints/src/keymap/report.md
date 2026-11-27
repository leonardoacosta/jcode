# crates/jcode-setup-hints/src/keymap/report.rs

- render_report · function · L12-L79 — pub fn render_report(cfg: &KeybindingsConfig, snapshot: &KeymapSnapshot) -> String
- render_conflict_block · function · L81-L112 — fn render_conflict_block(c: &Conflict) -> String
- render_status_line · function · L116-L131 — pub fn render_status_line(cfg: &KeybindingsConfig, snapshot: &KeymapSnapshot) -> Option<String>
- tests · module · L134-L190 — mod tests
- snapshot_with · function · L139-L148 — fn snapshot_with(bindings: Vec<DiscoveredBinding>) -> KeymapSnapshot
- term · function · L150-L158 — fn term(keys: &str, action: &str) -> DiscoveredBinding
- report_lists_conflicts_with_field_names · function · L161-L169 — fn report_lists_conflicts_with_field_names()
- report_says_clean_when_no_conflicts · function · L172-L177 — fn report_says_clean_when_no_conflicts()
- status_line_present_only_on_conflict · function · L180-L189 — fn status_line_present_only_on_conflict()
