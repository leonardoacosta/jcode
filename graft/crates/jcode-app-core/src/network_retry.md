# crates/jcode-app-core/src/network_retry.rs

- NetworkWaitPlan · struct · L9-L12 — pub struct NetworkWaitPlan
- classify_network_interruption · function · L14-L23 — pub fn classify_network_interruption(error: &(dyn std::error::Error + 'static)) -> Option<String>
- classify_message · function · L25-L27 — pub fn classify_message(message: &str) -> Option<String>
- classify_text · function · L29-L76 — fn classify_text(text: &str) -> Option<String>
- wait_plan · function · L78-L104 — pub fn wait_plan() -> NetworkWaitPlan
- wait_until_probably_online · function · L106-L115 — pub async fn wait_until_probably_online()
- is_probably_online · function · L117-L119 — pub async fn is_probably_online() -> bool
- probe_connectivity · function · L121-L127 — async fn probe_connectivity() -> bool
- wait_for_platform_change_or_delay · function · L129-L147 — async fn wait_for_platform_change_or_delay(delay: Duration)
- command_exists · function · L150-L161 — async fn command_exists(command: &str) -> bool
- shell_escape · function · L164-L166 — fn shell_escape(value: &str) -> String
- wait_for_command_output · function · L169-L186 — async fn wait_for_command_output(command: &str, args: &[&str])
- tests · module · L189-L268 — mod tests
- classifies_common_network_errors · function · L193-L198 — fn classifies_common_network_errors()
- classifies_real_world_outage_errors · function · L204-L245 — fn classifies_real_world_outage_errors()
- does_not_classify_deterministic_failures · function · L251-L267 — fn does_not_classify_deterministic_failures()
