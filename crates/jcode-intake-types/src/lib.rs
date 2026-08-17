//! Provider-neutral factory intake.
//!
//! Implements the `factory-intake` capability from the OpenSpec change
//! `add-factory-intake-capability`. Transport adapters map their own
//! payloads into [`Envelope`] and never leak transport vocabulary here.

pub mod dedupe;
pub mod envelope;
pub mod record;
pub mod redact;
pub mod sqlite;
pub mod store;

pub use dedupe::DedupeKey;
pub use envelope::{Attachment, Envelope};
pub use record::{Classification, IntakeEvent, Record, RecordId};
pub use redact::{Redactor, ScrubOutcome};
pub use sqlite::{SqliteIntakeStore, SqliteStoreError};
pub use store::{
    IntakeStore, Proposal, ProposalId, ProposalState, StoreError, TrackedWork, TrackedWorkId,
};
