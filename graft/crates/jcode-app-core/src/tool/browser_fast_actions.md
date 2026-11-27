# crates/jcode-app-core/src/tool/browser_fast_actions.rs

- OBSERVE_SCRIPT · constant · L6-L92 — pub(super) const OBSERVE_SCRIPT: &str = r#"return (() =>
- risky · function · L96-L156 — pub(super) fn risky(text: &str) -> bool
- candidates · function · L158-L346 — pub(super) fn candidates(input: &BrowserInput, observation: &Value) -> Result<Vec<Candidate>>
- tests · module · L349-L461 — mod tests
- input · function · L352-L359 — fn input() -> BrowserInput
- search_submission_requires_search_observation_and_caller_text · function · L362-L390 — fn search_submission_requires_search_observation_and_caller_text()
- nested_scroll_actions_are_scoped_and_directional · function · L393-L408 — fn nested_scroll_actions_are_scoped_and_directional()
- navigation_buttons_do_not_enable_side_effects · function · L411-L424 — fn navigation_buttons_do_not_enable_side_effects()
- observation_has_stable_per_document_nonce · function · L427-L432 — fn observation_has_stable_per_document_nonce()
- links_allow_self_but_not_other_browsing_contexts · function · L435-L445 — fn links_allow_self_but_not_other_browsing_contexts()
- form_buttons_remain_excluded_and_root_scroll_defaults_remain · function · L448-L460 — fn form_buttons_remain_excluded_and_root_scroll_defaults_remain()
