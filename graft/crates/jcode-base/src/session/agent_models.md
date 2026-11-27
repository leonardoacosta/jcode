# crates/jcode-base/src/session/agent_models.rs

- AgentModelOverrides · type · L7-L7 — pub type AgentModelOverrides = BTreeMap<String, String>;
- UPDATE_LOCK · constant · L11-L11 — static UPDATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
- path · function · L13-L20 — fn path(id: &str) -> Result<std::path::PathBuf>
- read_saved · function · L24-L38 — fn read_saved(path: &std::path::Path) -> Option<AgentModelOverrides>
- read_agent_model_overrides · function · L43-L48 — pub fn read_agent_model_overrides(id: &str) -> AgentModelOverrides
- refresh_agent_model_overrides · function · L51-L56 — pub fn refresh_agent_model_overrides(&mut self) -> Result<()>
- set_agent_model_override · function · L60-L83 — pub fn set_agent_model_override(&mut self, target: &str, model: Option<String>) -> Result<()>
- effective_agent_model · function · L87-L93 — pub fn effective_agent_model(&self, target: &str, global: Option<String>) -> Option<String>
- tests · module · L97-L263 — mod tests
- with_temp_home · function · L100-L114 — fn with_temp_home(f: impl FnOnce(&std::path::Path))
- override_writes_keep_transcript_backup_intact · function · L117-L142 — fn override_writes_keep_transcript_backup_intact()
- corrupt_override_file_without_backup_does_not_block_session · function · L145-L175 — fn corrupt_override_file_without_backup_does_not_block_session()
- session_agent_models_isolate_clear_inherit_and_resume · function · L178-L262 — fn session_agent_models_isolate_clear_inherit_and_resume()
