# crates/jcode-tui/src/tui/app/auth/auth_muse.rs

- is_device_code_progress · function · L17-L19 — pub(crate) fn is_device_code_progress(provider: &str) -> bool
- is_muse_subscription_attempt_event · function · L24-L26 — pub(crate) fn is_muse_subscription_attempt_event(provider: &str) -> bool
- pending_attempt_matches · function · L32-L40 — pub(crate) fn pending_attempt_matches(pending: &str, incoming: &str) -> bool
- next_slow_down_interval · function · L46-L48 — pub(crate) fn next_slow_down_interval(current_interval: u64, retry_after: u64) -> u64
- is_completed_login · function · L54-L58 — pub(crate) fn is_completed_login(login: &LoginCompleted) -> bool
- is_muse_subscription_profile · function · L63-L67 — pub(super) fn is_muse_subscription_profile(
- logout_provider · function · L71-L74 — pub(super) fn logout_provider() -> anyhow::Result<String>
- logout_all · function · L78-L83 — pub(super) fn logout_all(summary: &mut Vec<String>, errors: &mut Vec<String>)
- start_copilot_login · function · L86-L188 — pub(super) fn start_copilot_login(&mut self)
- start_muse_subscription_login · function · L198-L385 — pub(super) fn start_muse_subscription_login(&mut self)
- clear_muse_login_input · function · L397-L402 — fn clear_muse_login_input(&mut self)
- clear_matching_muse_pending · function · L404-L416 — fn clear_matching_muse_pending(&mut self, attempt: &str)
- take_muse_subscription_approval · function · L427-L498 — pub(super) fn take_muse_subscription_approval(&mut self, login: &LoginCompleted) -> bool
