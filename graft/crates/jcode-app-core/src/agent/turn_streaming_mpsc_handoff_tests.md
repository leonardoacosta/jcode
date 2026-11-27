# crates/jcode-app-core/src/agent/turn_streaming_mpsc_handoff_tests.rs

- configured_streaming_budget_and_disabled_handoff · function · L4-L29 — async fn configured_streaming_budget_and_disabled_handoff()
- callback_paths_honor_custom_and_disabled_budgets · function · L32-L95 — async fn callback_paths_honor_custom_and_disabled_budgets()
- WaitProbe · struct · L97-L97 — struct WaitProbe;
- ask_user_waits_past_budget_and_returns_its_answer · function · L100-L126 — async fn ask_user_waits_past_budget_and_returns_its_answer()
- streaming_ask_user_wait_is_not_automatically_backgrounded · function · L129-L147 — async fn streaming_ask_user_wait_is_not_automatically_backgrounded()
- name · function · L151-L153 — fn name(&self) -> &str
- description · function · L154-L156 — fn description(&self) -> &str
- parameters_schema · function · L157-L159 — fn parameters_schema(&self) -> serde_json::Value
- execute · function · L160-L167 — async fn execute(
- nonstreaming_background_wait_returns_original_result_after_budget · function · L171-L194 — async fn nonstreaming_background_wait_returns_original_result_after_budget()
- background_wait_is_not_automatically_backgrounded · function · L197-L217 — async fn background_wait_is_not_automatically_backgrounded()
- slow_tool_handoff_occurs_after_three_seconds_without_cancelling_work · function · L220-L236 — async fn slow_tool_handoff_occurs_after_three_seconds_without_cancelling_work()
- fast_tool_finishes_in_foreground · function · L239-L247 — async fn fast_tool_finishes_in_foreground()
- manual_background_and_shutdown_do_not_wait_for_budget · function · L250-L263 — async fn manual_background_and_shutdown_do_not_wait_for_budget()
