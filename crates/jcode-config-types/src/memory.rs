use serde::{Deserialize, Serialize};

/// `[memory]` table: policy for stored memories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub retention: MemoryRetentionConfig,
}

/// `[memory.retention]`: opt-in automatic removal of inactive memories.
///
/// Off by default. Age or supersession alone never deletes anything unless
/// `enabled` and `auto_purge` are both true. Only entries that are inactive
/// (superseded) and at least `inactive_max_age_days` old are removed, and
/// never active ones. Read from the user's global config only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryRetentionConfig {
    /// Master switch. Nothing is ever deleted automatically while false.
    pub enabled: bool,
    /// Run the purge at startup (also requires `enabled`).
    pub auto_purge: bool,
    /// Minimum age, in days since last update, of an inactive memory before it
    /// may be removed. Zero is refused: it would remove memories the moment
    /// they are superseded.
    pub inactive_max_age_days: u64,
}

impl Default for MemoryRetentionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_purge: false,
            inactive_max_age_days: 30,
        }
    }
}

impl MemoryRetentionConfig {
    /// Whether the startup hook may delete anything.
    pub fn auto_purge_active(&self) -> bool {
        self.enabled && self.auto_purge && self.inactive_max_age_days > 0
    }
}
