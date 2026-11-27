# crates/jcode-applet-types/tests/applet_types.rs

- manifest · function · L5-L13 — fn manifest(capabilities: &[Capability]) -> Manifest
- document · function · L15-L17 — fn document(view: serde_json::Value) -> Document
- round_trips_a_rich_document_with_images_and_placements · function · L20-L59 — fn round_trips_a_rich_document_with_images_and_placements()
- placements_are_independent_of_tool_calls · function · L62-L78 — fn placements_are_independent_of_tool_calls()
- unknown_nodes_degrade_to_fallback_and_survive_round_trip · function · L81-L94 — fn unknown_nodes_degrade_to_fallback_and_survive_round_trip()
- malformed_known_nodes_are_errors · function · L97-L106 — fn malformed_known_nodes_are_errors()
- capabilities_gate_host_actions_and_image_sources · function · L109-L159 — fn capabilities_gate_host_actions_and_image_sources()
- enforces_limits_and_references · function · L162-L209 — fn enforces_limits_and_references()
- patches_are_revisioned_atomic_and_path_restricted · function · L212-L285 — fn patches_are_revisioned_atomic_and_path_restricted()
- manifest_schema_and_tool_card_claims · function · L288-L311 — fn manifest_schema_and_tool_card_claims()
