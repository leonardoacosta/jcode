# tests/context_window_matrix.rs

- SENTINEL_WINDOW · constant · L42-L42 — const SENTINEL_WINDOW: usize = 123_456;
- representative_models · function · L48-L57 — fn representative_models() -> Vec<(&'static str, Option<&'static str>)>
- resolved_windows_are_always_plausible · function · L65-L78 — fn resolved_windows_are_always_plausible()
- configured_window_wins_over_family_fallbacks · function · L89-L109 — fn configured_window_wins_over_family_fallbacks()
- resolution_is_deterministic · function · L117-L128 — fn resolution_is_deterministic()
- provider_hints_do_not_silently_widen_windows · function · L138-L159 — fn provider_hints_do_not_silently_widen_windows()
- unknown_models_fall_back_to_the_declared_default · function · L167-L175 — fn unknown_models_fall_back_to_the_declared_default()
- cache_lookups_do_not_match_unrelated_model_ids · function · L184-L194 — fn cache_lookups_do_not_match_unrelated_model_ids()
