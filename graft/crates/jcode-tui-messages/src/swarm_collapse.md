# crates/jcode-tui-messages/src/swarm_collapse.rs

- MARKER · constant · L13-L13 — const MARKER: &str = "\u{200b}\u{200b}";
- STATE_COLLAPSED · constant · L14-L14 — const STATE_COLLAPSED: &str = "swarm-tldr:collapsed";
- STATE_EXPANDED · constant · L15-L15 — const STATE_EXPANDED: &str = "swarm-tldr:expanded";
- CollapsibleSwarmContent · struct · L18-L22 — pub struct CollapsibleSwarmContent<'a>
- encode_collapsible_swarm_content · function · L25-L27 — pub fn encode_collapsible_swarm_content(tldr: &str, body: &str) -> String
- parse_collapsible_swarm_content · function · L30-L44 — pub fn parse_collapsible_swarm_content(content: &str) -> Option<CollapsibleSwarmContent<'_>>
- toggle_collapsible_swarm_content · function · L48-L59 — pub fn toggle_collapsible_swarm_content(content: &str) -> Option<String>
- SWARM_INLINE_BODY_MAX_CHARS · constant · L63-L63 — pub const SWARM_INLINE_BODY_MAX_CHARS: usize = 120;
- swarm_preview_line · function · L67-L82 — pub fn swarm_preview_line(text: &str, max_chars: usize) -> String
- collapse_long_swarm_content · function · L88-L99 — pub fn collapse_long_swarm_content(content: &str) -> String
- tests · module · L102-L162 — mod tests
- short_single_line_stays_plain · function · L106-L108 — fn short_single_line_stays_plain()
- long_or_multiline_content_collapses_to_one_row_preview · function · L111-L124 — fn long_or_multiline_content_collapses_to_one_row_preview()
- already_collapsible_content_is_untouched · function · L127-L130 — fn already_collapsible_content_is_untouched()
- roundtrip_collapsed_then_toggle · function · L133-L147 — fn roundtrip_collapsed_then_toggle()
- plain_content_is_not_collapsible · function · L150-L153 — fn plain_content_is_not_collapsible()
- empty_body_parses · function · L156-L161 — fn empty_body_parses()
