# crates/jcode-base/src/provider/tests/session_binding.rs

- Observed · type · L3-L3 — type Observed = Arc<Mutex<Vec<(&'static str, Option<String>)>>>;
- SessionRecordingProvider · struct · L6-L11 — struct SessionRecordingProvider
- record · function · L14-L23 — fn record(&self, operation: &'static str)
- set_input_compression_profile · function · L28-L34 — fn set_input_compression_profile(
- set_session_id · function · L36-L38 — fn set_session_id(&self, session_id: &str)
- prewarm · function · L40-L42 — async fn prewarm(&self, _tools: &[ToolDefinition], _system_static: &str)
- complete · function · L44-L53 — async fn complete(
- name · function · L55-L57 — fn name(&self) -> &str
- set_model · function · L59-L61 — fn set_model(&self, _model: &str) -> anyhow::Result<()>
- fork · function · L63-L70 — fn fork(&self) -> Arc<dyn Provider>
- input_compression_binding_survives_runtime_replacement_and_isolates_forks · function · L74-L105 — fn input_compression_binding_survives_runtime_replacement_and_isolates_forks()
- RECORDING · constant · L76-L77 — static RECORDING: std::sync::LazyLock<Arc<SessionRecordingProvider>> =
- session_binding_reaches_prewarm_and_both_dispatch_modes_after_runtime_replacement · function · L108-L144 — fn session_binding_reaches_prewarm_and_both_dispatch_modes_after_runtime_replacement()
- session_binding_fork_retains_identity_and_rebinding_is_local_to_router · function · L147-L170 — fn session_binding_fork_retains_identity_and_rebinding_is_local_to_router()
- session_binding_new_session_fork_starts_unbound · function · L173-L207 — fn session_binding_new_session_fork_starts_unbound()
- OBSERVED · constant · L174-L175 — static OBSERVED: std::sync::LazyLock<Observed> =
