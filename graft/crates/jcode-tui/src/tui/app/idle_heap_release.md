# crates/jcode-tui/src/tui/app/idle_heap_release.rs

- IDLE_TRIM_AFTER · constant · L14-L14 — const IDLE_TRIM_AFTER: std::time::Duration = std::time::Duration::from_secs(60);
- RETENTION_CHECK_INTERVAL · constant · L20-L20 — const RETENTION_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
- CLIENT_RETENTION_TRIM_THRESHOLD_BYTES · constant · L27-L27 — const CLIENT_RETENTION_TRIM_THRESHOLD_BYTES: u64 = 16 * 1024 * 1024;
- IDLE_RETRIM_INTERVAL · constant · L35-L35 — const IDLE_RETRIM_INTERVAL: std::time::Duration = std::time::Duration::from_secs(300);
- IdleHeapRelease · struct · L38-L46 — pub(super) struct IdleHeapRelease
- maybe_release_idle_heap · function · L51-L101 — pub(super) fn maybe_release_idle_heap(&mut self)
- retention_check_due · function · L104-L106 — fn retention_check_due(last_check: Option<std::time::Instant>, now: std::time::Instant) -> bool
- client_retention_trim_threshold_bytes · function · L108-L114 — fn client_retention_trim_threshold_bytes(global_threshold: u64) -> u64
- idle_retrim_due · function · L118-L120 — fn idle_retrim_due(last_trim: Option<std::time::Instant>, now: std::time::Instant) -> bool
- tests · module · L123-L158 — mod tests
- retention_check_runs_immediately_then_obeys_interval · function · L127-L135 — fn retention_check_runs_immediately_then_obeys_interval()
- client_retention_threshold_caps_default_but_honors_lower_and_disabled · function · L138-L148 — fn client_retention_threshold_caps_default_but_honors_lower_and_disabled()
- idle_retrim_waits_for_first_trim_then_fires_on_cadence · function · L151-L157 — fn idle_retrim_waits_for_first_trim_then_fires_on_cadence()
