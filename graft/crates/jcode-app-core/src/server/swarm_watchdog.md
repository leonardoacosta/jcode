# crates/jcode-app-core/src/server/swarm_watchdog.rs

- MAX_SWEEP_INTERVAL · constant · L33-L33 — const MAX_SWEEP_INTERVAL: Duration = Duration::from_secs(60);
- MIN_SWEEP_INTERVAL · constant · L35-L35 — const MIN_SWEEP_INTERVAL: Duration = Duration::from_secs(5);
- StallState · struct · L38-L43 — struct StallState
- STALLED · constant · L45-L46 — static STALLED: LazyLock<StdMutex<HashMap<String, StallState>>> =
- stall_after · function · L49-L52 — pub(super) fn stall_after() -> Option<Duration>
- sweep_interval · function · L55-L57 — pub(super) fn sweep_interval(stall_after: Duration) -> Duration
- stalled_for_secs · function · L61-L69 — pub(super) fn stalled_for_secs(session_id: &str) -> Option<u64>
- StallInputs · struct · L73-L81 — pub(super) struct StallInputs<'a>
- evaluate_stall · function · L85-L96 — pub(super) fn evaluate_stall(inputs: StallInputs<'_>, stall_after: Duration) -> Option<u64>
- NewStall · struct · L100-L105 — pub(super) struct NewStall
- update_stalls · function · L111-L176 — pub(super) fn update_stalls(
- format_duration · function · L178-L186 — fn format_duration(secs: u64) -> String
- coordinator_message · function · L188-L194 — pub(super) fn coordinator_message(stall: &NewStall) -> String
- nudge_message · function · L196-L201 — pub(super) fn nudge_message(silent_secs: u64) -> String
- sweep_stalled_workers · function · L205-L263 — pub(super) async fn sweep_stalled_workers(
- clear_for_tests · function · L266-L272 — pub(super) fn clear_for_tests(session_ids: &[&str])
- tests · module · L276-L276 — mod tests;
