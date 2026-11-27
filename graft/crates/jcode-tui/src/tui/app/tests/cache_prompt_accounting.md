# crates/jcode-tui/src/tui/app/tests/cache_prompt_accounting.rs

- record_reported_test_cache_usage · function · L3-L10 — fn record_reported_test_cache_usage(app: &mut App) -> bool
- cache_accounting_cancelled_request_without_usage_keeps_previous_cache_state · function · L13-L63 — fn cache_accounting_cancelled_request_without_usage_keeps_previous_cache_state()
- cache_accounting_claude_reuses_written_boundary_not_uncached_tail · function · L66-L98 — fn cache_accounting_claude_reuses_written_boundary_not_uncached_tail()
- cache_accounting_remote_late_claude_usage_keeps_reusable_boundary · function · L101-L139 — fn cache_accounting_remote_late_claude_usage_keeps_reusable_boundary()
- cache_accounting_openai_app · function · L141-L149 — fn cache_accounting_openai_app() -> App
- cache_accounting_stats · function · L151-L154 — fn cache_accounting_stats(app: &mut App) -> String
- cache_accounting_openai_writes_are_subsets_live_and_completed · function · L157-L208 — fn cache_accounting_openai_writes_are_subsets_live_and_completed()
- cache_accounting_remote_snapshots_count_openai_prompt_once · function · L211-L249 — fn cache_accounting_remote_snapshots_count_openai_prompt_once()
- cache_accounting_mixed_history_and_live_sum_resolved_prompts · function · L252-L298 — fn cache_accounting_mixed_history_and_live_sum_resolved_prompts()
- cache_accounting_legacy_history_does_not_guess_from_writes_or_current_provider · function · L301-L332 — fn cache_accounting_legacy_history_does_not_guess_from_writes_or_current_provider()
- cache_accounting_fully_cached_anthropic_has_nonzero_denominator · function · L335-L350 — fn cache_accounting_fully_cached_anthropic_has_nonzero_denominator()
- cache_accounting_bare_command_reports_without_changing_preference · function · L353-L365 — fn cache_accounting_bare_command_reports_without_changing_preference()
- cache_accounting_missing_last_read_remains_unknown · function · L368-L382 — fn cache_accounting_missing_last_read_remains_unknown()
- cache_report_exposes_actual_expiry_notification_policy · function · L385-L420 — fn cache_report_exposes_actual_expiry_notification_policy()
