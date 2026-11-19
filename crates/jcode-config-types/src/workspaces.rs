//! `[workspaces]`: ordered rules mapping a git `origin` to a named workspace.
//!
//! Rules live only in the user's global config (root file and its include
//! fragments). A project checkout can never add or change one, so a cloned
//! repository cannot grant itself hooks or model overrides.

use crate::{AgentRoleModels, HooksConfig};
use serde::{Deserialize, Serialize};

/// Issue tracker a workspace uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tracker {
    Linear,
    Ado,
    None,
}

impl Tracker {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "linear" => Some(Self::Linear),
            "ado" => Some(Self::Ado),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Ado => "ADO (Azure DevOps)",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    /// Evaluated in order; the first rule whose `origin` matches wins.
    pub rules: Vec<WorkspaceRule>,
}

impl WorkspacesConfig {
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Whether any rule could contribute a lifecycle hook for `event`.
    pub fn has_lifecycle_entries(&self, event: &str) -> bool {
        self.rules
            .iter()
            .any(|rule| !rule.hooks.lifecycle_entries(event).is_empty())
    }

    /// One warning per rule problem, naming the rule.
    pub fn warnings(&self) -> Vec<String> {
        self.rules
            .iter()
            .enumerate()
            .flat_map(|(index, rule)| rule.warnings(index))
            .collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceRule {
    /// Workspace name shown in the header badge.
    pub name: String,
    /// Glob over the credential-free `host/path` of the origin URL, for
    /// example `dev.azure.com/brown/**`. Case-insensitive. `*` and `?` stay
    /// within one path segment, `**` crosses segments.
    pub origin: String,
    /// `linear`, `ado`, or `none`. Anything else skips this rule's hints.
    pub tracker: Option<String>,
    /// Branch naming convention, for example `feat/<id>-<slug>`.
    pub branch_template: Option<String>,
    /// Skills worth reaching for in this workspace.
    pub skills: Vec<String>,
    /// Expected effective `git config user.email`. Never written to Git config.
    pub expected_email: Option<String>,
    /// Lifecycle hook tables (`[[workspaces.rules.hooks.PreToolUse]]`, ...)
    /// that run in addition to global `[hooks]` for sessions in this workspace.
    /// Legacy snake_case hook keys are not supported here.
    pub hooks: HooksConfig,
    /// Swarm worker model for this workspace (or `inherit`).
    pub swarm_model: Option<String>,
    /// Role-slot models for this workspace (`[workspaces.rules.roles]`).
    pub roles: AgentRoleModels,
}

impl WorkspaceRule {
    /// The configured tracker. `Err` carries the unknown value.
    pub fn tracker_kind(&self) -> Result<Option<Tracker>, String> {
        match self.tracker.as_deref().map(str::trim) {
            None | Some("") => Ok(None),
            Some(value) => Tracker::parse(value)
                .map(Some)
                .ok_or_else(|| value.to_string()),
        }
    }

    /// Hints (tracker, branch template, skills) are skipped when the tracker
    /// value is unknown, so a typo never injects half-trusted guidance.
    pub fn hints_valid(&self) -> bool {
        self.tracker_kind().is_ok()
    }

    /// Workspace model for a spawn: the role slot, then `swarm_model`.
    /// Returns `(model, role_that_matched)`.
    pub fn spawn_model(&self, role: Option<&str>) -> Option<(String, Option<String>)> {
        if let Some(role) = role
            && let Some(model) = self.roles.get(role)
        {
            return Some((model.to_string(), Some(role.trim().to_ascii_lowercase())));
        }
        self.swarm_model
            .as_deref()
            .map(str::trim)
            .filter(|model| !model.is_empty())
            .map(|model| (model.to_string(), None))
    }

    fn warnings(&self, index: usize) -> Vec<String> {
        let label = if self.name.trim().is_empty() {
            format!("workspaces.rules[{index}]")
        } else {
            format!("workspace rule \"{}\"", self.name.trim())
        };
        let mut out = Vec::new();
        if self.name.trim().is_empty() {
            out.push(format!("{label}: `name` is empty; the rule never matches"));
        }
        if self.origin.trim().is_empty() {
            out.push(format!(
                "{label}: `origin` is empty; the rule never matches"
            ));
        }
        if let Err(value) = self.tracker_kind() {
            out.push(format!(
                "{label}: unknown tracker {value:?} (expected linear, ado, or none); its hints are skipped"
            ));
        }
        let legacy = self.hooks.pre_tool_transform.is_some()
            || self.hooks.turn_start.is_some()
            || self.hooks.turn_end.is_some()
            || self.hooks.session_start.is_some()
            || self.hooks.session_end.is_some()
            || self.hooks.pre_tool.is_some()
            || self.hooks.post_tool.is_some()
            || self.hooks.post_tool_feedback.is_some();
        if legacy {
            out.push(format!(
                "{label}: only lifecycle hook tables (PreToolUse, ...) are supported in workspace hooks; legacy keys are ignored"
            ));
        }
        out
    }
}

impl HooksConfig {
    /// Global lifecycle hooks followed by `workspace` entries. A workspace
    /// entry whose `command` matches a global one replaces it. Legacy keys
    /// come from `self` unchanged.
    pub fn with_workspace_hooks(&self, workspace: &HooksConfig) -> HooksConfig {
        fn merge(
            global: &[crate::HookEntry],
            workspace: &[crate::HookEntry],
        ) -> Vec<crate::HookEntry> {
            let mut merged: Vec<_> = global
                .iter()
                .filter(|entry| {
                    !workspace
                        .iter()
                        .any(|other| other.command.trim() == entry.command.trim())
                })
                .cloned()
                .collect();
            merged.extend(workspace.iter().cloned());
            merged
        }
        let mut hooks = self.clone();
        hooks.lifecycle_session_start = merge(
            &self.lifecycle_session_start,
            &workspace.lifecycle_session_start,
        );
        hooks.lifecycle_pre_tool_use = merge(
            &self.lifecycle_pre_tool_use,
            &workspace.lifecycle_pre_tool_use,
        );
        hooks.lifecycle_post_tool_use = merge(
            &self.lifecycle_post_tool_use,
            &workspace.lifecycle_post_tool_use,
        );
        hooks.lifecycle_subagent_stop = merge(
            &self.lifecycle_subagent_stop,
            &workspace.lifecycle_subagent_stop,
        );
        hooks.lifecycle_stop = merge(&self.lifecycle_stop, &workspace.lifecycle_stop);
        hooks.lifecycle_pre_compact = merge(
            &self.lifecycle_pre_compact,
            &workspace.lifecycle_pre_compact,
        );
        hooks
    }
}

#[cfg(test)]
#[path = "workspaces_tests.rs"]
mod tests;
