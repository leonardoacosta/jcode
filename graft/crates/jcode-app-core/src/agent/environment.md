# crates/jcode-app-core/src/agent/environment.rs

- EnvSnapshotDetail · enum · L8-L11 — pub(super) enum EnvSnapshotDetail
- cached_git_state_for_dir · function · L13-L29 — pub(super) fn cached_git_state_for_dir(
- git_state_for_snapshot · function · L35-L65 — fn git_state_for_snapshot(dir: &Path) -> Option<GitState>
- IN_FLIGHT · constant · L36-L38 — static IN_FLIGHT: std::sync::LazyLock<
- set_log_context · function · L69-L78 — pub(super) fn set_log_context(&self)
- log_env_snapshot · function · L81-L83 — pub(super) fn log_env_snapshot(&mut self, reason: &str)
- log_env_snapshot_unsaved · function · L89-L91 — pub(super) fn log_env_snapshot_unsaved(&mut self, reason: &str)
- record_env_snapshot_inner · function · L93-L104 — fn record_env_snapshot_inner(&mut self, reason: &str, persist: bool)
- env_snapshot_detail · function · L106-L112 — pub(super) fn env_snapshot_detail(&self) -> EnvSnapshotDetail
- build_env_snapshot · function · L114-L166 — pub(super) fn build_env_snapshot(
- WARMING · constant · L125-L125 — static WARMING: std::sync::Once = std::sync::Once::new();
- tests · module · L170-L233 — mod tests
- MultiplexedSlotProvider · struct · L183-L183 — struct MultiplexedSlotProvider;
- complete · function · L187-L197 — async fn complete(
- name · function · L199-L201 — fn name(&self) -> &str
- display_name · function · L203-L205 — fn display_name(&self) -> String
- fork · function · L207-L209 — fn fork(&self) -> Arc<dyn Provider>
- log_context_names_the_profile_not_the_transport_slot · function · L216-L232 — async fn log_context_names_the_profile_not_the_transport_slot()
