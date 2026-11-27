# crates/jcode-tui-core/src/graph_topology.rs

- GraphNode · struct · L2-L17 — pub struct GraphNode
- GraphEdge · struct · L20-L27 — pub struct GraphEdge
- truncate_chars · function · L29-L34 — fn truncate_chars(s: &str, max_chars: usize) -> &str
- truncate_smart · function · L36-L56 — fn truncate_smart(s: &str, max_len: usize) -> String
- build_graph_topology · function · L63-L85 — pub fn build_graph_topology(
- collect_memory_nodes · function · L87-L113 — fn collect_memory_nodes(
- collect_tag_nodes · function · L115-L145 — fn collect_tag_nodes(
- collect_cluster_nodes · function · L147-L178 — fn collect_cluster_nodes(
- collect_edges · function · L180-L232 — fn collect_edges(
- bound_topology_size · function · L234-L280 — fn bound_topology_size(
- MAX_NODES · constant · L240-L240 — const MAX_NODES: usize = 96;
- edge_kind_name · function · L282-L291 — fn edge_kind_name(kind: &EdgeKind) -> &'static str
- graph_node_score · function · L293-L297 — pub fn graph_node_score(node: &GraphNode) -> f32
- tests · module · L300-L336 — mod tests
- build_graph_topology_deduplicates_nodes_across_project_and_global_graphs · function · L305-L320 — fn build_graph_topology_deduplicates_nodes_across_project_and_global_graphs()
- build_graph_topology_caps_large_graphs_for_stable_rendering · function · L323-L335 — fn build_graph_topology_caps_large_graphs_for_stable_rendering()
