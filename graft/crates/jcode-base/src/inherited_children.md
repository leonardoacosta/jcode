# crates/jcode-base/src/inherited_children.rs

- MCP_CHILD_MARKER_ENV · constant · L18-L18 — pub const MCP_CHILD_MARKER_ENV: &str = "JCODE_MCP_CHILD";
- imp · module · L21-L161 — mod imp
- REAP_INTERVAL · constant · L25-L25 — const REAP_INTERVAL: Duration = Duration::from_secs(30);
- children_of · function · L27-L50 — fn children_of(pid: u32) -> HashSet<i32>
- has_mcp_marker · function · L52-L60 — fn has_mcp_marker(pid: i32) -> bool
- reap · function · L63-L70 — fn reap(pids: &mut HashSet<i32>)
- start · function · L72-L110 — pub fn start()
- tests · module · L113-L160 — mod tests
- reaps_exited_inherited_child · function · L115-L124 — fn reaps_exited_inherited_child()
- detects_mcp_marker · function · L127-L159 — fn detects_mcp_marker()
- start · function · L165-L174 — pub fn start()
- ONCE · constant · L169-L169 — static ONCE: std::sync::Once = std::sync::Once::new();
