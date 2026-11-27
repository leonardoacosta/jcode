# crates/jcode-tui/src/tui/cloud_git_stderr.rs

- MAX_LINE · constant · L4-L4 — const MAX_LINE: usize = 192;
- SCAN_BUDGET · constant · L5-L5 — const SCAN_BUDGET: usize = 64 * 1024;
- RECORD_BUDGET · constant · L6-L6 — const RECORD_BUDGET: usize = 128;
- timing_record · function · L8-L38 — fn timing_record(line: &[u8]) -> Option<String>
- drain · function · L40-L75 — pub(super) async fn drain(reader: impl AsyncRead + Unpin, mut emit: impl FnMut(String))
- ProcessGroup · struct · L78-L78 — pub(super) struct ProcessGroup(pub(super) Option<u32>);
- drop · function · L80-L87 — fn drop(&mut self)
- tests · module · L92-L92 — mod tests;
