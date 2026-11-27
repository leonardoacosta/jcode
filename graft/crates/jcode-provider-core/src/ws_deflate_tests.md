# crates/jcode-provider-core/src/ws_deflate_tests.rs

- jsonish · function · L8-L11 — fn jsonish(len: usize) -> String
- parses_extension_responses · function · L14-L26 — fn parses_extension_responses()
- deflate_roundtrips_through_rfc7692_inflate · function · L29-L40 — fn deflate_roundtrips_through_rfc7692_inflate()
- inflate_rejects_bombs · function · L43-L52 — fn inflate_rejects_bombs()
- reference_server · function · L58-L150 — async fn reference_server(
- run_client · function · L152-L177 — async fn run_client(
- compresses_large_messages_and_inflates_context_takeover_replies · function · L180-L201 — async fn compresses_large_messages_and_inflates_context_takeover_replies()
- declined_negotiation_is_a_transparent_passthrough · function · L204-L215 — async fn declined_negotiation_is_a_transparent_passthrough()
- off_mode_never_compresses_but_still_inflates · function · L218-L228 — async fn off_mode_never_compresses_but_still_inflates()
