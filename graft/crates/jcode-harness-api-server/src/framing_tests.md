# crates/jcode-harness-api-server/src/framing_tests.rs

- public_socket_rejects_malformed_and_unsupported_before_daemon_access · function · L8-L46 — async fn public_socket_rejects_malformed_and_unsupported_before_daemon_access()
- read_all · function · L48-L62 — async fn read_all(input: &'static str) -> Vec<std::io::Result<String>>
- frames_are_split_on_newlines · function · L65-L70 — async fn frames_are_split_on_newlines()
- a_final_frame_without_a_newline_is_still_returned · function · L73-L77 — async fn a_final_frame_without_a_newline_is_still_returned()
- an_unterminated_frame_is_refused_rather_than_buffered_forever · function · L83-L99 — async fn an_unterminated_frame_is_refused_rather_than_buffered_forever()
- a_large_but_terminated_frame_is_accepted · function · L104-L114 — async fn a_large_but_terminated_frame_is_accepted()
- each_frame_starts_from_a_clean_buffer · function · L120-L128 — async fn each_frame_starts_from_a_clean_buffer()
- aggregate_pdf_base64_payload_fits_transport_frame · function · L131-L142 — async fn aggregate_pdf_base64_payload_fits_transport_frame()
- cancelled_frame_reads_preserve_utf8_prefix_and_report_full_length · function · L145-L164 — async fn cancelled_frame_reads_preserve_utf8_prefix_and_report_full_length()
- cancellation_does_not_reset_the_accumulated_frame_limit · function · L167-L173 — async fn cancellation_does_not_reset_the_accumulated_frame_limit()
