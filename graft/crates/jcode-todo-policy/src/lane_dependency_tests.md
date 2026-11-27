# crates/jcode-todo-policy/src/lane_dependency_tests.rs

- item · function · L3-L13 — fn item(id: &str, phase: u32, group: &str, after: Option<&[&str]>) -> TodoItem
- validates_by_first_appearance_not_numeric_phase · function · L16-L22 — fn validates_by_first_appearance_not_numeric_phase()
- rejects_empty_duplicate_missing_same_phase_and_inconsistent_edges · function · L25-L47 — fn rejects_empty_duplicate_missing_same_phase_and_inconsistent_edges()
- persistence_defaults_and_round_trips · function · L50-L65 — fn persistence_defaults_and_round_trips()
- worker_cannot_overwrite_coordinator_prerequisites · function · L68-L93 — fn worker_cannot_overwrite_coordinator_prerequisites()
- worker_validation_uses_full_graph_and_rejects_dangling_downstream_edges · function · L96-L133 — fn worker_validation_uses_full_graph_and_rejects_dangling_downstream_edges()
