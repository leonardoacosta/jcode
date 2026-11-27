# src/cli/cloud_runtime.rs

- HELPER · constant · L12-L12 — const HELPER: &str = "/opt/jcode-cloud/runtime-update";
- LIVE_RELOAD_MARKER · constant · L15-L15 — const LIVE_RELOAD_MARKER: &str = "jcode-managed-cloud-live-reload-v1";
- MAX_PACKAGE_BYTES · constant · L16-L16 — const MAX_PACKAGE_BYTES: u64 = 256 * 1024 * 1024;
- remote_command · function · L20-L39 — fn remote_command(verb: &str, value: Option<&str>) -> Result<String>
- reload_command · function · L45-L138 — fn reload_command(wait_secs: u64, force: bool) -> String
- run_remote · function · L140-L152 — fn run_remote(script: &str, input: Option<File>) -> Result<std::process::ExitStatus>
- reload · function · L154-L161 — fn reload(wait_secs: u64, force: bool) -> Result<()>
- build_from_source · function · L165-L225 — fn build_from_source() -> Result<std::path::PathBuf>
- valid_release_tag · function · L227-L239 — fn valid_release_tag(value: &str) -> bool
- package · function · L241-L262 — fn package(path: &Path) -> Result<(File, String)>
- invoke · function · L264-L281 — fn invoke(verb: &str, value: Option<&str>, input: Option<File>) -> Result<()>
- run_cli · function · L283-L331 — pub(crate) fn run_cli(action: CloudRuntimeCommand) -> Result<()>
- tests · module · L334-L446 — mod tests
- runtime_remote_arguments_cannot_inject_shell_commands · function · L339-L373 — fn runtime_remote_arguments_cannot_inject_shell_commands()
- live_reload_marker_matches_daemon_capability · function · L376-L390 — fn live_reload_marker_matches_daemon_capability()
- runtime_reload_requires_a_verified_nonnegative_idle_count · function · L393-L423 — fn runtime_reload_requires_a_verified_nonnegative_idle_count()
- runtime_upload_hashes_and_rewinds_the_same_open_file · function · L426-L435 — fn runtime_upload_hashes_and_rewinds_the_same_open_file()
- runtime_rejects_laptop_executables_and_oversized_artifacts · function · L438-L445 — fn runtime_rejects_laptop_executables_and_oversized_artifacts()
