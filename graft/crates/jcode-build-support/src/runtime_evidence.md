# crates/jcode-build-support/src/runtime_evidence.rs

- EvidenceVerdict · enum · L8-L12 — pub enum EvidenceVerdict
- RuntimeEvidence · struct · L16-L27 — pub struct RuntimeEvidence
- valid_hash · function · L29-L31 — fn valid_hash(hash: &str) -> bool
- same_revision · function · L33-L36 — fn same_revision(left: &str, right: &str) -> bool
- new · function · L39-L104 — pub fn new(
- capture · function · L106-L163 — pub fn capture(
- tests · module · L167-L379 — mod tests
- runtime_evidence_hidden_index_flags_are_not_clean_evidence · function · L172-L205 — fn runtime_evidence_hidden_index_flags_are_not_clean_evidence()
- runtime_evidence_capture_never_executes_repository_filters_or_fsmonitor · function · L208-L228 — fn runtime_evidence_capture_never_executes_repository_filters_or_fsmonitor()
- evidence · function · L230-L243 — fn evidence(
- server · function · L245-L252 — fn server(hash: &str, dirty: bool) -> RuntimeIdentity
- runtime_evidence_matched_clean_hashes_and_abbreviations · function · L255-L262 — fn runtime_evidence_matched_clean_hashes_and_abbreviations()
- runtime_evidence_unverified_client_cannot_match_clean_same_revision · function · L265-L275 — fn runtime_evidence_unverified_client_cannot_match_clean_same_revision()
- runtime_evidence_unverified_client_preserves_known_mismatches · function · L278-L308 — fn runtime_evidence_unverified_client_preserves_known_mismatches()
- runtime_evidence_client_provenance_json_is_additive_and_defaults_unverified · function · L311-L322 — fn runtime_evidence_client_provenance_json_is_additive_and_defaults_unverified()
- runtime_evidence_mismatch_names_stale_server · function · L325-L330 — fn runtime_evidence_mismatch_names_stale_server()
- runtime_evidence_dirty_never_matches · function · L333-L344 — fn runtime_evidence_dirty_never_matches()
- runtime_evidence_unknown_never_matches · function · L347-L378 — fn runtime_evidence_unknown_never_matches()
