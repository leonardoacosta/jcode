# crates/jcode-tui/src/tui/session_picker/search_speed_tests.rs

- WORDS · constant · L8-L8 — const WORDS: &str = "config deployment performance synchronization throughput ";
- TYPO · constant · L9-L9 — const TYPO: &str = "conifg";
- ABSENT · constant · L10-L10 — const ABSENT: &str = "quartz";
- repeated_metadata · function · L12-L16 — fn repeated_metadata(bytes: usize) -> String
- fixtures · function · L18-L33 — fn fixtures(count: usize, bytes: usize, missing: &Path) -> Vec<SessionInfo>
- sample_count · function · L35-L45 — fn sample_count() -> usize
- report · function · L47-L67 — fn report(
- worker_sample · function · L71-L106 — fn worker_sample(
- benchmark_resume_search_speed · function · L110-L236 — fn benchmark_resume_search_speed()
- search_speed_fixture_preserves_typo_and_absent_semantics · function · L239-L252 — fn search_speed_fixture_preserves_typo_and_absent_semantics()
