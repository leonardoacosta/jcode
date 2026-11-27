# crates/jcode-provider-core/src/attempt_tracker.rs

- stream_event_is_replay_visible · function · L31-L69 — fn stream_event_is_replay_visible(event: &StreamEvent) -> bool
- AttemptGuard · struct · L74-L77 — pub struct AttemptGuard
- finish · function · L83-L86 — pub async fn finish(mut self) -> bool
- drop · function · L90-L92 — fn drop(&mut self)
- track_attempt_output · function · L103-L138 — pub fn track_attempt_output(
- retry_backoff_delay · function · L146-L151 — pub fn retry_backoff_delay(attempt: u32, base_ms: u64) -> std::time::Duration
- tests · module · L154-L310 — mod tests
- tracker_cancellation_consumer_drop_closes_idle_attempt · function · L158-L167 — async fn tracker_cancellation_consumer_drop_closes_idle_attempt()
- tracker_cancellation_guard_drop_stops_idle_forwarder · function · L170-L177 — async fn tracker_cancellation_guard_drop_stops_idle_forwarder()
- text_and_tool_events_are_replay_visible · function · L180-L194 — fn text_and_tool_events_are_replay_visible()
- keyed_tool_events_are_replay_visible · function · L197-L211 — fn keyed_tool_events_are_replay_visible()
- model_metadata_requires_rollback_even_without_text · function · L214-L220 — fn model_metadata_requires_rollback_even_without_text()
- status_events_are_not_replay_visible · function · L223-L243 — fn status_events_are_not_replay_visible()
- tracker_forwards_in_order_and_reports_output · function · L246-L278 — async fn tracker_forwards_in_order_and_reports_output()
- tracker_reports_no_output_for_status_only_attempt · function · L281-L292 — async fn tracker_reports_no_output_for_status_only_attempt()
- backoff_delay_is_jittered_exponential · function · L295-L309 — fn backoff_delay_is_jittered_exponential()
