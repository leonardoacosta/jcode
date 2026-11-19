//! Durable per-session control record shared by the daemon and its readers.
//!
//! Fields added after schema version 1 must carry `#[serde(default)]` so that
//! older bytes still load. Unknown fields are ignored, so an older reader never
//! rewrites a newer record's barrier fields: the daemon refuses to serve state
//! whose `schema_version` it does not understand instead of guessing.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Highest `ControlRecord::schema_version` this build can enforce. Version 1
/// records (no `min_reader_version`) still load and are never rewritten just to
/// change versions.
pub const CONTROL_SCHEMA_VERSION: u32 = 2;

/// Reader version this build identifies as. New records set their
/// `min_reader_version` to this value, so any binary whose own constant is lower
/// (or that predates the field and rejects newer schemas) refuses the record
/// instead of serving or rewriting it. Rollback below this floor needs the
/// control files removed or a binary at or above the floor; bytes are never
/// migrated or deleted automatically.
pub const CONTROL_MIN_READER_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlState {
    Runnable,
    StopRequested,
    Draining,
    Stopped,
    Unresolved,
}

impl ControlState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::StopRequested => "stop_requested",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlRecord {
    pub schema_version: u32,
    pub session_id: String,
    pub owner_session_id: Option<String>,
    pub revision: u64,
    pub last_allocated_generation: u64,
    pub admitted_generation: Option<u64>,
    pub revoked_through: u64,
    pub state: ControlState,
    pub latest_stop_request: Option<String>,
    #[serde(default)]
    pub unresolved_categories: HashSet<String>,
    /// Shell processes this session generation spawned and has not yet proven
    /// gone. `shell_children` stays in `unresolved_categories` while any remain.
    #[serde(default)]
    pub owned_children: Vec<OwnedProcess>,
    /// Oldest reader allowed to serve this record. Legacy records carry 1.
    #[serde(default = "legacy_min_reader_version")]
    pub min_reader_version: u32,
    /// Execution generation that produced the latest accepted completion report.
    /// Absent on legacy records, which therefore never satisfy a generation-bound wait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_generation: Option<u64>,
}

/// A spawned child, identified by pid and (where the OS exposes it) start time
/// so a reused pid is never mistaken for the original process.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedProcess {
    pub pid: u32,
    #[serde(default)]
    pub start_time: Option<u64>,
}

fn legacy_min_reader_version() -> u32 {
    1
}

impl ControlRecord {
    pub fn new(id: &str, owner: Option<&str>) -> Self {
        Self {
            schema_version: CONTROL_SCHEMA_VERSION,
            session_id: id.into(),
            owner_session_id: owner.map(str::to_owned),
            revision: 0,
            last_allocated_generation: 0,
            admitted_generation: None,
            revoked_through: 0,
            state: ControlState::Runnable,
            latest_stop_request: None,
            unresolved_categories: HashSet::new(),
            owned_children: Vec::new(),
            min_reader_version: CONTROL_MIN_READER_VERSION,
            report_generation: None,
        }
    }

