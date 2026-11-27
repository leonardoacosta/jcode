# crates/jcode-base/src/provider/tests/startup_pool.rs

- startup_pool_hydrates_disk_only_exhaustion_before_route_and_request_guards · function · L4-L93 — fn startup_pool_hydrates_disk_only_exhaustion_before_route_and_request_guards()
- WORKER · constant · L5-L5 — const WORKER: &str = "JCODE_TEST_COLD_QUOTA_ROUTING";
- startup_pool_tiers_override_last_account_then_fall_back_when_exhausted · function · L96-L143 — fn startup_pool_tiers_override_last_account_then_fall_back_when_exhausted()
- startup_pool_ready_priority_head_wins_over_saved_provider_default · function · L146-L180 — fn startup_pool_ready_priority_head_wins_over_saved_provider_default()
- startup_pool_saved_account_does_not_pin_new_sessions · function · L183-L217 — fn startup_pool_saved_account_does_not_pin_new_sessions()
- startup_pool_head_binds_when_no_config_default_exists · function · L220-L260 — fn startup_pool_head_binds_when_no_config_default_exists()
- startup_pool_head_binds_when_config_default_fails_to_apply · function · L263-L281 — fn startup_pool_head_binds_when_config_default_fails_to_apply()
- startup_pool_fixture · function · L283-L298 — fn startup_pool_fixture() -> crate::auth::account_pool::AccountPool
- startup_pool_all_exhausted_never_falls_through_to_manual_api_default · function · L301-L325 — fn startup_pool_all_exhausted_never_falls_through_to_manual_api_default()
- startup_pool_priority_skips_cooldown_and_returns_after_reset_without_saving_fallback · function · L328-L369 — fn startup_pool_priority_skips_cooldown_and_returns_after_reset_without_saving_fallback()
- startup_pool_explicit_initial_provider_and_model_remain_authoritative · function · L372-L386 — fn startup_pool_explicit_initial_provider_and_model_remain_authoritative()
- startup_pool_never_uses_excluded_or_unenrolled_metered_keys · function · L389-L405 — fn startup_pool_never_uses_excluded_or_unenrolled_metered_keys()
- startup_pool_api_default_spellings_preserve_credential_pins · function · L408-L441 — fn startup_pool_api_default_spellings_preserve_credential_pins()
- startup_pool_named_defaults_are_preserved_but_unused_profiles_do_not_shadow_native · function · L444-L515 — fn startup_pool_named_defaults_are_preserved_but_unused_profiles_do_not_shadow_native()
- startup_pool_bare_default_rotates_ready_accounts_without_changing_saved_login · function · L518-L562 — fn startup_pool_bare_default_rotates_ready_accounts_without_changing_saved_login()
- startup_pool_excluded_oauth_default_is_not_a_pool_identity · function · L565-L589 — fn startup_pool_excluded_oauth_default_is_not_a_pool_identity()
- startup_pool_claude_default_distinguishes_api_only_auto_from_oauth_alias · function · L592-L638 — fn startup_pool_claude_default_distinguishes_api_only_auto_from_oauth_alias()
- startup_pool_compatible_aliases_rotate_only_enrolled_saved_logins · function · L641-L687 — fn startup_pool_compatible_aliases_rotate_only_enrolled_saved_logins()
