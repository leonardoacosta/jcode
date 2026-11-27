# crates/jcode-tui/src/tui/session_picker/benchmark_io.rs

- POLL · constant · L9-L9 — const POLL: Duration = Duration::from_millis(16);
- LIMIT · constant · L10-L10 — const LIMIT: Duration = Duration::from_secs(10);
- NEEDLE · constant · L11-L11 — const NEEDLE: &str = "middleonlyplatypus";
- HomeGuard · struct · L13-L16 — struct HomeGuard
- install · function · L18-L28 — fn install(path: &Path) -> Self
- drop · function · L31-L44 — fn drop(&mut self)
- fixtures · function · L47-L107 — fn fixtures(home: &Path, count: usize) -> Vec<SessionInfo>
- Measurements · struct · L110-L116 — struct Measurements
- record · function · L118-L135 — fn record(
- row · function · L136-L144 — fn row(&self, scenario: &str, source: &str, attempts: usize) -> Value
- distribution · function · L146-L158 — fn distribution(values: &[Duration]) -> Value
- poll_until · function · L160-L174 — fn poll_until(mut done: impl FnMut() -> bool) -> (Duration, Duration, bool)
- type_query · function · L175-L182 — fn type_query(picker: &mut SessionPicker, query: &str)
- resume_bench_disk_preview_and_search · function · L186-L293 — fn resume_bench_disk_preview_and_search()
- benchmark_io_distribution_and_failures_are_explicit · function · L296-L309 — fn benchmark_io_distribution_and_failures_are_explicit()
- benchmark_io_real_fixture_load_and_search · function · L312-L340 — fn benchmark_io_real_fixture_load_and_search()
