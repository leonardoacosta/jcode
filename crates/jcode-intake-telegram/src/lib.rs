//! Telegram transport adapter for factory intake.
//!
//! Implements `channel-adapter-telegram`. This crate is the ONLY place
//! permitted to know Telegram's vocabulary; it maps updates into the
//! provider-neutral envelope and never leaks transport field names across
//! the intake boundary.

pub mod adapter;
pub mod allowlist;
pub mod mapping;

pub use adapter::{Handled, Outbound, TelegramAdapter};
pub use allowlist::{Allowlist, unauthorized_hint};
pub use mapping::{ParseOutcome, ParsedMessage, parse, to_envelope};
