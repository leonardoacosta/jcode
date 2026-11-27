# crates/jcode-app-core/src/server/swarm/lifecycle.rs

- idle_spawned_worker_reap_candidates · function · L8-L20 — pub(in crate::server) fn idle_spawned_worker_reap_candidates(
- DeadMemberSalvage · struct · L24-L29 — pub(in crate::server) struct DeadMemberSalvage
- is_empty · function · L32-L34 — pub(in crate::server) fn is_empty(&self) -> bool
- describe · function · L37-L59 — fn describe(&self, worker_label: &str) -> String
- salvage_plan_assignments_of · function · L73-L122 — pub(in crate::server) fn salvage_plan_assignments_of(
- salvage_assignments_of_dead_member · function · L128-L186 — pub(in crate::server) async fn salvage_assignments_of_dead_member(
- notify_coordinator_of_salvage · function · L190-L226 — pub(in crate::server) async fn notify_coordinator_of_salvage(
- remove_session_from_swarm · function · L228-L463 — pub(in crate::server) async fn remove_session_from_swarm(
- member_awaits_unrestored_notice · function · L467-L478 — pub(in crate::server) fn member_awaits_unrestored_notice(member: &SwarmMember) -> bool
- notify_owner_of_unrestored_idle_workers · function · L490-L620 — pub(in crate::server) async fn notify_owner_of_unrestored_idle_workers(
- NOTICE_LOCK · constant · L499-L499 — static NOTICE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
- set_member_task_label · function · L626-L640 — pub(in crate::server) async fn set_member_task_label(
- update_member_status · function · L646-L668 — pub(in crate::server) async fn update_member_status(
- update_member_status_with_report · function · L674-L698 — pub(in crate::server) async fn update_member_status_with_report(
- update_member_status_with_report_tldr · function · L704-L966 — pub(in crate::server) async fn update_member_status_with_report_tldr(
