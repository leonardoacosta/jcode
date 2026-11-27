# crates/jcode-tui-account-picker/src/lib.rs

- AccountProviderKind · enum · L3-L6 — pub enum AccountProviderKind
- AccountPickerCommand · enum · L10-L39 — pub enum AccountPickerCommand
- AccountPickerItem · struct · L43-L52 — pub struct AccountPickerItem
- action · function · L55-L70 — pub fn action(
- with_details · function · L72-L75 — pub fn with_details(mut self, details: Vec<(String, String)>) -> Self
- AccountPickerSummary · struct · L80-L88 — pub struct AccountPickerSummary
- action_kind_label · function · L90-L106 — pub fn action_kind_label(command: &AccountPickerCommand) -> &'static str
- item_matches_filter · function · L108-L124 — pub fn item_matches_filter(item: &AccountPickerItem, filter: &str) -> bool
- tests · module · L127-L146 — mod tests
- item_filter_matches_provider_title_and_action_kind · function · L131-L145 — fn item_filter_matches_provider_title_and_action_kind()
- overlay · module · L148-L148 — mod overlay;
