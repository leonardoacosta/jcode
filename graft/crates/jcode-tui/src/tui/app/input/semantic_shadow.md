# crates/jcode-tui/src/tui/app/input/semantic_shadow.rs

- OBSERVER · constant · L12-L12 — pub(super) static OBSERVER: OnceLock<ShadowObserver> = OnceLock::new();
- ShadowObserver · struct · L14-L18 — pub(super) struct ShadowObserver
- default · function · L21-L27 — fn default() -> Self
- PendingRecord · struct · L30-L35 — struct PendingRecord<F: FnOnce(ShadowRecord)>
- drop · function · L38-L46 — fn drop(&mut self)
- submit · function · L50-L122 — pub(super) fn submit(
- tests · module · L126-L431 — mod tests
- LayaEnv · struct · L131-L131 — struct LayaEnv(Option<std::ffi::OsString>);
- drop · function · L133-L138 — fn drop(&mut self)
- check_evidence · function · L141-L154 — fn check_evidence() -> ContinuationEvidence
- semantic_shadow_loopback_verdicts_failures_caps_and_route_changes · function · L161-L335 — async fn semantic_shadow_loopback_verdicts_failures_caps_and_route_changes()
- semantic_shadow_disabled_never_initializes_and_unconfigured_is_uncertain · function · L338-L368 — async fn semantic_shadow_disabled_never_initializes_and_unconfigured_is_uncertain()
- semantic_shadow_cancellation_records_started_call_and_releases_permit · function · L375-L430 — async fn semantic_shadow_cancellation_records_started_call_and_releases_permit()
