//! Shared semantic types for explicit rendered tool artifacts.
//!
//! The artifact body remains the surrounding tool output string. This crate
//! only defines the display descriptor that accompanies that output.

use serde::{Deserialize, Serialize};

/// A recognized presentation for an explicit rendered tool artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderedArtifactKind {
    Markdown,
    Message,
    Code,
}

/// Display-only metadata associated with one tool result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedArtifact {
    pub kind: RenderedArtifactKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}
