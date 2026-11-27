# src/bin/session_memory_bench.rs

- Args · struct · L11-L43 — struct Args
- Scenario · enum · L46-L49 — enum Scenario
- BenchMode · enum · L52-L57 — enum BenchMode
- main · function · L59-L145 — fn main() -> anyhow::Result<()>
- benchmark_metadata · function · L147-L204 — fn benchmark_metadata(
- redact_report · function · L206-L226 — fn redact_report(value: &mut serde_json::Value, home: Option<&str>)
- load_or_build_session · function · L228-L247 — fn load_or_build_session(args: &Args) -> anyhow::Result<Session>
- build_synthetic_session · function · L249-L293 — fn build_synthetic_session(turns: usize, tool_input_kib: usize, tool_output_kib: usize) -> Session
- build_side_panel · function · L295-L322 — fn build_side_panel(args: &Args) -> SidePanelSnapshot
- text_block · function · L324-L329 — fn text_block(text: String) -> ContentBlock
- tests · module · L332-L370 — mod tests
- missing_binary_identity_is_unknown_and_non_comparable · function · L336-L342 — fn missing_binary_identity_is_unknown_and_non_comparable()
- metadata_records_identity_and_hardware · function · L345-L352 — fn metadata_records_identity_and_hardware()
- report_redacts_home_and_secrets_without_changing_measurements · function · L355-L369 — fn report_redacts_home_and_secrets_without_changing_measurements()
- make_blob · function · L372-L384 — fn make_blob(prefix: &str, target_len: usize) -> String
- CHUNK · constant · L378-L378 — const CHUNK: &str = "abcdefghijklmnopqrstuvwxyz0123456789 ";
