# crates/jcode-app-core/src/server/assignment_targets.rs

- filter_swarm_agent_candidates · function · L20-L35 — pub(super) fn filter_swarm_agent_candidates<'a>(
- is_drivable_auto_worker · function · L39-L41 — pub(super) fn is_drivable_auto_worker(member: &SwarmMember, req_session_id: &str) -> bool
- member_has_active_assignment · function · L50-L55 — pub(super) fn member_has_active_assignment(
- AUTO_ASSIGN_CLAIM_TTL · constant · L59-L59 — const AUTO_ASSIGN_CLAIM_TTL: std::time::Duration = std::time::Duration::from_secs(15);
- auto_assign_claims · function · L73-L78 — pub(super) fn auto_assign_claims() -> &'static std::sync::Mutex<HashMap<String, std::time::Instant>>
- CLAIMS · constant · L75-L76 — static CLAIMS: std::sync::OnceLock<std::sync::Mutex<HashMap<String, std::time::Instant>>> =
- auto_assign_claim_key · function · L80-L82 — pub(super) fn auto_assign_claim_key(swarm_id: &str, session_id: &str) -> String
- try_claim_auto_assign_target · function · L86-L99 — pub(super) fn try_claim_auto_assign_target(swarm_id: &str, session_id: &str) -> bool
- release_auto_assign_claim · function · L101-L106 — pub(super) fn release_auto_assign_claim(swarm_id: &str, session_id: &str)
- no_auto_target_error · function · L111-L123 — pub(super) fn no_auto_target_error(busy_skipped: usize) -> String
- select_and_claim_auto_target · function · L132-L149 — pub(super) fn select_and_claim_auto_target(
