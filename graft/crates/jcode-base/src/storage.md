# crates/jcode-base/src/storage.rs

- read_json · function · L9-L22 — pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T>
- test_env_lock · function · L28-L31 — pub fn test_env_lock() -> &'static Mutex<()>
- ENV_LOCK · constant · L29-L29 — static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
- lock_test_env · function · L34-L38 — pub fn lock_test_env() -> MutexGuard<'static, ()>
- tests · module · L41-L41 — mod tests;
