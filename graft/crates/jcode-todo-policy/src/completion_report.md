# crates/jcode-todo-policy/src/completion_report.rs

- escape · function · L5-L21 — fn escape(value: &str) -> String
- text · function · L23-L28 — fn text(out: &mut String, label: &str, value: Option<&str>)
- history · function · L30-L40 — fn history(out: &mut String, label: &str, values: impl Iterator<Item = &'static str>)
- issue · function · L42-L46 — fn issue(out: &mut Vec<String>, label: &str, passes: bool)
- goal_issues · function · L48-L137 — fn goal_issues(goal: &TodoGoal) -> Vec<String>
- render_completion_report · function · L141-L360 — pub fn render_completion_report(todos: &[TodoItem], plan: &TodoPlan, goals: &[TodoGoal]) -> String
- past · function · L193-L207 — macro_rules! past
- field · function · L327-L331 — macro_rules! field
- hist · function · L332-L336 — macro_rules! hist
- tests · module · L364-L364 — mod tests;
