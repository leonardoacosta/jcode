# crates/jcode-tui/src/tui/app/commands_plan.rs

- PlanCommand · struct · L7-L9 — pub(super) struct PlanCommand
- parse_plan_command · function · L11-L25 — pub(super) fn parse_plan_command(trimmed: &str) -> Option<PlanCommand>
- build_plan_prompt · function · L27-L46 — pub(super) fn build_plan_prompt(goal: Option<&str>) -> String
- plan_launch_notice · function · L48-L62 — pub(super) fn plan_launch_notice(goal: Option<&str>, interrupted: bool) -> String
- handle_plan_command_local · function · L64-L92 — pub(super) fn handle_plan_command_local(app: &mut App, command: PlanCommand)
- tests · module · L95-L145 — mod tests
- parse_plan_accepts_bare_and_goal_forms · function · L99-L114 — fn parse_plan_accepts_bare_and_goal_forms()
- parse_plan_rejects_other_commands · function · L117-L121 — fn parse_plan_rejects_other_commands()
- build_plan_prompt_is_plan_only_and_targets_plan_card · function · L124-L133 — fn build_plan_prompt_is_plan_only_and_targets_plan_card()
- launch_notice_encourages_conversation_before_planning · function · L136-L144 — fn launch_notice_encourages_conversation_before_planning()
