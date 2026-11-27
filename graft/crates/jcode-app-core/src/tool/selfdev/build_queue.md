# crates/jcode-app-core/src/tool/selfdev/build_queue.rs

- optimized_test_shell_command · function · L4-L21 — pub(super) fn optimized_test_shell_command(command: &str) -> String
- append_output_line · function · L23-L27 — async fn append_output_line(file: &mut tokio::fs::File, line: impl AsRef<str>)
- wait_for_turn · function · L29-L86 — async fn wait_for_turn(
- stream_build_command · function · L88-L170 — async fn stream_build_command(
- run_test_build · function · L172-L184 — async fn run_test_build(output_path: PathBuf, reason: &str) -> Result<TaskResult>
- run_test_request · function · L186-L251 — async fn run_test_request(
- follow_existing_build · function · L253-L335 — async fn follow_existing_build(
- run_build_request · function · L337-L448 — async fn run_build_request(
- do_build · function · L450-L696 — pub(super) async fn do_build(
- do_build_reload · function · L702-L849 — pub(super) async fn do_build_reload(
- do_test · function · L851-L989 — pub(super) async fn do_test(
- do_cancel_build · function · L991-L1053 — pub(super) async fn do_cancel_build(
