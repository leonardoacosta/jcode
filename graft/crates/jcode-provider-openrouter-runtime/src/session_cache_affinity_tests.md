# crates/jcode-provider-openrouter-runtime/src/session_cache_affinity_tests.rs

- profile · function · L6-L12 — fn profile(enabled: bool) -> jcode_base::config::NamedProviderConfig
- provider · function · L14-L16 — fn provider(name: &str, enabled: bool) -> OpenRouterProvider
- capture · function · L19-L97 — fn capture(
- affinity_config_defaults_off_and_round_trips_opt_in · function · L100-L117 — fn affinity_config_defaults_off_and_round_trips_opt_in()
- affinity_wire_default_off_and_both_protocols_opt_in · function · L120-L146 — fn affinity_wire_default_off_and_both_protocols_opt_in()
- affinity_lifecycle_is_stable_scoped_opaque_and_forks_rebind · function · L149-L189 — fn affinity_lifecycle_is_stable_scoped_opaque_and_forks_rebind()
- affinity_explicit_overrides_and_retries_remain_coherent · function · L192-L218 — fn affinity_explicit_overrides_and_retries_remain_coherent()
- affinity_conflicts_and_invalid_explicit_keys_fail_before_transport · function · L221-L249 — fn affinity_conflicts_and_invalid_explicit_keys_fail_before_transport()
- affinity_rejects_duplicate_and_auth_header_conflicts · function · L252-L266 — fn affinity_rejects_duplicate_and_auth_header_conflicts()
- affinity_strict_profile_stays_off_and_explicit_body_is_untouched · function · L269-L282 — fn affinity_strict_profile_stays_off_and_explicit_body_is_untouched()
- affinity_inferred_responses_preserves_final_key · function · L285-L304 — fn affinity_inferred_responses_preserves_final_key()
- affinity_header_only_override_sets_matching_body · function · L307-L326 — fn affinity_header_only_override_sets_matching_body()
- affinity_generated_identity_is_stable_across_retries_and_model_changes · function · L329-L344 — fn affinity_generated_identity_is_stable_across_retries_and_model_changes()
- affinity_review_rejects_padded_overrides_before_any_http · function · L347-L382 — fn affinity_review_rejects_padded_overrides_before_any_http()
- affinity_review_validates_before_cold_catalog_and_routing_http · function · L385-L423 — fn affinity_review_validates_before_cold_catalog_and_routing_http()
