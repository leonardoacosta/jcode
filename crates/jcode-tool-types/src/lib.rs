#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub output: String,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub artifact: Option<jcode_message_types::RenderedArtifact>,
    pub outcome: jcode_message_types::ToolOutcome,
    pub images: Vec<ToolImage>,
}

#[derive(Debug, Clone)]
pub struct ToolImage {
    pub media_type: String,
    pub data: String,
    pub label: Option<String>,
}

impl ToolOutput {
    pub fn new(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
            title: None,
            metadata: None,
            artifact: None,
            outcome: jcode_message_types::ToolOutcome::Success,
            images: Vec::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_artifact(mut self, artifact: jcode_message_types::RenderedArtifact) -> Self {
        self.artifact = Some(artifact);
        self
    }

    pub fn with_outcome(mut self, outcome: jcode_message_types::ToolOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn with_image(mut self, media_type: impl Into<String>, data: impl Into<String>) -> Self {
        self.images.push(ToolImage {
            media_type: media_type.into(),
            data: data.into(),
            label: None,
        });
        self
    }

    pub fn with_labeled_image(
        mut self,
        media_type: impl Into<String>,
        data: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        self.images.push(ToolImage {
            media_type: media_type.into(),
            data: data.into(),
            label: Some(label.into()),
        });
        self
    }
}

/// Resolve tool name aliases to their canonical internal names.
///
/// Providers can present tools with Claude Code aliases (e.g. `file_grep`,
/// `shell_exec`) or API namespace prefixes (e.g. `functions.bash`). Models can
/// repeat those names in sub-tool calls such as `batch`, while our registry
/// uses canonical internal names (`agentgrep`, `bash`). This mapping ensures
/// all of those forms resolve correctly.
///
/// This lives in `jcode-tool-types` (rather than the tool `Registry`) so that
/// low-level crates such as config can normalize tool names without depending
/// on the full tool subsystem.
pub fn resolve_tool_name(name: &str) -> &str {
    // Some function-calling APIs expose a recipient such as `functions.bash`.
    // Models occasionally preserve that transport namespace when constructing
    // a nested tool call, especially inside `batch`.
    let name = name.strip_prefix("functions.").unwrap_or(name);

    match name {
        "communicate" => "swarm",
        "task" | "task_runner" => "subagent",
        "launch" => "open",
        "shell" => "bash",
        "shell_exec" => "bash",
        "read_file" => "read",
        "file_read" => "read",
        "write_file" => "write",
        "file_write" => "write",
        "edit_file" => "edit",
        "file_edit" => "edit",
        // The native grep tool was removed in favor of agentgrep, but models
        // still frequently call `grep` (and OAuth's `file_grep`). agentgrep's
        // grep mode accepts `pattern` as an alias for `query`, so these calls
        // work as-is.
        "grep" | "file_grep" => "agentgrep",
        "skill" | "Skill" => "skill_manage",
        // The integration catalog tool was renamed from `discover_tools`;
        // models trained on or resuming from the old vocabulary still emit it.
        "discover_tools" => "integration_tools",
        "todoread" | "todowrite" | "todo_read" | "todo_write" | "todos" => "todo",
        // The Anthropic OAuth surface advertises PascalCase tool names and
        // reverse-maps them provider-side for top-level calls, but nested
        // `batch` subcall names bypass that mapping and resolve here (issue
        // #486). Keep these in sync with anthropic_map_tool_name_from_oauth.
        "Bash" => "bash",
        "Read" => "read",
        "Write" => "write",
        "Edit" => "edit",
        "Grep" => "agentgrep",
        "Agent" => "subagent",
        "ScheduleWakeup" => "schedule",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::{ToolOutput, resolve_tool_name};
    use jcode_message_types::{RenderedArtifact, RenderedArtifactKind};

    #[test]
    fn tool_output_artifacts_are_explicit_only() {
        assert!(ToolOutput::new("plain").artifact.is_none());
        let output = ToolOutput::new("# document").with_artifact(
            RenderedArtifact::new(RenderedArtifactKind::Markdown).with_title("Notes"),
        );
        let artifact = output.artifact.expect("artifact");
        assert_eq!(artifact.kind, RenderedArtifactKind::Markdown);
        assert_eq!(artifact.title.as_deref(), Some("Notes"));
    }

    #[test]
    fn tool_output_defaults_to_success_and_accepts_semantic_outcomes() {
        assert_eq!(
            ToolOutput::new("plain").outcome,
            jcode_message_types::ToolOutcome::Success
        );
        assert_eq!(
            ToolOutput::new("partial")
                .with_outcome(jcode_message_types::ToolOutcome::TimeoutWithProgress)
                .outcome,
            jcode_message_types::ToolOutcome::TimeoutWithProgress
        );
    }

    #[test]
    fn resolve_tool_name_strips_function_namespace_before_alias_resolution() {
        assert_eq!(resolve_tool_name("functions.bash"), "bash");
        assert_eq!(resolve_tool_name("functions.shell_exec"), "bash");
        assert_eq!(resolve_tool_name("functions.file_grep"), "agentgrep");
    }

    #[test]
    fn resolve_tool_name_does_not_strip_unrecognized_namespaces() {
        assert_eq!(
            resolve_tool_name("mcp.functions.bash"),
            "mcp.functions.bash"
        );
    }

    #[test]
    fn resolve_tool_name_maps_pascalcase_oauth_aliases() {
        // Anthropic OAuth advertises PascalCase names; batch subcalls resolve
        // through here rather than the provider-side reverse map (issue #486).
        assert_eq!(resolve_tool_name("Read"), "read");
        assert_eq!(resolve_tool_name("Bash"), "bash");
        assert_eq!(resolve_tool_name("Write"), "write");
        assert_eq!(resolve_tool_name("Edit"), "edit");
        assert_eq!(resolve_tool_name("Grep"), "agentgrep");
        assert_eq!(resolve_tool_name("Agent"), "subagent");
        assert_eq!(resolve_tool_name("ScheduleWakeup"), "schedule");
        assert_eq!(resolve_tool_name("Skill"), "skill_manage");
        assert_eq!(resolve_tool_name("functions.Read"), "read");
    }
}
