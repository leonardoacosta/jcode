# src/cli/cloud_move/transport/tests.rs

- RestoreEnv · struct · L3-L3 — struct RestoreEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);
- save · function · L6-L13 — fn save(names: &[&'static str]) -> Self
- drop · function · L17-L24 — fn drop(&mut self)
- preflight_cache_identity_tracks_account_endpoint_release_and_binary · function · L28-L56 — fn preflight_cache_identity_tracks_account_endpoint_release_and_binary() -> Result<()>
- managed_preparation_verifies_default_once_and_still_probes_custom_binary · function · L59-L110 — fn managed_preparation_verifies_default_once_and_still_probes_custom_binary() -> Result<()>
- concurrent_preflights_share_success_and_failed_checks_clear_marker · function · L113-L158 — fn concurrent_preflights_share_success_and_failed_checks_clear_marker() -> Result<()>
- scripted_target · function · L160-L169 — fn scripted_target(script: &str) -> Result<(tempfile::TempDir, Target)>
- ssh_retry_reports_the_underlying_failure · function · L172-L181 — fn ssh_retry_reports_the_underlying_failure()
- account · function · L183-L188 — fn account() -> CloudAccount
- custom_transport_preserves_remote_argument_quoting · function · L191-L209 — fn custom_transport_preserves_remote_argument_quoting() -> Result<()>
- empty_custom_transport_is_an_error · function · L212-L218 — fn empty_custom_transport_is_an_error()
- ssh_drop_is_distinct_from_remote_command_failure · function · L221-L238 — fn ssh_drop_is_distinct_from_remote_command_failure() -> Result<()>
- credential_import_rejects_unexpected_output_without_disclosing_it · function · L241-L253 — fn credential_import_rejects_unexpected_output_without_disclosing_it() -> Result<()>
- credential_export_returns_private_bytes · function · L256-L261 — fn credential_export_returns_private_bytes() -> Result<()>
- credential_failures_do_not_echo_remote_secrets · function · L264-L276 — fn credential_failures_do_not_echo_remote_secrets() -> Result<()>
