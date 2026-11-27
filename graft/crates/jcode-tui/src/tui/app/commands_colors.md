# crates/jcode-tui/src/tui/app/commands_colors.rs

- USAGE · constant · L13-L19 — const USAGE: &str = "Usage:\n  \
- handle_colors_command · function · L21-L49 — pub(super) fn handle_colors_command(app: &mut App, trimmed: &str) -> bool
- configured_palette · function · L51-L59 — fn configured_palette() -> Palette
- list_colors · function · L61-L80 — fn list_colors(app: &mut App)
- active_background · function · L86-L92 — fn active_background() -> (u8, u8, u8)
- show_harmony · function · L94-L126 — fn show_harmony(app: &mut App)
- export_colors · function · L128-L139 — fn export_colors(app: &mut App)
- generate_palette · function · L147-L185 — fn generate_palette(app: &mut App, seed: Option<&str>)
- set_color · function · L187-L217 — fn set_color(app: &mut App, role_key: &str, value: &str)
- reset_colors · function · L219-L243 — fn reset_colors(app: &mut App, role_key: Option<&str>)
- persist · function · L249-L257 — fn persist(
- harmony_delta_line · function · L259-L267 — fn harmony_delta_line() -> String
- tests · module · L270-L304 — mod tests
- only_claims_the_colors_command · function · L274-L279 — fn only_claims_the_colors_command()
- config_template_lists_the_full_default_palette · function · L284-L293 — fn config_template_lists_the_full_default_palette()
- usage_text_documents_every_subcommand · function · L296-L303 — fn usage_text_documents_every_subcommand()
