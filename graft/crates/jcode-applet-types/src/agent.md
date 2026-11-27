# crates/jcode-applet-types/src/agent.rs

- APPLET_ID · constant · L8-L8 — pub const APPLET_ID: &str = "jcode.agent";
- JCODE_APPLET_MIME · constant · L11-L11 — pub const JCODE_APPLET_MIME: &str = "application/vnd.jcode.applet+json";
- manifest · function · L14-L31 — pub fn manifest() -> Manifest
- AgentApplets · struct · L36-L39 — pub struct AgentApplets
- get · function · L42-L44 — pub fn get(&self, id: &str) -> Option<&Instance>
- get_mut · function · L45-L47 — pub fn get_mut(&mut self, id: &str) -> Option<&mut Instance>
- tests · module · L51-L80 — mod tests
- manifest_is_valid_and_allows_html · function · L56-L61 — fn manifest_is_valid_and_allows_html()
- snapshot_round_trips_and_defaults_empty · function · L64-L79 — fn snapshot_round_trips_and_defaults_empty()
