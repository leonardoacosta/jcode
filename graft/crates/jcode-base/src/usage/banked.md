# crates/jcode-base/src/usage/banked.rs

- AccountBankedResets · struct · L12-L19 — pub struct AccountBankedResets
- earliest_expiry · function · L23-L36 — pub fn earliest_expiry(expirations: &[Option<String>]) -> Option<String>
- anthropic_offers · function · L39-L42 — fn anthropic_offers() -> &'static Mutex<HashMap<String, AccountBankedResets>>
- OFFERS · constant · L40-L40 — static OFFERS: OnceLock<Mutex<HashMap<String, AccountBankedResets>>> = OnceLock::new();
- record_anthropic_offer · function · L46-L73 — pub(super) fn record_anthropic_offer(
- cached_account_banked_resets · function · L76-L105 — pub fn cached_account_banked_resets(
- try_cache_lock · function · L107-L116 — fn try_cache_lock<T>(mutex: &Mutex<T>) -> Option<std::sync::MutexGuard<'_, T>>
- tests · module · L119-L172 — mod tests
- invalid_expirations_do_not_hide_valid_ones · function · L123-L133 — fn invalid_expirations_do_not_hide_valid_ones()
- busy_and_poisoned_caches_are_unavailable · function · L136-L146 — fn busy_and_poisoned_caches_are_unavailable()
- anthropic_offer_is_recorded_and_cleared · function · L149-L171 — fn anthropic_offer_is_recorded_and_cleared()
