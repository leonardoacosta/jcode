# crates/jcode-app-core/src/ambient/prompt.rs

- MemoryGraphHealth · struct · L11-L20 — pub struct MemoryGraphHealth
- RecentSessionInfo · struct · L24-L30 — pub struct RecentSessionInfo
- ResourceBudget · struct · L34-L40 — pub struct ResourceBudget
- gather_memory_graph_health · function · L43-L103 — pub fn gather_memory_graph_health(
- gather_feedback_memories · function · L112-L170 — pub fn gather_feedback_memories(memory_manager: &crate::memory::MemoryManager) -> Vec<String>
- gather_recent_sessions · function · L173-L270 — pub fn gather_recent_sessions(since: Option<DateTime<Utc>>) -> Vec<RecentSessionInfo>
- RECENT_SESSION_LIMIT · constant · L191-L191 — const RECENT_SESSION_LIMIT: usize = 20;
- build_ambient_system_prompt · function · L276-L521 — pub fn build_ambient_system_prompt(
- format_scheduled_session_message · function · L523-L552 — pub fn format_scheduled_session_message(item: &ScheduledItem) -> String
- format_duration_rough · function · L555-L573 — pub(crate) fn format_duration_rough(d: chrono::Duration) -> String
- format_minutes_human · function · L577-L597 — pub fn format_minutes_human(mins: u32) -> String
