# crates/jcode-base/src/config/task_policy.rs

- validate_task_policy · function · L5-L8 — pub(crate) fn validate_task_policy(&self) -> anyhow::Result<()>
- policy_from_toml · function · L11-L32 — pub fn policy_from_toml(
- policy_from_table · function · L35-L59 — fn policy_from_table(table: toml::Table, source: &str) -> anyhow::Result<serde_json::Value>
- effective_task_policy · function · L62-L70 — pub fn effective_task_policy() -> anyhow::Result<serde_json::Value>
- tests · module · L74-L108 — mod tests
- config_task_policy_old_config_and_field_provenance · function · L78-L88 — fn config_task_policy_old_config_and_field_provenance()
- config_task_policy_invalid_limits_and_project_overrides_rejected · function · L91-L107 — fn config_task_policy_invalid_limits_and_project_overrides_rejected()
