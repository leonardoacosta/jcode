use super::*;

impl Config {
    pub(crate) fn validate_task_policy(&self) -> anyhow::Result<()> {
        self.tasks.validate().map_err(anyhow::Error::msg)?;
        self.worktrees.validate().map_err(anyhow::Error::msg)
    }

    /// Project TOML is not a supported layer. Reject policy declarations rather than merge authority.
    pub fn policy_from_toml(
        content: &str,
        source: &str,
        project: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        if let Some(project) = project {
            let table: toml::Table = toml::from_str(project)?;
            anyhow::ensure!(
                !["tasks", "worktrees", "setup"]
                    .iter()
                    .any(|key| table.contains_key(*key)),
                "Project task/worktree/setup policy is unsupported; repository content cannot grant user authority"
            );
        }
        let config: Self = toml::from_str(content)?;
        config.validate_task_policy()?;
        let table: toml::Table = toml::from_str(content)?;
        let mut sources = BTreeMap::new();
        for (section, value) in [
            ("tasks", serde_json::to_value(&config.tasks)?),
            ("worktrees", serde_json::to_value(&config.worktrees)?),
        ] {
            for key in value.as_object().expect("policy object").keys() {
                let explicit = table
                    .get(section)
                    .and_then(toml::Value::as_table)
                    .is_some_and(|t| t.contains_key(key));
                sources.insert(
                    format!("{section}.{key}"),
                    if explicit { source } else { "default" },
                );
            }
        }
        Ok(serde_json::json!({
            "tasks": config.tasks, "worktrees": config.worktrees, "sources": sources,
            "project_policy": "unsupported: project declarations cannot grant task, filesystem, setup or publication authority",
            "environment_policy": "no task/worktree environment overrides are supported",
            "authorization": "configuration is not persisted task authorization",
        }))
    }

    /// No config migration, cache refresh, provider access or filesystem writes.
    pub fn effective_task_policy() -> anyhow::Result<serde_json::Value> {
        let path = Self::path().ok_or_else(|| anyhow::anyhow!("No config path"))?;
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };
        Self::policy_from_toml(&content, &path.display().to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_task_policy_old_config_and_field_provenance() {
        let report =
            Config::policy_from_toml("[tasks]\nmax_attempts = 2\n", "fixture.toml", None).unwrap();
        assert_eq!(report["tasks"]["unattended"], false);
        assert_eq!(report["sources"]["tasks.max_attempts"], "fixture.toml");
        assert_eq!(report["sources"]["tasks.unattended"], "default");
        assert_eq!(report["worktrees"]["root"], "~/worktrees");
        let old: Config = toml::from_str("[agents]\n").unwrap();
        assert!(!old.tasks.unattended);
        assert!(!old.worktrees.allow_creation);
    }

    #[test]
    fn config_task_policy_invalid_limits_and_project_overrides_rejected() {
        assert!(Config::policy_from_toml("[tasks]\nmax_attempts = 0", "fixture", None).is_err());
        assert!(
            Config::policy_from_toml("[worktrees]\nroot = '../escape'", "fixture", None).is_err()
        );
        assert!(
            Config::policy_from_toml("[worktrees]\ncleanup = 'automatic'", "fixture", None)
                .is_err()
        );
        let error = Config::policy_from_toml("", "default", Some("[tasks]\nunattended = true"))
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Project task/worktree/setup policy is unsupported")
        );
    }
}
