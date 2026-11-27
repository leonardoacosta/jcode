# crates/jcode-app-core/src/server/client_session_tests/resume.rs

- setup_runtime_dir · function · L5-L10 — fn setup_runtime_dir() -> Result<(tempfile::TempDir, Option<std::ffi::OsString>)>
- restore_runtime_dir · function · L12-L18 — fn restore_runtime_dir(prev_runtime: Option<std::ffi::OsString>)
- test_writer · function · L20-L24 — fn test_writer() -> Result<(Arc<Mutex<WriteHalf>>, crate::transport::Stream)>