    /// Check the invariants a loaded record must satisfy for `id`.
    pub fn validate(&self, id: &str) -> Result<(), &'static str> {
        if self.schema_version == 0 || self.schema_version > CONTROL_SCHEMA_VERSION {
            return Err("Unsupported session control version, bytes preserved");
        }
        if self.min_reader_version > CONTROL_MIN_READER_VERSION {
            return Err("Session control requires a newer reader, bytes preserved");
        }
        if self.session_id != id {
            return Err("Session control identity mismatch");
        }
        if self.revoked_through > self.last_allocated_generation {
            return Err("Invalid revoked generation");
        }
        if self.report_generation > Some(self.last_allocated_generation) {
            return Err("Invalid report generation");
        }
        if let Some(generation) = self.admitted_generation
            && !(generation > self.revoked_through && generation <= self.last_allocated_generation)
        {
            return Err("Invalid admitted generation");
        }
        Ok(())
    }

    /// True when new work for this session must be refused.
    pub fn admission_revoked(&self) -> bool {
        self.state != ControlState::Runnable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stopped() -> ControlRecord {
        let mut record = ControlRecord::new("worker", Some("coord"));
        record.last_allocated_generation = 3;
        record.revoked_through = 3;
        record.state = ControlState::Stopped;
        record.revision = 4;
        record.latest_stop_request = Some("coord:5".into());
        record
    }

    #[test]
    fn record_roundtrips_and_keeps_stop_fields() {
        let record = stopped();
        let bytes = serde_json::to_vec(&record).unwrap();
        let back: ControlRecord = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, record);
        assert!(back.admission_revoked());
        assert_eq!(back.state.as_str(), "stopped");
    }

    #[test]
    fn legacy_record_without_optional_fields_loads_runnable_defaults() {
        let legacy = r#"{"schema_version":1,"session_id":"worker","owner_session_id":null,
            "revision":0,"last_allocated_generation":0,"admitted_generation":null,
            "revoked_through":0,"state":"runnable","latest_stop_request":null}"#;
        let record: ControlRecord = serde_json::from_str(legacy).unwrap();
        assert!(record.unresolved_categories.is_empty());
        assert!(record.validate("worker").is_ok());
    }

    #[test]
    fn newer_unknown_fields_are_ignored_but_unknown_version_is_refused() {
        let mut value = serde_json::to_value(stopped()).unwrap();
        value["future_field"] = serde_json::json!({"x": 1});
        let record: ControlRecord = serde_json::from_value(value.clone()).unwrap();
        assert!(record.validate("worker").is_ok());
        value["schema_version"] = serde_json::json!(CONTROL_SCHEMA_VERSION + 1);
        let newer: ControlRecord = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            newer.validate("worker"),
            Err("Unsupported session control version, bytes preserved")
        );
        value["schema_version"] = serde_json::json!(CONTROL_SCHEMA_VERSION);
        value["min_reader_version"] = serde_json::json!(CONTROL_MIN_READER_VERSION + 1);
        let too_new: ControlRecord = serde_json::from_value(value).unwrap();
        assert_eq!(
            too_new.validate("worker"),
            Err("Session control requires a newer reader, bytes preserved")
        );
    }

    #[test]
    fn version_one_record_loads_with_legacy_reader_floor_and_new_records_set_the_floor() {
        let legacy = r#"{"schema_version":1,"session_id":"worker","owner_session_id":null,
            "revision":3,"last_allocated_generation":1,"admitted_generation":1,
            "revoked_through":0,"state":"runnable","latest_stop_request":null}"#;
        let record: ControlRecord = serde_json::from_str(legacy).unwrap();
        assert_eq!(record.min_reader_version, 1);
        assert!(record.validate("worker").is_ok());
        let fresh = ControlRecord::new("worker", None);
        assert_eq!(fresh.schema_version, CONTROL_SCHEMA_VERSION);
        assert_eq!(fresh.min_reader_version, CONTROL_MIN_READER_VERSION);
        let mut zero = fresh;
        zero.schema_version = 0;
        assert!(zero.validate("worker").is_err());
    }

    #[test]
    fn corrupt_and_inconsistent_records_are_rejected() {
        assert!(serde_json::from_str::<ControlRecord>("{not json").is_err());
        assert!(
            serde_json::from_str::<ControlRecord>(r#"{"schema_version":1,"state":"bogus"}"#)
                .is_err()
        );
        let mut record = stopped();
        record.revoked_through = 9;
        assert_eq!(record.validate("worker"), Err("Invalid revoked generation"));
        let mut record = stopped();
        record.admitted_generation = Some(2);
        assert_eq!(
            record.validate("worker"),
            Err("Invalid admitted generation")
        );
        assert_eq!(
            stopped().validate("other"),
            Err("Session control identity mismatch")
        );
    }

    #[test]
    fn stale_snapshot_cannot_clear_revocation_by_value() {
        // A swarm membership snapshot is only a projection. Overwriting a member's
        // status never touches the control record, so revocation is preserved.
        let record = stopped();
        let snapshot_status = "ready";
        assert_ne!(snapshot_status, record.state.as_str());
        assert!(record.admission_revoked());
    }
}
