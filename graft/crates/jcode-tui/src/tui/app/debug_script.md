# crates/jcode-tui/src/tui/app/debug_script.rs

- build_debug_snapshot · function · L4-L42 — pub(in crate::tui::app) fn build_debug_snapshot(&self) -> DebugSnapshot
- eval_assertions · function · L44-L189 — fn eval_assertions(&self, assertions: &[DebugAssertion]) -> Vec<DebugAssertResult>
- handle_assertions · function · L191-L201 — pub(in crate::tui::app) fn handle_assertions(&mut self, raw: &str) -> String
- handle_script_run · function · L203-L241 — pub(in crate::tui::app) fn handle_script_run(&mut self, raw: &str) -> String
- handle_test_script · function · L243-L329 — pub(in crate::tui::app) fn handle_test_script(
- apply_wait_ms · function · L331-L341 — pub(in crate::tui::app) fn apply_wait_ms(&mut self, wait_ms: u64) -> String
- lookup_snapshot_value · function · L343-L370 — fn lookup_snapshot_value(&self, snapshot: &DebugSnapshot, field: &str) -> serde_json::Value
- lookup_json_path · function · L372-L388 — fn lookup_json_path(value: &serde_json::Value, parts: &[&str]) -> serde_json::Value
- execute_script_step · function · L390-L488 — fn execute_script_step(&mut self, step: &str) -> String
- check_debug_command · function · L490-L506 — pub(in crate::tui::app) fn check_debug_command(&mut self) -> Option<String>
- parse_key_spec · function · L508-L554 — pub(in crate::tui::app) fn parse_key_spec(
- parse_and_inject_key · function · L557-L565 — pub(in crate::tui::app) fn parse_and_inject_key(
