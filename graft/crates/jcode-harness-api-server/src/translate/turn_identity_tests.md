# crates/jcode-harness-api-server/src/translate/turn_identity_tests.rs

- started · function · L3-L5 — fn started(generation: &str, id: u64) -> Value
- scoped · function · L6-L8 — fn scoped(generation: &str, id: u64, event: Value) -> Value
- bridge · function · L9-L14 — fn bridge() -> BridgeState
- request_rejection_does_not_settle_scoped_observer · function · L17-L38 — fn request_rejection_does_not_settle_scoped_observer()
- scoped_output_translates_and_old_completion_cannot_finish_new_turn · function · L40-L72 — fn scoped_output_translates_and_old_completion_cannot_finish_new_turn()
- runtime_bootstrap_rejects_retired_start_and_completion · function · L74-L91 — fn runtime_bootstrap_rejects_retired_start_and_completion()
- same_turn_bootstrap_is_idempotent_and_legacy_bridge_remains_compatible · function · L93-L113 — fn same_turn_bootstrap_is_idempotent_and_legacy_bridge_remains_compatible()
