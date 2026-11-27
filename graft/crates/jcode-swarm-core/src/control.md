# crates/jcode-swarm-core/src/control.rs

- CONTROL_SCHEMA_VERSION · constant · L14-L14 — pub const CONTROL_SCHEMA_VERSION: u32 = 2;
- CONTROL_MIN_READER_VERSION · constant · L22-L22 — pub const CONTROL_MIN_READER_VERSION: u32 = 2;
- ControlState · enum · L26-L32 — pub enum ControlState
- as_str · function · L35-L43 — pub fn as_str(self) -> &'static str
- ControlRecord · struct · L47-L70 — pub struct ControlRecord
- OwnedProcess · struct · L75-L79 — pub struct OwnedProcess
- legacy_min_reader_version · function · L81-L83 — fn legacy_min_reader_version() -> u32
- new · function · L86-L102 — pub fn new(id: &str, owner: Option<&str>) -> Self
- validate · function · L105-L127 — pub fn validate(&self, id: &str) -> Result<(), &'static str>
- admission_revoked · function · L130-L132 — pub fn admission_revoked(&self) -> bool
- tests · module · L136-L237 — mod tests
- stopped · function · L139-L147 — fn stopped() -> ControlRecord
- record_roundtrips_and_keeps_stop_fields · function · L150-L157 — fn record_roundtrips_and_keeps_stop_fields()
- legacy_record_without_optional_fields_loads_runnable_defaults · function · L160-L167 — fn legacy_record_without_optional_fields_loads_runnable_defaults()
- newer_unknown_fields_are_ignored_but_unknown_version_is_refused · function · L170-L188 — fn newer_unknown_fields_are_ignored_but_unknown_version_is_refused()
- version_one_record_loads_with_legacy_reader_floor_and_new_records_set_the_floor · function · L191-L204 — fn version_one_record_loads_with_legacy_reader_floor_and_new_records_set_the_floor()
- corrupt_and_inconsistent_records_are_rejected · function · L207-L226 — fn corrupt_and_inconsistent_records_are_rejected()
- stale_snapshot_cannot_clear_revocation_by_value · function · L229-L236 — fn stale_snapshot_cannot_clear_revocation_by_value()
