# crates/jcode-app-core/src/server/reload_trace.rs

- MAX_TRACE_FILE_BYTES · constant · L14-L14 — const MAX_TRACE_FILE_BYTES: u64 = 16 * 1024 * 1024;
- sanitize_file_component · function · L16-L32 — fn sanitize_file_component(value: &str) -> String
- trace_dir · function · L34-L36 — fn trace_dir() -> Result<std::path::PathBuf>
- trace_path · function · L38-L40 — pub(super) fn trace_path(reload_id: &str) -> Result<std::path::PathBuf>
- record · function · L42-L135 — pub(super) fn record(reload_id: &str, phase: &str, mut fields: Map<String, Value>)
- WARNED · constant · L74-L74 — static WARNED: AtomicBool = AtomicBool::new(false);
- record_value · function · L137-L147 — pub(super) fn record_value(reload_id: &str, phase: &str, fields: Value)
- tests · module · L150-L219 — mod tests
- record_value_appends_jsonl_trace_event · function · L155-L186 — fn record_value_appends_jsonl_trace_event() -> anyhow::Result<()>
- record_value_stops_appending_past_size_cap · function · L189-L218 — fn record_value_stops_appending_past_size_cap() -> anyhow::Result<()>
