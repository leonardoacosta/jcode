# crates/jcode-app-core/src/update_metadata.rs

- UpdateMetadata · struct · L16-L30 — pub struct UpdateMetadata
- default · function · L33-L42 — fn default() -> Self
- load · function · L46-L54 — pub fn load() -> Result<Self>
- save · function · L56-L64 — pub fn save(&self) -> Result<()>
- should_check · function · L66-L74 — pub fn should_check(&self) -> bool
- rate_limited_backoff_remaining · function · L78-L84 — pub fn rate_limited_backoff_remaining(&self) -> Option<Duration>
- metadata_path · function · L87-L89 — pub(super) fn metadata_path() -> Result<PathBuf>
- record_release_update_duration · function · L91-L96 — pub(super) fn record_release_update_duration(duration: Duration)
- record_source_update_duration · function · L98-L103 — pub(super) fn record_source_update_duration(duration: Duration)
- tests · module · L106-L133 — mod tests
- respects_rate_limit_backoff · function · L110-L123 — fn respects_rate_limit_backoff()
- enforces_check_interval · function · L126-L132 — fn enforces_check_interval()
