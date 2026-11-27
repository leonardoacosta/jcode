# crates/jcode-setup-hints/src/keymap/terminal.rs

- parse_ghostty_keybind_line · function · L22-L50 — pub fn parse_ghostty_keybind_line(line: &str) -> Option<DiscoveredBinding>
- parse_trigger · function · L55-L91 — fn parse_trigger(trigger: &str) -> Option<KeyChord>
- split_trigger_tokens · function · L96-L123 — fn split_trigger_tokens(trigger: &str) -> Vec<String>
- parse_ghostty_keybinds · function · L127-L132 — pub fn parse_ghostty_keybinds(output: &str) -> Vec<DiscoveredBinding>
- read_ghostty_keybinds · function · L137-L154 — pub fn read_ghostty_keybinds() -> Vec<DiscoveredBinding>
- CANDIDATES · constant · L140-L143 — const CANDIDATES: [&str; 2] = [
- read_ghostty_keybinds · function · L157-L167 — pub fn read_ghostty_keybinds() -> Vec<DiscoveredBinding>
- tests · module · L170-L233 — mod tests
- parses_listed_keybind · function · L174-L178 — fn parses_listed_keybind()
- parses_bare_config_line · function · L181-L185 — fn parses_bare_config_line()
- parses_named_punctuation_key · function · L188-L191 — fn parses_named_punctuation_key()
- parses_digit_key · function · L194-L197 — fn parses_digit_key()
- parses_literal_plus_key · function · L200-L203 — fn parses_literal_plus_key()
- skips_comments_and_blanks · function · L206-L209 — fn skips_comments_and_blanks()
- skips_multi_key_sequences · function · L212-L214 — fn skips_multi_key_sequences()
- skips_logical_copy_paste_triggers · function · L217-L220 — fn skips_logical_copy_paste_triggers()
- parses_full_output · function · L223-L232 — fn parses_full_output()
