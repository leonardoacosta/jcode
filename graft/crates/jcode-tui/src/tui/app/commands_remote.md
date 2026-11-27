# crates/jcode-tui/src/tui/app/commands_remote.rs

- REMOTE_HELP · constant · L7-L21 — const REMOTE_HELP: &str = "\
- CLOUD_ACCOUNT_URL · constant · L23-L23 — const CLOUD_ACCOUNT_URL: &str = "https://jcode.sh/account";
- handle_remote_command · function · L25-L45 — pub(super) fn handle_remote_command(app: &mut App, trimmed: &str) -> bool
- activate_cloud · function · L47-L77 — fn activate_cloud(app: &mut App)
- show_status · function · L79-L87 — fn show_status(app: &mut App)
- toggle · function · L89-L130 — fn toggle(app: &mut App, enabled: bool)
- show_pairing_invite · function · L132-L164 — fn show_pairing_invite(app: &mut App)
- revoke · function · L166-L189 — fn revoke(app: &mut App, target: &str)
- tests · module · L192-L222 — mod tests
- remote_parser_does_not_claim_remote_release · function · L199-L205 — fn remote_parser_does_not_claim_remote_release()
- help_documents_every_supported_subcommand · function · L210-L221 — fn help_documents_every_supported_subcommand()
