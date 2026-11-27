# crates/jcode-base/src/usage/anthropic_reset_tests.rs

- server · function · L6-L56 — async fn server(responses: Vec<(u16, Value)>) -> (String, tokio::task::JoinHandle<Vec<String>>)
- status · function · L58-L66 — fn status(available: bool, next: Option<&str>) -> Value
- profile · function · L68-L73 — fn profile() -> Value
- prepare · function · L75-L85 — async fn prepare(
- offer_requires_an_eligible_reset_arm · function · L88-L108 — fn offer_requires_an_eligible_reset_arm()
- availability_below_the_wall_is_not_hidden_or_invented · function · L111-L125 — fn availability_below_the_wall_is_not_hidden_or_invented()
- only_explicit_denial_is_reported_as_ineligible · function · L128-L138 — fn only_explicit_denial_is_reported_as_ineligible()
- preparation_is_read_only_and_pins_the_organization · function · L141-L158 — async fn preparation_is_read_only_and_pins_the_organization()
- spent_or_absent_offers_never_reach_the_profile_or_claim · function · L161-L177 — async fn spent_or_absent_offers_never_reach_the_profile_or_claim()
- a_login_without_an_organization_cannot_be_prepared · function · L180-L194 — async fn a_login_without_an_organization_cannot_be_prepared()
- claim_posts_the_program_to_the_pinned_organization · function · L197-L226 — async fn claim_posts_the_program_to_the_pinned_organization()
- outcomes_distinguish_cleared_limits_and_unknown_results · function · L229-L238 — fn outcomes_distinguish_cleared_limits_and_unknown_results()
