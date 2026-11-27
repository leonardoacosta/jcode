# crates/jcode-app-core/src/ambient/runner_recovery_tests.rs

- request_for · function · L7-L23 — fn request_for(session_id: &str) -> ScheduleRequest
- history_state · function · L25-L35 — fn history_state(id: &str) -> (ItemState, Vec<crate::ambient::DeliveryAttempt>)
- revoked_target_is_suppressed_never_woken_and_recorded_cancelled · function · L38-L82 — async fn revoked_target_is_suppressed_never_woken_and_recorded_cancelled()
- acknowledged_delivery_is_recorded_as_delivered_and_not_redelivered · function · L85-L117 — async fn acknowledged_delivery_is_recorded_as_delivered_and_not_redelivered()
- failed_delivery_is_uncertain_and_not_retried_automatically · function · L120-L144 — async fn failed_delivery_is_uncertain_and_not_retried_automatically()
- stop_and_resume · function · L148-L159 — fn stop_and_resume(session_id: &str) -> (u64, u64)
- deliver_with_origin · function · L161-L169 — async fn deliver_with_origin(session_id: &str, origin: u64, provider: Arc<dyn Provider>) -> String
- item_from_a_revoked_generation_is_suppressed_even_after_resume · function · L172-L189 — async fn item_from_a_revoked_generation_is_suppressed_even_after_resume()
- item_from_a_newer_generation_than_the_revocation_still_delivers · function · L192-L210 — async fn item_from_a_newer_generation_than_the_revocation_still_delivers()
