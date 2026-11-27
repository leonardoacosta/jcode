# crates/jcode-base/src/provider/catalog_scheduler.rs

- SWEEP_INTERVAL · constant · L28-L28 — const SWEEP_INTERVAL: Duration = Duration::from_secs(60);
- INITIAL_SWEEP_DELAY · constant · L33-L33 — const INITIAL_SWEEP_DELAY: Duration = Duration::from_secs(10);
- SCHEDULER_STARTED · constant · L35-L35 — static SCHEDULER_STARTED: OnceLock<()> = OnceLock::new();
- bump_catalog_generation · function · L40-L42 — pub fn bump_catalog_generation()
- ensure_started · function · L49-L69 — pub(crate) fn ensure_started()
- profile_catalog_cache_needs_refresh · function · L76-L85 — pub(crate) fn profile_catalog_cache_needs_refresh(
- sweep_stale_profile_catalogs · function · L90-L129 — fn sweep_stale_profile_catalogs() -> usize
- tests · module · L132-L161 — mod tests
- ensure_started_is_a_noop_without_a_runtime · function · L136-L140 — fn ensure_started_is_a_noop_without_a_runtime()
- ensure_started_is_idempotent_under_a_runtime · function · L143-L148 — async fn ensure_started_is_idempotent_under_a_runtime()
- sweep_interval_is_shorter_than_the_cache_staleness_window · function · L151-L160 — fn sweep_interval_is_shorter_than_the_cache_staleness_window()
