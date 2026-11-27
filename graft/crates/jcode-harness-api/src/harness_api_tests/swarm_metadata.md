# crates/jcode-harness-api/src/harness_api_tests/swarm_metadata.rs

- TempDir · struct · L4-L4 — struct TempDir(std::path::PathBuf);
- new · function · L6-L19 — fn new() -> Self
- NEXT · constant · L7-L7 — static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
- write · function · L20-L22 — fn write(&self, name: &str, value: serde_json::Value)
- drop · function · L25-L27 — fn drop(&mut self)
- session · function · L29-L31 — fn session(id: &str) -> SessionInfo
- old_session_wire_shape_defaults_and_omits_swarm_fields · function · L34-L43 — fn old_session_wire_shape_defaults_and_omits_swarm_fields()
- new_session_wire_shape_roundtrips_unknown_status · function · L46-L51 — fn new_session_wire_shape_roundtrips_unknown_status()
- enrichment_maps_only_swarm_ownership_and_preserves_title · function · L54-L82 — fn enrichment_maps_only_swarm_ownership_and_preserves_title()
- enrichment_prefers_latest_snapshot_and_existing_api_values · function · L85-L104 — fn enrichment_prefers_latest_snapshot_and_existing_api_values()
- malformed_missing_and_legacy_snapshots_degrade_per_record · function · L107-L128 — fn malformed_missing_and_legacy_snapshots_degrade_per_record()
