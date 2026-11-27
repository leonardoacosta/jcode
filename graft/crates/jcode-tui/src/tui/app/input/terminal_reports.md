# crates/jcode-tui/src/tui/app/input/terminal_reports.rs

- strip_terminal_control_sequences · function · L24-L94 — pub(in crate::tui::app) fn strip_terminal_control_sequences(
- csi_run_length · function · L101-L116 — fn csi_run_length(bytes: &[u8]) -> Option<usize>
- bare_terminal_report_length · function · L124-L141 — fn bare_terminal_report_length(bytes: &[u8]) -> Option<usize>
- REPORT_FINALS · constant · L139-L139 — const REPORT_FINALS: [u8; 6] = *b"Mm~RIO";
