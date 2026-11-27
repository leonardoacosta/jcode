# crates/jcode-app-core/src/update_preview.rs

- PreviewRelease · struct · L8-L15 — struct PreviewRelease
- preview_number · function · L17-L29 — pub(super) fn preview_number(tag: &str) -> Option<u64>
- preview_is_installable · function · L31-L47 — fn preview_is_installable(candidate: &PreviewRelease) -> bool
- select_latest_preview · function · L49-L60 — fn select_latest_preview(
- fetch_latest_preview_blocking · function · L62-L93 — pub fn fetch_latest_preview_blocking() -> Result<Option<GitHubRelease>>
- prepare_preview_update_blocking · function · L95-L102 — pub(super) fn prepare_preview_update_blocking() -> Result<PreparedUpdate>
- release_identity · function · L104-L110 — fn release_identity(version: &str) -> &str
- explicit_stable_is_update · function · L112-L114 — fn explicit_stable_is_update(release: &str, current: &str, release_build: bool) -> bool
- check_for_update_with_channel_blocking · function · L118-L144 — pub fn check_for_update_with_channel_blocking(
- prepare_stable_update_blocking · function · L146-L154 — pub(super) fn prepare_stable_update_blocking() -> Result<PreparedUpdate>
- check_for_stable_update_blocking · function · L156-L172 — fn check_for_stable_update_blocking() -> Result<Option<GitHubRelease>>
- tests · module · L176-L176 — mod tests;
