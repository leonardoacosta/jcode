# crates/jcode-tui/src/tui/app/remote_header_hint.rs

- HINT_FILE · constant · L19-L19 — const HINT_FILE: &str = "remote_header_hint.json";
- RemoteHeaderHint · struct · L22-L31 — struct RemoteHeaderHint
- SessionFactsHint · struct · L34-L40 — struct SessionFactsHint
- hint_path · function · L42-L46 — fn hint_path() -> Option<std::path::PathBuf>
- origin · function · L48-L50 — fn origin() -> String
- read_hint · function · L52-L61 — fn read_hint() -> Option<RemoteHeaderHint>
- bare_model · function · L64-L71 — fn bare_model(model: &str) -> String
- apply_remote_header_hint · function · L76-L102 — pub(super) fn apply_remote_header_hint(&mut self)
- apply_configured_route_facts · function · L111-L173 — pub(super) fn apply_configured_route_facts(&mut self)
- apply_session_facts_hint · function · L180-L204 — fn apply_session_facts_hint(&mut self, facts: SessionFactsHint)
- persist_remote_header_hint · function · L209-L246 — pub(super) fn persist_remote_header_hint(&self)
- tests · module · L250-L288 — mod tests
- bare_model_ignores_route_prefix_and_case · function · L254-L261 — fn bare_model_ignores_route_prefix_and_case()
- legacy_hint_without_session_facts_still_parses · function · L264-L269 — fn legacy_hint_without_session_facts_still_parses()
- session_facts_round_trip · function · L272-L287 — fn session_facts_round_trip()
