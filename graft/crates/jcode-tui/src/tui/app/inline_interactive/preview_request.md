# crates/jcode-tui/src/tui/app/inline_interactive/preview_request.rs

- InlinePickerPreviewRequest · enum · L4-L18 — pub(super) enum InlinePickerPreviewRequest
- kind · function · L21-L27 — fn kind(&self) -> PickerKind
- filter · function · L29-L36 — pub(super) fn filter(&self) -> &str
- account_provider_filter · function · L38-L46 — fn account_provider_filter(&self) -> Option<&str>
- open · function · L48-L56 — pub(super) fn open(&self, app: &mut App)
- matches_picker · function · L58-L70 — pub(super) fn matches_picker(&self, app: &App, picker: &InlineInteractiveState) -> bool
- picker_account_provider_scope · function · L73-L101 — pub(super) fn picker_account_provider_scope(picker: &InlineInteractiveState) -> Option<&str>
