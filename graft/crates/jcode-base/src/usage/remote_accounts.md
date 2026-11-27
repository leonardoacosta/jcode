# crates/jcode-base/src/usage/remote_accounts.rs

- RemoteAccounts · struct · L7-L12 — struct RemoteAccounts
- REMOTE · constant · L14-L14 — static REMOTE: OnceLock<Mutex<RemoteAccounts>> = OnceLock::new();
- try_remote_lock · function · L16-L27 — fn try_remote_lock(
- clear_remote_account_usage · function · L30-L41 — pub fn clear_remote_account_usage() -> u64
- remote_account_usage_generation · function · L44-L50 — pub(super) fn remote_account_usage_generation() -> u64
- remote_account_usage_received · function · L53-L58 — pub fn remote_account_usage_received() -> bool
- set_remote_account_usage · function · L63-L68 — pub fn set_remote_account_usage(
- set_remote_account_usage_with_age · function · L70-L83 — pub(super) fn set_remote_account_usage_with_age(
- import_remote_account_usage · function · L85-L95 — pub(super) fn import_remote_account_usage(
- import_snapshot · function · L98-L112 — fn import_snapshot(
- remote_account_list · function · L116-L136 — pub fn remote_account_list() -> Option<(Vec<AccountUsageSnapshot>, Vec<AccountUsageSnapshot>)>
- cached · function · L138-L148 — pub(super) fn cached(
- lookup · function · L151-L179 — fn lookup(
- remote_account_banked_resets · function · L183-L202 — pub fn remote_account_banked_resets(
- tests · module · L205-L302 — mod tests
- delayed_snapshot_cannot_overwrite_live_or_reconnected_usage · function · L209-L226 — fn delayed_snapshot_cannot_overwrite_live_or_reconnected_usage()
- connection_reset_is_not_an_empty_authoritative_response · function · L229-L240 — fn connection_reset_is_not_an_empty_authoritative_response()
- empty_remote_and_expired_remote_are_unknown · function · L243-L301 — fn empty_remote_and_expired_remote_are_unknown()
