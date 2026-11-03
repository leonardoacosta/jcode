use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TasksConfig {
    pub unattended: bool,
    pub max_concurrent: u32,
    pub max_attempts: u32,
    pub max_runtime_seconds: u64,
    pub max_spend_microusd: u64,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            unattended: false,
            max_concurrent: 1,
            max_attempts: 3,
            max_runtime_seconds: 3600,
            max_spend_microusd: 1_000_000,
        }
    }
}

impl TasksConfig {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [
            ("max_concurrent", u64::from(self.max_concurrent)),
            ("max_attempts", u64::from(self.max_attempts)),
            ("max_runtime_seconds", self.max_runtime_seconds),
            ("max_spend_microusd", self.max_spend_microusd),
        ] {
            if value == 0 {
                return Err(format!("tasks.{name} must be positive and finite"));
            }
        }
        Ok(())
    }

    /// Evaluate a trusted persisted snapshot immediately before dispatch/restart.
    /// Does not reserve an attempt or grant authorization.
    pub fn admit(&self, task: &TaskAdmissionSnapshot) -> AdmissionDecision {
        let mut decision = AdmissionDecision {
            state: AdmissionState::Ready,
            reasons: vec![],
            remaining_attempts: 0,
            remaining_runtime_seconds: 0,
            remaining_spend_microusd: 0,
        };
        let denial = if let Err(reason) = self.validate() {
            Some((AdmissionState::Blocked, reason))
        } else if task.cancelled {
            Some((AdmissionState::Blocked, "Task is cancelled".into()))
        } else if !self.unattended {
            Some((
                AdmissionState::Blocked,
                "Unattended admission is disabled by user policy".into(),
            ))
        } else if !task.ownership_verified {
            Some((
                AdmissionState::NeedsHuman,
                "Task attempt ownership is unverified".into(),
            ))
        } else if let Some(owner) = &task.live_attempt {
            Some((
                AdmissionState::Waiting,
                format!("Existing live attempt {owner}; do not dispatch a duplicate"),
            ))
        } else if let Some(auth) = &task.authorization {
            if auth.scope.trim().is_empty() || auth.scope != task.scope {
                Some((
                    AdmissionState::NeedsHuman,
                    "Persisted authorization does not match task scope".into(),
                ))
            } else if auth.max_attempts == 0
                || auth.max_runtime_seconds == 0
                || auth.max_spend_microusd == 0
            {
                Some((
                    AdmissionState::Blocked,
                    "Persisted task limits must be positive and finite".into(),
                ))
            } else if task.setup_required {
                Some((
                    AdmissionState::NeedsHuman,
                    "Host setup is not authorized by task policy".into(),
                ))
            } else if task.deadline_expired {
                Some((AdmissionState::Blocked, "Task deadline has expired".into()))
            } else {
                decision.remaining_attempts = self
                    .max_attempts
                    .min(auth.max_attempts)
                    .saturating_sub(task.attempts);
                decision.remaining_runtime_seconds = self
                    .max_runtime_seconds
                    .min(auth.max_runtime_seconds)
                    .saturating_sub(task.runtime_seconds);
                decision.remaining_spend_microusd = self
                    .max_spend_microusd
                    .min(auth.max_spend_microusd)
                    .saturating_sub(task.spent_microusd);
                if decision.remaining_attempts == 0 {
                    Some((
                        AdmissionState::Blocked,
                        "Task attempt limit exhausted".into(),
                    ))
                } else if decision.remaining_runtime_seconds == 0 {
                    Some((
                        AdmissionState::Blocked,
                        "Task runtime limit exhausted".into(),
                    ))
                } else if decision.remaining_spend_microusd == 0 {
                    Some((AdmissionState::Blocked, "Task spend limit exhausted".into()))
                } else if task.active_attempts >= self.max_concurrent {
                    Some((
                        AdmissionState::Waiting,
                        "Task concurrency capacity exhausted".into(),
                    ))
                } else if !task.dependencies_ready {
                    Some((
                        AdmissionState::Waiting,
                        "Task dependencies are not ready".into(),
                    ))
                } else if !task.workspace_ready {
                    Some((
                        AdmissionState::Waiting,
                        "Task workspace is not ready".into(),
                    ))
                } else {
                    None
                }
            }
        } else {
            Some((
                AdmissionState::NeedsHuman,
                "Persisted task authorization is missing".into(),
            ))
        };
        if let Some((state, reason)) = denial {
            decision.state = state;
            decision.reasons.push(reason);
        } else {
            decision.reasons.push("Persisted scope, ownership, dependencies, workspace and finite limits permit admission".into());
        }
        decision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorktreesConfig {
    pub root: String,
    pub creator: Option<String>,
    /// Only user-scoped configuration can set future creation authority.
    pub allow_creation: bool,
    pub cleanup: WorktreeCleanup,
}

impl Default for WorktreesConfig {
    fn default() -> Self {
        Self {
            root: "~/worktrees".into(),
            creator: None,
            allow_creation: false,
            cleanup: WorktreeCleanup::Manual,
        }
    }
}

impl WorktreesConfig {
    pub fn validate(&self) -> Result<(), String> {
        let path = std::path::Path::new(&self.root);
        if !(path.is_absolute() || self.root.starts_with("~/"))
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err("worktrees.root must be absolute or ~/ relative, without traversal".into());
        }
        if let Some(creator) = &self.creator
            && (creator.is_empty()
                || creator == "."
                || creator == ".."
                || !creator
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')))
        {
            return Err("worktrees.creator must be a verified single identity component".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeCleanup {
    #[default]
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskAuthorization {
    pub scope: String,
    pub max_attempts: u32,
    pub max_runtime_seconds: u64,
    pub max_spend_microusd: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TaskAdmissionSnapshot {
    pub authorization: Option<TaskAuthorization>,
    pub scope: String,
    pub cancelled: bool,
    pub dependencies_ready: bool,
    pub workspace_ready: bool,
    pub ownership_verified: bool,
    pub live_attempt: Option<String>,
    pub active_attempts: u32,
    pub attempts: u32,
    pub runtime_seconds: u64,
    pub spent_microusd: u64,
    pub setup_required: bool,
    pub deadline_expired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdmissionState {
    Ready,
    Waiting,
    Blocked,
    NeedsHuman,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionDecision {
    pub state: AdmissionState,
    pub reasons: Vec<String>,
    pub remaining_attempts: u32,
    pub remaining_runtime_seconds: u64,
    pub remaining_spend_microusd: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_resource_deadline_setup_and_ownership_edges() {
        let (policy, ready) = ready();
        for field in [
            "attempts",
            "runtime",
            "spend",
            "concurrency",
            "workspace",
            "ownership",
            "setup",
            "deadline",
        ] {
            let mut task = ready.clone();
            let expected = match field {
                "attempts" => {
                    task.attempts = 2;
                    AdmissionState::Blocked
                }
                "runtime" => {
                    task.runtime_seconds = u64::MAX;
                    AdmissionState::Blocked
                }
                "spend" => {
                    task.spent_microusd = u64::MAX;
                    AdmissionState::Blocked
                }
                "concurrency" => {
                    task.active_attempts = 1;
                    AdmissionState::Waiting
                }
                "workspace" => {
                    task.workspace_ready = false;
                    AdmissionState::Waiting
                }
                "ownership" => {
                    task.ownership_verified = false;
                    AdmissionState::NeedsHuman
                }
                "setup" => {
                    task.setup_required = true;
                    AdmissionState::NeedsHuman
                }
                _ => {
                    task.deadline_expired = true;
                    AdmissionState::Blocked
                }
            };
            assert_eq!(policy.admit(&task).state, expected, "{field}");
        }
        let mut task = ready;
        task.dependencies_ready = false;
        let decision = policy.admit(&task);
        assert_eq!(decision.reasons, vec!["Task dependencies are not ready"]);
    }

    #[test]
    fn task_policy_defaults_fail_closed_and_limits_validate() {
        let mut tasks = TasksConfig::default();
        assert!(!tasks.unattended);
        assert!(tasks.validate().is_ok());
        tasks.max_attempts = 0;
        assert!(tasks.validate().unwrap_err().contains("max_attempts"));
        tasks = TasksConfig::default();
        tasks.max_spend_microusd = 0;
        assert!(tasks.validate().is_err());
        let worktrees = WorktreesConfig::default();
        assert!(!worktrees.allow_creation);
        assert!(worktrees.creator.is_none());
    }

    fn ready() -> (TasksConfig, TaskAdmissionSnapshot) {
        let policy = TasksConfig {
            unattended: true,
            ..Default::default()
        };
        let snapshot = TaskAdmissionSnapshot {
            authorization: Some(TaskAuthorization {
                scope: "bounded task".into(),
                max_attempts: 2,
                max_runtime_seconds: 100,
                max_spend_microusd: 100,
            }),
            scope: "bounded task".into(),
            dependencies_ready: true,
            workspace_ready: true,
            ownership_verified: true,
            ..Default::default()
        };
        (policy, snapshot)
    }

    #[test]
    fn admission_has_no_clock_window_and_never_widens_authorization() {
        let (mut policy, snapshot) = ready();
        let decision = policy.admit(&snapshot);
        assert_eq!(decision.state, AdmissionState::Ready);
        assert_eq!(decision.remaining_attempts, 2);
        policy.max_attempts = 100;
        assert_eq!(policy.admit(&snapshot).remaining_attempts, 2);
        let mut changed = snapshot.clone();
        changed.scope = "expanded scope".into();
        assert_eq!(policy.admit(&changed).state, AdmissionState::NeedsHuman);
        changed = snapshot;
        changed.spent_microusd = 100;
        assert_eq!(policy.admit(&changed).state, AdmissionState::Blocked);
    }

    #[test]
    fn admission_explains_disabled_unknown_cancelled_dependencies_and_duplicate() {
        let (mut policy, mut snapshot) = ready();
        policy.unattended = false;
        assert_eq!(policy.admit(&snapshot).state, AdmissionState::Blocked);
        policy.unattended = true;
        snapshot.authorization = None;
        assert_eq!(policy.admit(&snapshot).state, AdmissionState::NeedsHuman);
        let (_, mut snapshot) = ready();
        snapshot.dependencies_ready = false;
        assert_eq!(policy.admit(&snapshot).state, AdmissionState::Waiting);
        snapshot.cancelled = true;
        assert_eq!(policy.admit(&snapshot).state, AdmissionState::Blocked);
        snapshot.cancelled = false;
        snapshot.live_attempt = Some("attempt-1".into());
        let decision = policy.admit(&snapshot);
        assert_eq!(decision.state, AdmissionState::Waiting);
        assert!(decision.reasons.iter().any(|s| s.contains("attempt-1")));
    }
}
