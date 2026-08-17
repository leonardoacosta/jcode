use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextProvenance {
    Persisted,
    CurrentClient,
    Herdr,
    InferredFromPath,
    Unavailable,
}

impl ContextProvenance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Persisted => "persisted",
            Self::CurrentClient => "current-client",
            Self::Herdr => "herdr",
            Self::InferredFromPath => "inferred",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextConfidence {
    Authoritative,
    Inferred,
    Unavailable,
}

impl ContextConfidence {
    pub fn label(self) -> &'static str {
        match self {
            Self::Authoritative => "authoritative",
            Self::Inferred => "inferred",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRow {
    pub label: String,
    pub value: Option<String>,
    pub stable_id: Option<String>,
    pub provenance: ContextProvenance,
    pub confidence: ContextConfidence,
    #[serde(default)]
    pub copyable: bool,
    #[serde(default)]
    pub focusable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

impl ContextRow {
    pub fn unavailable(label: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
            stable_id: None,
            provenance: ContextProvenance::Unavailable,
            confidence: ContextConfidence::Unavailable,
            copyable: false,
            focusable: false,
            unavailable_reason: Some(reason.into()),
        }
    }

    pub fn current(label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            label: label.into(),
            stable_id: Some(value.clone()),
            value: Some(value),
            provenance: ContextProvenance::CurrentClient,
            confidence: ContextConfidence::Inferred,
            copyable: true,
            focusable: false,
            unavailable_reason: None,
        }
    }

    pub fn persisted(
        label: impl Into<String>,
        value: impl Into<String>,
        stable_id: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            stable_id: Some(stable_id.into()),
            provenance: ContextProvenance::Persisted,
            confidence: ContextConfidence::Authoritative,
            copyable: true,
            focusable: false,
            unavailable_reason: None,
        }
    }

    pub fn display_value(&self) -> &str {
        self.value
            .as_deref()
            .or(self.unavailable_reason.as_deref())
            .unwrap_or("unavailable")
    }

    pub fn copy_value(&self) -> Option<&str> {
        self.stable_id.as_deref().or(self.value.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContextSection {
    pub title: String,
    pub rows: Vec<ContextRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContextSnapshot {
    pub semantic: ContextSection,
    pub execution: ContextSection,
}

impl ContextSnapshot {
    pub fn rows(&self) -> impl Iterator<Item = &ContextRow> {
        self.semantic.rows.iter().chain(self.execution.rows.iter())
    }
}
