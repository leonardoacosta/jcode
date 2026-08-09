use jcode_session_types::{
    ContextConfidence, ContextProvenance, ContextRow, ContextSection, ContextSnapshot,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedContextIdentity {
    pub organization_id: Option<String>,
    pub organization_name: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub project_root: Option<String>,
    pub workspace_id: Option<String>,
    pub workspace_path: Option<String>,
    pub initiative_id: Option<String>,
    pub initiative_name: Option<String>,
    pub task_run_id: Option<String>,
    pub task_run_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurrentSessionContext {
    pub session_id: String,
    pub title: Option<String>,
    pub working_dir: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub resume_group: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HerdrContext {
    pub pane_id: Option<String>,
    pub workspace: Option<String>,
    pub harness_state: Option<String>,
    pub unavailable_reason: Option<String>,
}

pub fn build_context_snapshot(
    persisted: Option<&PersistedContextIdentity>,
    current: &CurrentSessionContext,
    herdr: Option<&HerdrContext>,
) -> ContextSnapshot {
    let persisted = persisted.cloned().unwrap_or_default();
    let project_label = persisted
        .project_name
        .clone()
        .or_else(|| project_name_from_path(current.working_dir.as_deref()));
    let project_row = if let Some(project_id) = persisted.project_id.clone() {
        ContextRow::persisted(
            "Project",
            project_label.unwrap_or_else(|| project_id.clone()),
            project_id,
        )
    } else if let Some(label) = project_label {
        inferred_row("Project", label)
    } else {
        ContextRow::unavailable("Project", "no project context")
    };

    let workspace_row = if let Some(workspace_id) = persisted.workspace_id.clone() {
        ContextRow::persisted(
            "Workspace",
            persisted
                .workspace_path
                .clone()
                .or_else(|| current.working_dir.clone())
                .unwrap_or_else(|| workspace_id.clone()),
            workspace_id,
        )
    } else if let Some(path) = current.working_dir.clone() {
        ContextRow::current("Workspace", path)
    } else {
        ContextRow::unavailable("Workspace", "no working directory")
    };

    let organization_row = match (persisted.organization_id, persisted.organization_name) {
        (Some(id), Some(name)) => ContextRow::persisted("Organization", name, id),
        (Some(id), None) => ContextRow::persisted("Organization", id.clone(), id),
        _ => ContextRow::unavailable("Organization", "no organization selected"),
    };

    let initiative_row = match (persisted.initiative_id, persisted.initiative_name) {
        (Some(id), Some(name)) => ContextRow::persisted("Initiative", name, id),
        (Some(id), None) => ContextRow::persisted("Initiative", id.clone(), id),
        _ => ContextRow::unavailable("Initiative", "no initiative selected"),
    };

    let task_row = match (persisted.task_run_id, persisted.task_run_name) {
        (Some(id), Some(name)) => ContextRow::persisted("Task/run", name, id),
        (Some(id), None) => ContextRow::persisted("Task/run", id.clone(), id),
        _ => ContextRow::unavailable("Task/run", "no task/run selected"),
    };

    let mut session_row = ContextRow::current(
        "Jcode session",
        current
            .title
            .clone()
            .unwrap_or_else(|| current.session_id.clone()),
    );
    session_row.stable_id = Some(current.session_id.clone());
    session_row.focusable = true;

    let mut provider_row = ContextRow::unavailable("Provider/model", "provider/model unavailable");
    if current.provider.is_some() || current.model.is_some() {
        provider_row = ContextRow::current(
            "Provider/model",
            format!(
                "{} / {}",
                current.provider.as_deref().unwrap_or("unknown"),
                current.model.as_deref().unwrap_or("unknown")
            ),
        );
        provider_row.copyable = false;
    }

    let herdr_row = match herdr {
        Some(meta) if meta.pane_id.is_some() => {
            let mut row = ContextRow {
                label: "Herdr pane".to_string(),
                value: Some(format!(
                    "{}{}{}",
                    meta.pane_id.as_deref().unwrap_or("unknown"),
                    meta.workspace
                        .as_deref()
                        .map(|w| format!(" · {w}"))
                        .unwrap_or_default(),
                    meta.harness_state
                        .as_deref()
                        .map(|s| format!(" · {s}"))
                        .unwrap_or_default()
                )),
                stable_id: meta.pane_id.clone(),
                provenance: ContextProvenance::Herdr,
                confidence: ContextConfidence::Inferred,
                copyable: true,
                focusable: true,
                unavailable_reason: None,
            };
            if row.value.as_deref() == Some("") {
                row.value = meta.pane_id.clone();
            }
            row
        }
        Some(meta) => ContextRow::unavailable(
            "Herdr pane",
            meta.unavailable_reason
                .clone()
                .unwrap_or_else(|| "Herdr metadata unavailable".to_string()),
        ),
        None => ContextRow::unavailable("Herdr pane", "Herdr not detected"),
    };

    ContextSnapshot {
        semantic: ContextSection {
            title: "Semantic context".to_string(),
            rows: vec![
                organization_row,
                project_row,
                workspace_row,
                initiative_row,
                task_row,
            ],
        },
        execution: ContextSection {
            title: "Execution substrate".to_string(),
            rows: vec![session_row, provider_row, herdr_row],
        },
    }
}

pub fn load_persisted_context(path: &Path) -> anyhow::Result<Option<PersistedContextIdentity>> {
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&data)?))
}

