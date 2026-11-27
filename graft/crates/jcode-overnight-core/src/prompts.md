# crates/jcode-overnight-core/src/prompts.rs

- overnight_phase · function · L7-L28 — pub(crate) fn overnight_phase(manifest: &OvernightManifest, now: DateTime<Utc>) -> &'static str
- time_relation_to_target · function · L30-L40 — pub(crate) fn time_relation_to_target(manifest: &OvernightManifest, now: DateTime<Utc>) -> String
- relative_time · function · L42-L49 — pub(crate) fn relative_time(then: DateTime<Utc>, now: DateTime<Utc>) -> String
- next_prompt_label · function · L51-L89 — pub(crate) fn next_prompt_label(manifest: &OvernightManifest, now: DateTime<Utc>) -> String
- build_coordinator_prompt · function · L91-L151 — pub fn build_coordinator_prompt(
- build_visible_current_session_prompt · function · L153-L207 — pub fn build_visible_current_session_prompt(manifest: &OvernightManifest) -> String
- build_continuation_prompt · function · L209-L220 — pub fn build_continuation_prompt(manifest: &OvernightManifest) -> String
- build_handoff_ready_prompt · function · L222-L227 — pub fn build_handoff_ready_prompt(manifest: &OvernightManifest) -> String
- build_morning_report_prompt · function · L229-L235 — pub fn build_morning_report_prompt(manifest: &OvernightManifest) -> String
- build_post_wake_continuation_prompt · function · L237-L243 — pub fn build_post_wake_continuation_prompt(manifest: &OvernightManifest) -> String
- build_final_wrapup_prompt · function · L245-L251 — pub fn build_final_wrapup_prompt(manifest: &OvernightManifest) -> String
- prompt_event_summary · function · L253-L267 — pub fn prompt_event_summary(prompt: &str) -> String
