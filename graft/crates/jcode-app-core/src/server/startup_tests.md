# crates/jcode-app-core/src/server/startup_tests.rs

- TestProvider · struct · L14-L14 — struct TestProvider;
- complete · function · L18-L28 — async fn complete(
- name · function · L30-L32 — fn name(&self) -> &str
- fork · function · L34-L36 — fn fork(&self) -> Arc<dyn Provider>
- server_run_refuses_to_replace_live_socket · function · L40-L67 — async fn server_run_refuses_to_replace_live_socket()
- is_server_ready_returns_false_immediately_for_missing_socket · function · L70-L80 — async fn is_server_ready_returns_false_immediately_for_missing_socket()
- wait_for_existing_server_tolerates_delayed_listener · function · L83-L100 — async fn wait_for_existing_server_tolerates_delayed_listener()
- server_initializes_schedule_runner_even_when_ambient_disabled · function · L103-L111 — fn server_initializes_schedule_runner_even_when_ambient_disabled()
- readiness_probes_do_not_contend_with_group_transfer_admission · function · L114-L145 — async fn readiness_probes_do_not_contend_with_group_transfer_admission()
- debug_accept_loop_responds_to_ping_without_affecting_client_count · function · L148-L177 — async fn debug_accept_loop_responds_to_ping_without_affecting_client_count()
