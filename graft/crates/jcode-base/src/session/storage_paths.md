# crates/jcode-base/src/session/storage_paths.rs

- session_path_in_dir · function · L7-L9 — pub(crate) fn session_path_in_dir(base: &std::path::Path, session_id: &str) -> PathBuf
- file_len_or_zero · function · L13-L15 — pub(super) fn file_len_or_zero(path: &Path) -> u64
- persist_vector_mode_label · function · L17-L23 — pub(super) fn persist_vector_mode_label(mode: PersistVectorMode) -> &'static str
- session_path · function · L25-L28 — pub fn session_path(session_id: &str) -> Result<PathBuf>
- session_journal_path_from_snapshot · function · L30-L37 — pub fn session_journal_path_from_snapshot(path: &Path) -> PathBuf
- session_journal_path · function · L39-L43 — pub fn session_journal_path(session_id: &str) -> Result<PathBuf>
- session_exists · function · L45-L49 — pub fn session_exists(session_id: &str) -> bool
