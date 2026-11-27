# crates/jcode-tui/src/tui/app/remote/swarm_plan_core.rs

- RemoteSwarmPlanSnapshot · struct · L5-L12 — pub(super) struct RemoteSwarmPlanSnapshot
- status_notice · function · L15-L68 — pub fn status_notice(&self) -> String
- tests · module · L72-L203 — mod tests
- plan_item · function · L77-L88 — fn plan_item(id: &str, status: &str) -> PlanItem
- snapshot · function · L90-L100 — fn snapshot(items: Vec<PlanItem>, summary: Option<PlanGraphStatus>) -> RemoteSwarmPlanSnapshot
- summary_fixture · function · L102-L118 — fn summary_fixture() -> PlanGraphStatus
- fixture_items · function · L120-L127 — fn fixture_items() -> Vec<PlanItem>
- swarm_plan_status_notice_includes_graph_hints · function · L130-L138 — fn swarm_plan_status_notice_includes_graph_hints()
- swarm_plan_status_notice_prefers_summary_item_count · function · L141-L146 — fn swarm_plan_status_notice_prefers_summary_item_count()
- swarm_plan_status_notice_empty_plan_without_summary · function · L149-L152 — fn swarm_plan_status_notice_empty_plan_without_summary()
- swarm_plan_status_notice_empty_summary_has_no_graph_suffix · function · L155-L159 — fn swarm_plan_status_notice_empty_summary_has_no_graph_suffix()
- swarm_plan_status_notice_all_done_shows_only_done_segment · function · L162-L183 — fn swarm_plan_status_notice_all_done_shows_only_done_segment()
- swarm_plan_status_notice_reports_cycles · function · L186-L194 — fn swarm_plan_status_notice_reports_cycles()
- swarm_plan_status_notice_reports_unresolved_dependencies · function · L197-L202 — fn swarm_plan_status_notice_reports_unresolved_dependencies()
