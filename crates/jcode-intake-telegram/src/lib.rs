//! Telegram transport adapter for factory intake.
//!
//! Implements `channel-adapter-telegram`. This crate is the ONLY place
//! permitted to know Telegram's vocabulary; it maps updates into the
//! provider-neutral envelope and never leaks transport field names across
//! the intake boundary.

pub mod adapter;
pub mod allowlist;
pub mod client;
pub mod credential;
pub mod mapping;

pub use adapter::{Handled, Outbound, TelegramAdapter};
pub use allowlist::{Allowlist, unauthorized_hint};
pub use client::{ApiError, TelegramClient};
pub use credential::{BotToken, CredentialError, load_bot_token};
pub use mapping::{ParseOutcome, ParsedMessage, parse, to_envelope};
