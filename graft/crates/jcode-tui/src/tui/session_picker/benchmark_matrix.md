# crates/jcode-tui/src/tui/session_picker/benchmark_matrix.rs

- TestTerminal · type · L9-L9 — type TestTerminal = Terminal<TestBackend>;
- distribution_ms · function · L12-L30 — pub(super) fn distribution_ms(samples: &[Duration]) -> Value
- sample_count · function · L32-L38 — pub(super) fn sample_count() -> usize
- report · function · L40-L51 — pub(super) fn report(scenario: &str, details: Value, metrics: Value)
- resident_picker · function · L53-L75 — fn resident_picker(count: usize, turns: usize) -> SessionPicker
- terminal · function · L77-L79 — fn terminal(width: u16, height: u16) -> TestTerminal
- draw · function · L81-L83 — fn draw(picker: &mut SessionPicker, terminal: &mut TestTerminal)
- navigate · function · L85-L94 — fn navigate(picker: &mut SessionPicker, code: KeyCode) -> Duration
- clear_renderer_caches · function · L96-L102 — fn clear_renderer_caches()
- unique_transcript · function · L104-L114 — fn unique_transcript(picker: &mut SessionPicker, sample: usize)
- GENERATION · constant · L105-L105 — static GENERATION: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
- benchmark_distributions_have_honest_empty_and_tail_semantics · function · L117-L131 — fn benchmark_distributions_have_honest_empty_and_tail_semantics()
- benchmark_resident_fixtures_never_launch_disk_or_workspace_reads · function · L134-L149 — fn benchmark_resident_fixtures_never_launch_disk_or_workspace_reads()
- benchmark_resume_experience_matrix · function · L153-L287 — fn benchmark_resume_experience_matrix()
- benchmark_resume_content_matrix · function · L291-L360 — fn benchmark_resume_content_matrix()
