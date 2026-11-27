# crates/jcode-tui/src/tui/ui_tests/palette_topology.rs

- RoleAreas · type · L17-L17 — type RoleAreas = BTreeMap<&'static str, u32>;
- RoleAdjacency · type · L18-L18 — type RoleAdjacency = BTreeMap<(&'static str, &'static str), u32>;
- measure · function · L21-L109 — fn measure() -> (RoleAreas, RoleAdjacency)
- print_measured_palette_topology · function · L114-L131 — fn print_measured_palette_topology()
- camel · function · L133-L143 — fn camel(key: &str) -> String
- measured_topology_covers_real_area_and_adjacency · function · L148-L165 — fn measured_topology_covers_real_area_and_adjacency()
