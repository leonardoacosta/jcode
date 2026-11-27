# crates/jcode-base/src/auth/test_sandbox.rs

- AuthTestSandbox · struct · L6-L10 — pub struct AuthTestSandbox
- new · function · L13-L41 — pub fn new() -> anyhow::Result<Self>
- root · function · L43-L45 — pub fn root(&self) -> &Path
- config_dir · function · L47-L49 — pub fn config_dir(&self) -> PathBuf
- external_dir · function · L51-L53 — pub fn external_dir(&self) -> PathBuf
- env_file_path · function · L55-L57 — pub fn env_file_path(&self, file_name: &str) -> PathBuf
- write_env_file · function · L59-L71 — pub fn write_env_file(
- write_openai_compatible_api_key · function · L73-L80 — pub fn write_openai_compatible_api_key(
- drop · function · L84-L93 — fn drop(&mut self)
- reset_global_auth_state · function · L96-L100 — fn reset_global_auth_state()
- tracked_env_vars · function · L102-L157 — fn tracked_env_vars() -> Vec<String>
- tests · module · L160-L200 — mod tests
- sandbox_isolates_jcode_home_and_config_dir · function · L164-L178 — fn sandbox_isolates_jcode_home_and_config_dir()
- sandbox_openai_compatible_key_file_is_read_from_temp_config · function · L181-L199 — fn sandbox_openai_compatible_key_file_is_read_from_temp_config()
