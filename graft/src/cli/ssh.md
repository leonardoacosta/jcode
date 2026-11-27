# src/cli/ssh.rs

- CLIENT_RELOAD · constant · L8-L8 — static CLIENT_RELOAD: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
- stash_client_reload · function · L10-L14 — pub(crate) fn stash_client_reload(session_id: String)
- client_reload_command · function · L16-L48 — fn client_reload_command(
- validate · function · L50-L88 — fn validate(args: &Args) -> Result<()>
- run · function · L90-L101 — pub(crate) async fn run(args: Args) -> Result<()>
- run_unix · function · L104-L230 — async fn run_unix(args: Args) -> Result<()>
- quote · function · L232-L234 — fn quote(value: &str) -> String
- last_transfer_for_host · function · L236-L247 — fn last_transfer_for_host<'a>(
- resume_hint · function · L250-L267 — pub(crate) fn resume_hint(session_id: &str) -> Option<String>
- tests · module · L270-L420 — mod tests
- ssh_client_reload_preserves_remote_transport_and_workspace · function · L275-L334 — fn ssh_client_reload_preserves_remote_transport_and_workspace()
- transfer_timestamp_requires_matching_host_and_valid_lease · function · L337-L358 — fn transfer_timestamp_requires_matching_host_and_valid_lease()
- remote_modes_accept_explicit_remote_ids_without_local_lookup · function · L361-L375 — fn remote_modes_accept_explicit_remote_ids_without_local_lookup()
- remote_modes_reject_local_only_operations_before_connecting · function · L378-L391 — fn remote_modes_reject_local_only_operations_before_connecting()
- resume_hint_retains_remote_identity_and_quotes_paths · function · L394-L419 — fn resume_hint_retains_remote_identity_and_quotes_paths()
