# crates/jcode-app-core/src/server/history_provider_tests.rs

- session_with_provider_key · function · L3-L11 — fn session_with_provider_key(key: Option<&str>) -> crate::session::Session
- history_provider_name_prefers_persisted_openai_key · function · L14-L20 — fn history_provider_name_prefers_persisted_openai_key()
- history_provider_name_preserves_unknown_runtime_profile · function · L23-L29 — fn history_provider_name_preserves_unknown_runtime_profile()
- history_provider_name_maps_a_compatible_profile_key_to_its_label · function · L35-L47 — fn history_provider_name_maps_a_compatible_profile_key_to_its_label()
- history_provider_name_maps_a_bare_compatible_profile_key · function · L53-L67 — fn history_provider_name_maps_a_bare_compatible_profile_key()
- history_provider_name_keeps_import_source_codes_verbatim · function · L73-L82 — fn history_provider_name_keeps_import_source_codes_verbatim()
- SlotOnlyProvider · struct · L88-L88 — struct SlotOnlyProvider;
- complete · function · L92-L102 — async fn complete(
- name · function · L104-L106 — fn name(&self) -> &str
- display_name · function · L108-L110 — fn display_name(&self) -> String
- fork · function · L112-L114 — fn fork(&self) -> Arc<dyn Provider>
- history_provider_name_falls_back_to_the_profile_label · function · L118-L124 — fn history_provider_name_falls_back_to_the_profile_label()
