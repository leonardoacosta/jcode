# crates/jcode-app-core/src/agent/cache_monitor_tests.rs

- CompactionCacheProvider · struct · L6-L10 — struct CompactionCacheProvider
- complete · function · L14-L54 — async fn complete(
- name · function · L56-L58 — fn name(&self) -> &str
- model · function · L60-L62 — fn model(&self) -> String
- fork · function · L64-L66 — fn fork(&self) -> Arc<dyn Provider>
- native_compaction_cache_refresh_is_classified_before_compaction_is_announced · function · L70-L167 — async fn native_compaction_cache_refresh_is_classified_before_compaction_is_announced()
- AccountFailoverProvider · struct · L173-L176 — struct AccountFailoverProvider
- complete · function · L180-L201 — async fn complete(
- name · function · L203-L205 — fn name(&self) -> &str
- model · function · L207-L209 — fn model(&self) -> String
- active_resolved_credential · function · L211-L213 — fn active_resolved_credential(&self) -> Option<jcode_provider_core::ResolvedCredential>
- oauth_account_label · function · L215-L217 — fn oauth_account_label(&self) -> Option<String>
- fork · function · L219-L221 — fn fork(&self) -> Arc<dyn Provider>
- run_account_turns · function · L224-L255 — async fn run_account_turns(switch_account: bool) -> Vec<ServerEvent>
- account_failover_miss_names_both_accounts_with_readable_counts · function · L258-L285 — async fn account_failover_miss_names_both_accounts_with_readable_counts()
- same_account_zero_read_explains_what_was_ruled_out · function · L288-L309 — async fn same_account_zero_read_explains_what_was_ruled_out()
