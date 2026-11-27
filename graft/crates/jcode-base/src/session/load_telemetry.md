# crates/jcode-base/src/session/load_telemetry.rs

- BURST_THRESHOLD · constant · L23-L23 — const BURST_THRESHOLD: usize = 32;
- BURST_WINDOW · constant · L27-L27 — const BURST_WINDOW: Duration = Duration::from_millis(250);
- BACKTRACE_CAPTURED · constant · L32-L32 — static BACKTRACE_CAPTURED: AtomicBool = AtomicBool::new(false);
- BurstState · struct · L35-L48 — struct BurstState
- reset · function · L51-L58 — fn reset(&mut self)
- LoadLogDecision · enum · L68-L73 — pub(super) enum LoadLogDecision
- should_log · function · L76-L78 — pub(super) fn should_log(self) -> bool
- note_load · function · L83-L115 — pub(super) fn note_load(elapsed: Duration, bytes: u64) -> LoadLogDecision
- flush · function · L119-L121 — fn flush()
- announce_burst · function · L126-L140 — fn announce_burst(state: &BurstState)
- flush_locked · function · L142-L160 — fn flush_locked(state: &mut BurstState)
- thread_label · function · L162-L167 — fn thread_label() -> String
- compact_backtrace · function · L170-L183 — fn compact_backtrace() -> String
- tests · module · L186-L232 — mod tests
- contiguous_loads_are_suppressed_after_the_threshold · function · L190-L215 — fn contiguous_loads_are_suppressed_after_the_threshold()
- a_gap_longer_than_the_window_starts_a_new_run · function · L218-L231 — fn a_gap_longer_than_the_window_starts_a_new_run()
