# crates/jcode-app-core/src/tool/file_lock.rs

- LockMap · type · L13-L13 — type LockMap = Mutex<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>;
- locks · function · L15-L18 — fn locks() -> &'static LockMap
- LOCKS · constant · L16-L16 — static LOCKS: OnceLock<LockMap> = OnceLock::new();
- key · function · L20-L22 — fn key(path: &Path) -> PathBuf
- lock · function · L25-L33 — pub(crate) async fn lock(path: &Path) -> OwnedMutexGuard<()>
- lock_all · function · L37-L46 — pub(crate) async fn lock_all(paths: impl IntoIterator<Item = PathBuf>) -> Vec<OwnedMutexGuard<()>>
- tests · module · L49-L72 — mod tests
- concurrent_edits_to_one_file_do_not_lose_updates · function · L53-L71 — async fn concurrent_edits_to_one_file_do_not_lose_updates()
