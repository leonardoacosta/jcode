# crates/jcode-tui/src/tui/app/auth_remote/onboarding.rs

- Onboarding · struct · L8-L11 — pub(in crate::tui::app) struct Onboarding
- dismiss · function · L14-L17 — pub(super) fn dismiss(&mut self)
- remote_has_no_logins · function · L20-L33 — fn remote_has_no_logins(providers: &[super::command::ProviderStatus]) -> bool
- poll_ssh_login_onboarding · function · L36-L89 — pub(in crate::tui::app) fn poll_ssh_login_onboarding(&mut self) -> bool
- open_ssh_import_offer · function · L91-L113 — fn open_ssh_import_offer(&mut self) -> bool
- open_ssh_import_decision · function · L115-L157 — pub(super) fn open_ssh_import_decision(&mut self, offer: bool)
- select_ssh_import_decision · function · L159-L184 — pub(in crate::tui::app) fn select_ssh_import_decision(&mut self, accept: bool)
- tests · module · L188-L347 — mod tests
- empty_status · function · L194-L203 — fn empty_status() -> Vec<ProviderStatus>
- queue_empty_status · function · L205-L212 — fn queue_empty_status(app: &mut App)
- ssh_onboarding_requires_complete_empty_status_including_api_keys · function · L215-L231 — fn ssh_onboarding_requires_complete_empty_status_including_api_keys()
- ssh_onboarding_offers_once_and_no_opens_normal_login_without_copying · function · L234-L258 — fn ssh_onboarding_offers_once_and_no_opens_normal_login_without_copying()
- ssh_onboarding_yes_chooses_provider_then_requires_separate_copy_consent · function · L261-L287 — fn ssh_onboarding_yes_chooses_provider_then_requires_separate_copy_consent()
- ssh_onboarding_pasted_yes_and_no_stay_private · function · L290-L308 — fn ssh_onboarding_pasted_yes_and_no_stay_private()
- ssh_onboarding_never_replaces_drafts_or_explicit_login · function · L311-L327 — fn ssh_onboarding_never_replaces_drafts_or_explicit_login()
- ssh_onboarding_unknown_status_never_claims_signed_out_or_retries · function · L330-L346 — fn ssh_onboarding_unknown_status_never_claims_signed_out_or_retries()
