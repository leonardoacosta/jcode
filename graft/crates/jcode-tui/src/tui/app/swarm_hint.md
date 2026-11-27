# crates/jcode-tui/src/tui/app/swarm_hint.rs

- MAX_SHOWS · constant · L19-L19 — const MAX_SHOWS: u32 = 3;
- STATE_FILE · constant · L21-L21 — const STATE_FILE: &str = "swarm_config_hint.json";
- SwarmHintState · struct · L24-L27 — struct SwarmHintState
- state_path · function · L29-L33 — fn state_path() -> Option<std::path::PathBuf>
- load_state · function · L35-L40 — fn load_state() -> SwarmHintState
- save_state · function · L42-L56 — fn save_state(state: &SwarmHintState)
- should_show · function · L60-L62 — pub(super) fn should_show(shows: u32, shown_this_session: bool) -> bool
- hint_message · function · L65-L67 — pub(super) fn hint_message() -> String
- maybe_surface_swarm_config_hint · function · L73-L82 — pub(in crate::tui::app) fn maybe_surface_swarm_config_hint(&mut self)
- tests · module · L86-L128 — mod tests
- shows_until_lifetime_cap · function · L90-L95 — fn shows_until_lifetime_cap()
- shows_at_most_once_per_session · function · L98-L100 — fn shows_at_most_once_per_session()
- hint_mentions_the_prompt_file_and_that_it_is_the_config · function · L103-L107 — fn hint_mentions_the_prompt_file_and_that_it_is_the_config()
- state_persists_show_count · function · L110-L127 — fn state_persists_show_count()
