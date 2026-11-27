# crates/jcode-app-core/src/agent/prompting.rs

- runtime_skills_snapshot · function · L6-L24 — pub(crate) fn runtime_skills_snapshot(&self) -> serde_json::Value
- runtime_agent_model_overrides · function · L26-L28 — pub(crate) fn runtime_agent_model_overrides(&self) -> serde_json::Value
- runtime_context_snapshot · function · L30-L35 — pub(crate) async fn runtime_context_snapshot(&self) -> serde_json::Value
- prepare_debug_context · function · L39-L44 — pub async fn prepare_debug_context(&mut self) -> serde_json::Value
- debug_context · function · L48-L74 — pub async fn debug_context(&self) -> serde_json::Value
- log_prompt_prefix_accounting · function · L76-L88 — pub(super) fn log_prompt_prefix_accounting(
- build_memory_prompt_nonblocking_shared · function · L90-L129 — pub(super) fn build_memory_prompt_nonblocking_shared(
- build_system_prompt_split · function · L133-L181 — pub(super) fn build_system_prompt_split(
- append_session_system_prompt · function · L183-L193 — fn append_session_system_prompt(&self, mut prompt: String) -> String
- build_memory_prompt_nonblocking · function · L197-L203 — pub(super) fn build_memory_prompt_nonblocking(
