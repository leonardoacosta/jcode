# crates/jcode-app-core/src/server/file_touch_service.rs

- FileTouchService · struct · L26-L31 — pub(crate) struct FileTouchService
- new · function · L35-L40 — pub(crate) fn new() -> Self
- record_touch · function · L45-L58 — pub(crate) async fn record_touch(&self, path: PathBuf, access: FileAccess)
- accesses_for_path · function · L63-L65 — pub(crate) async fn accesses_for_path(&self, path: &Path) -> Option<Vec<FileAccess>>
- sorted_file_strings_for_session · function · L69-L79 — pub(crate) async fn sorted_file_strings_for_session(&self, session_id: &str) -> Vec<String>
- snapshot · function · L85-L87 — pub(crate) async fn snapshot(&self) -> HashMap<PathBuf, Vec<FileAccess>>
- reverse_snapshot · function · L90-L92 — pub(crate) async fn reverse_snapshot(&self) -> HashMap<String, HashSet<PathBuf>>
- clear_session · function · L99-L124 — pub(crate) async fn clear_session(&self, session_id: &str)
- expire_older_than · function · L128-L146 — pub(crate) async fn expire_older_than(&self, max_age: Duration)
