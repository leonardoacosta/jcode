# crates/jcode-harness-api-server/src/translate/credentials.rs

- credential_binding · function · L6-L28 — pub(super) fn credential_binding(
- write_credential · function · L30-L65 — pub(super) fn write_credential(
- atomic_owner_write · function · L67-L121 — pub(super) fn atomic_owner_write(path: &std::path::Path, bytes: &[u8]) -> Result<(), String>
- ensure_owner_only_dir · function · L123-L145 — pub(super) fn ensure_owner_only_dir(path: &std::path::Path) -> Result<(), String>
- validate_owner_file_if_exists · function · L147-L170 — pub(super) fn validate_owner_file_if_exists(path: &std::path::Path) -> Result<(), String>
- atomic_replace · function · L173-L178 — pub(super) fn atomic_replace(
- atomic_replace · function · L181-L188 — pub(super) fn atomic_replace(
- MoveFileExW · function · L186-L187 — unsafe extern "system"
- MOVEFILE_REPLACE_EXISTING · constant · L189-L189 — const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
- MOVEFILE_WRITE_THROUGH · constant · L190-L190 — const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
