# crates/jcode-tui/src/tui/ui_changelog.rs

- ChangelogEntry · struct · L5-L10 — pub(super) struct ChangelogEntry<'a>
- ChangelogGroup · struct · L14-L18 — pub(super) struct ChangelogGroup
- parse_changelog_from · function · L30-L32 — pub(super) fn parse_changelog_from(changelog: &str) -> Vec<ChangelogEntry<'_>>
- parse_changelog_from_impl · function · L34-L65 — fn parse_changelog_from_impl(changelog: &str) -> Vec<ChangelogEntry<'_>>
- parse_changelog · function · L68-L71 — fn parse_changelog() -> Vec<ChangelogEntry<'static>>
- format_changelog_timestamp · function · L73-L76 — fn format_changelog_timestamp(timestamp: i64) -> Option<String>
- group_changelog_entries · function · L79-L85 — pub(super) fn group_changelog_entries(
- group_changelog_entries_impl · function · L87-L135 — fn group_changelog_entries_impl(
- get_grouped_changelog · function · L141-L153 — pub(super) fn get_grouped_changelog() -> Vec<ChangelogGroup>
- GROUPS · constant · L142-L142 — static GROUPS: OnceLock<Vec<ChangelogGroup>> = OnceLock::new();
- get_unseen_changelog_entries · function · L159-L199 — pub(super) fn get_unseen_changelog_entries() -> &'static Vec<String>
- ENTRIES · constant · L160-L160 — static ENTRIES: OnceLock<Vec<String>> = OnceLock::new();
