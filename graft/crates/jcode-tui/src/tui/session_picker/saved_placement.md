# crates/jcode-tui/src/tui/session_picker/saved_placement.rs

- BOOST_DAYS · constant · L7-L7 — const BOOST_DAYS: i64 = 7;
- SavedPlacement · struct · L10-L13 — pub(super) struct SavedPlacement
- path · function · L16-L18 — fn path() -> anyhow::Result<std::path::PathBuf>
- load · function · L20-L37 — pub(super) fn load() -> Self
- boosted · function · L39-L41 — pub(super) fn boosted(&self, session: &SessionInfo, now: DateTime<Utc>) -> bool
- boosted_at · function · L43-L52 — fn boosted_at(
- record_picks · function · L54-L73 — pub(super) fn record_picks(ids: impl Iterator<Item = String>)
- tests · module · L77-L95 — mod tests
- saved_boost_expires_without_removing_bookmark · function · L81-L94 — fn saved_boost_expires_without_removing_bookmark()
