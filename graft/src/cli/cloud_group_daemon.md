# src/cli/cloud_group_daemon.rs

- daemon_command · function · L1-L31 — fn daemon_command(action: &str, payload: &serde_json::Value) -> Result<serde_json::Value>
- select_group · function · L33-L90 — fn select_group(
- prepare_group · function · L92-L118 — fn prepare_group(group: &mut Group) -> Result<()>
- import_group · function · L120-L122 — fn import_group(bundle: &serde_json::Value) -> Result<()>
- standalone_without_daemon · function · L124-L162 — fn standalone_without_daemon(id: &str) -> Result<bool>
- json_mentions_session · function · L164-L171 — fn json_mentions_session(value: &serde_json::Value, id: &str) -> bool
