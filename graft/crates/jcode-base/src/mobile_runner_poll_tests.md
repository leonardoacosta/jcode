# crates/jcode-base/src/mobile_runner_poll_tests.rs

- settle · function · L13-L17 — async fn settle()
- tick · function · L19-L22 — async fn tick(seconds: u64)
- cancel · function · L24-L27 — async fn cancel(task: JoinHandle<()>)
- poll_commands_starts_immediately · function · L30-L47 — async fn poll_commands_starts_immediately()
- poll_commands_drains_completed_work_without_delay · function · L50-L75 — async fn poll_commands_drains_completed_work_without_delay()
- poll_commands_idle_claims_once_per_five_seconds · function · L78-L95 — async fn poll_commands_idle_claims_once_per_five_seconds()
- poll_commands_errors_retry_after_twenty_not_twenty_five_seconds · function · L98-L123 — async fn poll_commands_errors_retry_after_twenty_not_twenty_five_seconds()
- poll_commands_cancellation_stops_future_claims · function · L126-L145 — async fn poll_commands_cancellation_stops_future_claims()
- poll_commands_controlled_clock_baseline_comparison · function · L148-L197 — async fn poll_commands_controlled_clock_baseline_comparison()
