# crates/jcode-base/src/mobile_runner_process.rs

- STDERR_TAIL_BYTES · constant · L6-L6 — const STDERR_TAIL_BYTES: usize = 600 * 4 + 4;
- BusyMarker · struct · L8-L8 — struct BusyMarker(Option<PathBuf>);
- touch · function · L10-L16 — fn touch(&self)
- drop · function · L19-L28 — fn drop(&mut self)
- stderr_tail · function · L31-L55 — async fn stderr_tail(mut stderr: tokio::process::ChildStderr) -> Result<String>
- run · function · L57-L111 — pub(super) async fn run(
