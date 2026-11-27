# crates/jcode-tui/src/tui/app/ui_prefs.rs

- UI_PREFS_FILE · constant · L9-L9 — const UI_PREFS_FILE: &str = "ui_preferences.json";
- UiPreferences · struct · L12-L19 — pub(crate) struct UiPreferences
- prefs_path · function · L21-L25 — fn prefs_path() -> Option<std::path::PathBuf>
- load · function · L27-L32 — pub(crate) fn load() -> UiPreferences
- inline_images_visible · function · L35-L37 — pub(crate) fn inline_images_visible() -> bool
- save_inline_images_visible · function · L41-L57 — pub(crate) fn save_inline_images_visible(visible: bool)
- tests · module · L60-L103 — mod tests
- inline_images_visibility_round_trips_through_disk · function · L64-L84 — fn inline_images_visibility_round_trips_through_disk()
- save_preserves_unknown_future_fields_via_load_modify_write · function · L87-L102 — fn save_preserves_unknown_future_fields_via_load_modify_write()