pub fn save_persisted_context(
    path: &Path,
    identity: &PersistedContextIdentity,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(identity)?)?;
    Ok(())
}

pub fn session_context_path(base: &Path, session_id: &str) -> PathBuf {
    let safe = session_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    base.join("context").join(format!("{safe}.json"))
}

fn inferred_row(label: &str, value: String) -> ContextRow {
    ContextRow {
        label: label.to_string(),
        value: Some(value.clone()),
        stable_id: Some(value),
        provenance: ContextProvenance::InferredFromPath,
        confidence: ContextConfidence::Inferred,
        copyable: true,
        focusable: false,
        unavailable_reason: None,
    }
}

fn project_name_from_path(path: Option<&str>) -> Option<String> {
    let path = Path::new(path?);
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_snapshot_prefers_persisted_project_over_home_reconnect() {
        let persisted = PersistedContextIdentity {
            project_id: Some("project:jcode".into()),
            project_name: Some("jcode".into()),
            workspace_id: Some("workspace:source".into()),
            workspace_path: Some("/home/nyaptor/dev/jcode/source/jcode".into()),
            ..Default::default()
        };
        let current = CurrentSessionContext {
            session_id: "s1".into(),
            working_dir: Some("/home/nyaptor".into()),
            ..Default::default()
        };
        let snapshot = build_context_snapshot(Some(&persisted), &current, None);
        assert_eq!(
            snapshot.semantic.rows[1].stable_id.as_deref(),
            Some("project:jcode")
        );
        assert_eq!(
            snapshot.semantic.rows[2].display_value(),
            "/home/nyaptor/dev/jcode/source/jcode"
        );
        assert_eq!(
            snapshot.semantic.rows[1].provenance,
            ContextProvenance::Persisted
        );
    }

    #[test]
    fn context_snapshot_degrades_when_herdr_is_absent() {
        let current = CurrentSessionContext {
            session_id: "s1".into(),
            working_dir: Some("/repo/jcode".into()),
            ..Default::default()
        };
        let snapshot = build_context_snapshot(None, &current, None);
        let herdr = snapshot
            .execution
            .rows
            .iter()
            .find(|r| r.label == "Herdr pane")
            .unwrap();
        assert_eq!(herdr.provenance, ContextProvenance::Unavailable);
        assert!(herdr.display_value().contains("Herdr not detected"));
    }

    #[test]
    fn persisted_context_round_trips() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = session_context_path(temp.path(), "a/b");
        let identity = PersistedContextIdentity {
            project_id: Some("project".into()),
            ..Default::default()
        };
        save_persisted_context(&path, &identity).unwrap();
        assert_eq!(load_persisted_context(&path).unwrap(), Some(identity));
        assert!(path.ends_with("context/a_b.json"));
    }
}
