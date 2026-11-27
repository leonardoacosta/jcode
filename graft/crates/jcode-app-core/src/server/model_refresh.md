# crates/jcode-app-core/src/server/model_refresh.rs

- REFRESH_INTERVAL · constant · L9-L9 — const REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
- REFRESH_TIMEOUT · constant · L10-L10 — const REFRESH_TIMEOUT: Duration = Duration::from_secs(2 * 60);
- WATCHED · constant · L14-L15 — static WATCHED: LazyLock<Mutex<HashMap<usize, Weak<dyn Provider>>>> =
- ensure_periodic_refresh · function · L17-L41 — pub(super) fn ensure_periodic_refresh(provider: &Arc<dyn Provider>) -> bool
- refresh_periodically · function · L43-L72 — async fn refresh_periodically(provider: Weak<dyn Provider>)
- tests · module · L75-L163 — mod tests
- Probe · struct · L79-L82 — struct Probe
- complete · function · L86-L94 — async fn complete(
- name · function · L95-L97 — fn name(&self) -> &str
- model · function · L98-L100 — fn model(&self) -> String
- fork · function · L101-L103 — fn fork(&self) -> Arc<dyn Provider>
- prefetch_models · function · L104-L106 — async fn prefetch_models(&self) -> anyhow::Result<()>
- refresh_model_catalog · function · L107-L115 — async fn refresh_model_catalog(
- periodic_model_refresh_runs_every_half_hour_and_deduplicates_attaches · function · L119-L142 — async fn periodic_model_refresh_runs_every_half_hour_and_deduplicates_attaches()
- periodic_model_refresh_recovers_from_a_hung_catalog_request · function · L145-L162 — async fn periodic_model_refresh_recovers_from_a_hung_catalog_request()
