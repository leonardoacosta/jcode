# crates/jcode-tui/src/tui/app/slash_command_parser.rs

- SlashCommandMatch · struct · L8-L11 — pub(super) struct SlashCommandMatch
- find_known_commands · function · L13-L25 — pub(super) fn find_known_commands(input: &str, names: &[&str]) -> Vec<SlashCommandMatch>
- active_token_before_cursor · function · L28-L41 — pub(super) fn active_token_before_cursor(input: &str, cursor: usize) -> Option<(usize, usize)>
- scan_slash_tokens · function · L43-L123 — fn scan_slash_tokens(input: &str, mut on_token: impl FnMut(usize, usize))
- tests · module · L126-L159 — mod tests
- COMMANDS · constant · L129-L129 — const COMMANDS: &[&str] = &["/plan", "/review", "/help"];
- finds_known_commands_at_start_and_after_whitespace · function · L132-L146 — fn finds_known_commands_at_start_and_after_whitespace()
- ignores_urls_paths_unknown_commands_and_code · function · L149-L152 — fn ignores_urls_paths_unknown_commands_and_code()
- finds_active_partial_token_for_completion · function · L155-L158 — fn finds_active_partial_token_for_completion()
