# crates/jcode-tui-account-picker/src/overlay_render.rs

- hotkey · function · L3-L5 — pub(super) fn hotkey(text: &'static str) -> Span<'static>
- provider_header_line · function · L7-L31 — pub(super) fn provider_header_line(
- ActionSection · enum · L34-L42 — pub(super) enum ActionSection
- action_section · function · L44-L68 — pub(super) fn action_section(item: &AccountPickerItem) -> ActionSection
- account_is_active · function · L70-L74 — pub(super) fn account_is_active(item: &AccountPickerItem) -> bool
- extract_account_label · function · L76-L86 — fn extract_account_label(title: &str) -> Option<String>
- compact_item_title · function · L88-L106 — pub(super) fn compact_item_title(item: &AccountPickerItem) -> String
- action_icon · function · L108-L125 — pub(super) fn action_icon(item: &AccountPickerItem) -> (&'static str, Color)
- account_count_summary · function · L127-L133 — pub(super) fn account_count_summary(count: usize) -> String
- action_kind_label · function · L135-L137 — pub(super) fn action_kind_label(command: &AccountPickerCommand) -> &'static str
- action_kind_badge · function · L139-L148 — pub(super) fn action_kind_badge(command: &AccountPickerCommand) -> (&'static str, Color)
- action_kind_help · function · L150-L192 — pub(super) fn action_kind_help(command: &AccountPickerCommand) -> &'static str
- command_preview · function · L194-L230 — pub(super) fn command_preview(command: &AccountPickerCommand) -> String
- metric_span · function · L232-L237 — pub(super) fn metric_span(label: &'static str, value: usize, color: Color) -> Span<'static>
- provider_style · function · L239-L260 — pub(super) fn provider_style(provider_id: &str) -> Style
- truncate_with_ellipsis · function · L262-L276 — pub(super) fn truncate_with_ellipsis(input: &str, width: usize) -> String
- centered_rect · function · L278-L295 — pub(super) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect
