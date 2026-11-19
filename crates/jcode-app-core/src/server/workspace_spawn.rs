//! Workspace-aware swarm spawn: model override and coordinator provenance.
//!
//! Both resolve against the worker's *target* directory, never the
//! coordinator's, and run git-touching work off the async executor.

use std::path::PathBuf;

/// Model for a spawn: explicit request > workspace role slot / `swarm_model`
/// (resolved for the worker's *target* dir) > global role slot. `None` falls
/// through to `agents.swarm_model` and then coordinator inheritance. Callers
/// validate the explicit request first, so configured models skip the
/// agent-selection gate.
pub(super) async fn resolve_requested_model(
    requested_model: Option<String>,
    target_dir: Option<&str>,
    role: Option<&str>,
) -> Option<String> {
    if requested_model
        .as_deref()
        .is_some_and(|model| !model.trim().is_empty())
    {
        return requested_model;
    }
    let dir = target_dir
        .filter(|dir| !dir.trim().is_empty())
        .map(PathBuf::from);
    let role_owned = role.map(str::to_string);
    let workspace = tokio::task::spawn_blocking(move || {
        crate::workspace::spawn_model_override(dir.as_deref(), role_owned.as_deref())
    })
    .await
    .ok()
    .flatten();
    workspace.map(|over| over.model).or_else(|| {
        let agents = &crate::config::config().agents;
        role.and_then(|role| agents.roles.get(role))
            .map(str::to_string)
    })
}

/// Remember which workspace the coordinator was in when it spawned the worker.
pub(super) async fn record_provenance(new_session_id: &str, coordinator_dir: Option<String>) {
    let Some(dir) = coordinator_dir.filter(|dir| !dir.trim().is_empty()) else {
        return;
    };
    let session_id = new_session_id.to_string();
    let _ = tokio::task::spawn_blocking(move || {
        crate::workspace::record_provenance(&session_id, Some(&PathBuf::from(dir)));
    })
    .await;
}

#[cfg(test)]
#[path = "workspace_spawn_tests.rs"]
mod tests;
