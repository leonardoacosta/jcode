# crates/jcode-provider-openai-runtime/src/openai_tests/opaque_recovery.rs

- OPAQUE_REJECTION · constant · L2-L2 — const OPAQUE_REJECTION: &str = r#"{"type":"error","status":400,"sequence_number":0,"response":null,"error":{"type":"invalid_request_error","message":"Invalid request"}}"#;
- opaque_recovery_public_case · function · L4-L234 — async fn opaque_recovery_public_case(
- opaque_recovery_fresh_success · function · L237-L239 — async fn opaque_recovery_fresh_success()
- opaque_recovery_persistent_success · function · L241-L243 — async fn opaque_recovery_persistent_success()
- opaque_recovery_repeated_rejection_bounded · function · L245-L247 — async fn opaque_recovery_repeated_rejection_bounded()
- opaque_recovery_specific_schema_untouched · function · L249-L251 — async fn opaque_recovery_specific_schema_untouched()
- opaque_recovery_after_created_forbidden · function · L253-L262 — async fn opaque_recovery_after_created_forbidden()
- opaque_recovery_after_text_forbidden · function · L264-L273 — async fn opaque_recovery_after_text_forbidden()
- opaque_recovery_after_thinking_forbidden · function · L275-L284 — async fn opaque_recovery_after_thinking_forbidden()
- opaque_recovery_after_tool_forbidden · function · L286-L288 — async fn opaque_recovery_after_tool_forbidden()
- opaque_recovery_cancel_invalidates_sockets · function · L290-L292 — async fn opaque_recovery_cancel_invalidates_sockets()
