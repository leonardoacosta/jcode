# crates/jcode-app-core/src/agent/turn_loop_policy.rs

- MAX_CONTEXT_LIMIT_RETRIES · constant · L7-L7 — pub(in crate::agent) const MAX_CONTEXT_LIMIT_RETRIES: u32 = 5;
- MAX_INCOMPLETE_CONTINUATION_ATTEMPTS · constant · L8-L8 — pub(in crate::agent) const MAX_INCOMPLETE_CONTINUATION_ATTEMPTS: u32 = 3;
- MAX_EMPTY_POST_TOOL_CONTINUATION_ATTEMPTS · constant · L15-L15 — pub(crate) const MAX_EMPTY_POST_TOOL_CONTINUATION_ATTEMPTS: u32 = 5;
- SEQUENTIAL_TOOL_ROUNDS_BEFORE_BATCH_NUDGE · constant · L16-L16 — pub(in crate::agent) const SEQUENTIAL_TOOL_ROUNDS_BEFORE_BATCH_NUDGE: u32 = 3;
- BATCH_NUDGE · constant · L17-L17 — pub(in crate::agent) const BATCH_NUDGE: &str = "<system-reminder>Several tool calls have been made one at a time. If the next independent operations can run concurrently, use the batch tool instead of making more sequential calls. Keep sequential calls when one result is required to decide the next operation.</system-reminder>";
- TOOL_CALLS_BEFORE_TODO_NUDGE · constant · L18-L18 — pub(in crate::agent) const TOOL_CALLS_BEFORE_TODO_NUDGE: usize = 20;
- TODO_NUDGE · constant · L19-L19 — pub(in crate::agent) const TODO_NUDGE: &str = "<system-reminder>Many tool calls have been made but this session has no todos. Use the todo tool to plan and track this task.</system-reminder>";
- update_sequential_tool_rounds · function · L20-L30 — pub(in crate::agent) fn update_sequential_tool_rounds(
- should_inject_batch_nudge · function · L32-L37 — pub(in crate::agent) fn should_inject_batch_nudge(
- plan_limit_reminder · function · L39-L45 — pub(in crate::agent) fn plan_limit_reminder(
