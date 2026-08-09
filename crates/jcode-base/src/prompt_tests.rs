use super::*;

/// Verify the default system prompt does NOT identify as "Claude Code"
/// It's fine to say "powered by Claude" but not "Claude Code" (Anthropic's product)
#[test]
fn test_default_system_prompt_no_claude_code_identity() {
    let prompt = DEFAULT_SYSTEM_PROMPT.to_lowercase();

    assert!(
        !prompt.contains("claude code"),
        "DEFAULT_SYSTEM_PROMPT should NOT identify as 'Claude Code'. Found in system_prompt.md"
    );
    assert!(
        !prompt.contains("claude-code"),
        "DEFAULT_SYSTEM_PROMPT should NOT contain 'claude-code'. Found in system_prompt.md"
    );
}

#[test]
fn mermaid_prompt_module_follows_capability() {
    let (enabled, _) = build_system_prompt_split_with_capabilities(
        None,
        &[],
        false,
        None,
        None,
        PromptCapabilities { mermaid: true },
    );
    assert!(enabled.static_part.contains(MERMAID_PROMPT));

    let (disabled, _) = build_system_prompt_split_with_capabilities(
        None,
        &[],
        false,
        None,
        None,
        PromptCapabilities { mermaid: false },
    );
    assert!(!disabled.static_part.contains("Mermaid diagrams"));
    assert!(!disabled.static_part.contains("fenced `mermaid` code block"));
}

/// Verify skill prompts don't accidentally introduce "Claude Code" identity
#[test]
fn test_skill_prompt_integration() {
    // Test that a skill prompt is properly appended and doesn't break anything
    let skill_prompt = "You are helping with a debugging task.";
    let prompt = build_system_prompt(Some(skill_prompt), &[]);

    // The prompt should contain our default system prompt
    assert!(prompt.contains("Your name is Jcode."));

    // The prompt should contain the skill prompt
    assert!(prompt.contains(skill_prompt));

    // The base prompt parts (excluding user-provided instruction files) should NOT contain
    // "Claude Code". We check DEFAULT_SYSTEM_PROMPT separately since user files may
    // legitimately contain it.
    let default_lower = DEFAULT_SYSTEM_PROMPT.to_lowercase();
    assert!(
        !default_lower.contains("claude code"),
        "DEFAULT_SYSTEM_PROMPT should NOT identify as 'Claude Code'"
    );
}

#[test]
fn test_load_agents_md_files_uses_sandboxed_global_files() {
    let _guard = crate::storage::lock_test_env();
    let prev_home = std::env::var_os("JCODE_HOME");
    let temp = tempfile::TempDir::new().unwrap();
    crate::env::set_var("JCODE_HOME", temp.path());
    std::fs::create_dir_all(temp.path().join("external")).unwrap();

    std::fs::write(
        temp.path().join("external/AGENTS.md"),
        "sandboxed global agents instructions",
    )
    .unwrap();

    let project_dir = tempfile::TempDir::new().unwrap();
    let (content, info) = load_agents_md_files_from_dir(Some(project_dir.path()));

    assert!(info.has_global_agents_md);
    let content = content.expect("global instructions content");
    assert!(content.contains("# Global Instructions (~/AGENTS.md)"));
    assert!(!content.contains("~/.AGENTS.md"));
    assert!(content.contains("sandboxed global agents instructions"));

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[test]
fn test_session_context_includes_time_timezone_and_system_info() {
    let context = build_session_context(None);
    assert!(context.contains("# Session Context"));
    assert!(context.contains("Time: "));
    assert!(context.contains("Timezone: UTC"));
    assert!(context.contains("OS: "));
    assert!(context.contains("Architecture: "));
    assert!(context.contains("Jcode version: "));
    assert!(!context.contains("Working directory: "));
    assert!(!context.contains("Git:"));
}

#[test]
fn test_split_prompt_does_not_inject_session_context_per_turn() {
    let (split, _info) = build_system_prompt_split(None, &[], false, None, None);
    assert!(!split.dynamic_part.contains("# Session Context"));
    assert!(!split.dynamic_part.contains("Time: "));
    assert!(!split.dynamic_part.contains("Timezone: UTC"));
}

#[test]
fn sponsored_discovery_is_not_injected_into_the_system_prompt() {
    let (split, _) = build_system_prompt_split(None, &[], false, None, None);
    assert!(!split.static_part.contains("Discoverable Tools"));
    assert!(!split.static_part.contains("integration_tools"));
}

#[test]
fn test_prompt_overlay_files_are_loaded_from_project_and_global_jcode_dirs() {
    let _guard = crate::storage::lock_test_env();
    let prev_home = std::env::var_os("JCODE_HOME");
    let temp = tempfile::TempDir::new().unwrap();
    crate::env::set_var("JCODE_HOME", temp.path());
    std::fs::create_dir_all(temp.path()).unwrap();
    std::fs::write(
        temp.path().join("prompt-overlay.md"),
        "global prompt overlay instructions",
    )
    .unwrap();

    let project_dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project_dir.path().join(".jcode")).unwrap();
    std::fs::write(
        project_dir.path().join(".jcode/prompt-overlay.md"),
        "project prompt overlay instructions",
    )
    .unwrap();

    let direct = load_prompt_overlay_files_from_dir(Some(project_dir.path()));

    assert!(direct.0.is_some(), "expected prompt overlay content");
    let direct_content = direct.0.unwrap();
    assert!(
        direct_content.contains("project prompt overlay instructions"),
        "expected project prompt overlay content"
    );
    assert!(
        direct_content.contains("global prompt overlay instructions"),
        "expected global prompt overlay content"
    );

    let (prompt, info) = build_system_prompt_full(None, &[], false, None, Some(project_dir.path()));
    assert!(prompt.contains("project prompt overlay instructions"));
    assert!(prompt.contains("global prompt overlay instructions"));
    assert!(info.prompt_overlay_chars > 0);

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[test]
fn test_preferred_tools_files_are_loaded_from_project_and_global_jcode_dirs() {
    let _guard = crate::storage::lock_test_env();
    let prev_home = std::env::var_os("JCODE_HOME");
    let temp = tempfile::TempDir::new().unwrap();
    crate::env::set_var("JCODE_HOME", temp.path());
    std::fs::create_dir_all(temp.path()).unwrap();
    std::fs::write(
        temp.path().join("preferred-tools.md"),
        "global preferred tools instructions",
    )
    .unwrap();

    let project_dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project_dir.path().join(".jcode")).unwrap();
    std::fs::write(
        project_dir.path().join(".jcode/preferred-tools.md"),
        "project preferred tools instructions",
    )
    .unwrap();

    let direct = load_preferred_tools_files_from_dir(Some(project_dir.path()));

    assert!(direct.0.is_some(), "expected preferred tools content");
    let direct_content = direct.0.unwrap();
    assert!(
        direct_content.contains("Project Preferred Tools (.jcode/preferred-tools.md)"),
        "expected project preferred tools section heading"
    );
    assert!(
        direct_content.contains("project preferred tools instructions"),
        "expected project preferred tools content"
    );
    assert!(
        direct_content.contains("Global Preferred Tools (~/.jcode/preferred-tools.md)"),
        "expected global preferred tools section heading"
    );
    assert!(
        direct_content.contains("global preferred tools instructions"),
        "expected global preferred tools content"
    );

    let (prompt, info) = build_system_prompt_full(None, &[], false, None, Some(project_dir.path()));
    assert!(prompt.contains("project preferred tools instructions"));
    assert!(prompt.contains("global preferred tools instructions"));
    assert!(info.preferred_tools_chars > 0);

    let (split, split_info) =
        build_system_prompt_split(None, &[], false, None, Some(project_dir.path()));
    assert!(
        split
            .static_part
            .contains("project preferred tools instructions")
    );
    assert!(
        split
            .static_part
            .contains("global preferred tools instructions")
    );
    assert!(split_info.preferred_tools_chars > 0);

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[test]
fn test_swarm_prompt_prefers_project_then_global_then_default() {
    let _guard = crate::storage::lock_test_env();
    let prev_home = std::env::var_os("JCODE_HOME");
    let temp = tempfile::TempDir::new().unwrap();
    crate::env::set_var("JCODE_HOME", temp.path());
    std::fs::create_dir_all(temp.path()).unwrap();

    let project_dir = tempfile::TempDir::new().unwrap();

    // No override files: built-in default.
    let prompt = load_swarm_prompt(Some(project_dir.path()));
    assert_eq!(prompt, DEFAULT_SWARM_PROMPT.trim());

    // Global override wins over the default.
    std::fs::write(temp.path().join("swarm-prompt.md"), "global swarm routing").unwrap();
    let prompt = load_swarm_prompt(Some(project_dir.path()));
    assert_eq!(prompt, "global swarm routing");

    // Project override wins over global.
    std::fs::create_dir_all(project_dir.path().join(".jcode")).unwrap();
    std::fs::write(
        project_dir.path().join(".jcode/swarm-prompt.md"),
        "project swarm routing",
    )
    .unwrap();
    let prompt = load_swarm_prompt(Some(project_dir.path()));
    assert_eq!(prompt, "project swarm routing");

    // A blank project file falls through to global instead of going empty.
    std::fs::write(project_dir.path().join(".jcode/swarm-prompt.md"), "   \n").unwrap();
    let prompt = load_swarm_prompt(Some(project_dir.path()));
    assert_eq!(prompt, "global swarm routing");

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[test]
fn test_default_swarm_prompt_mentions_model_and_list_models() {
    assert!(DEFAULT_SWARM_PROMPT.contains("list_models"));
    assert!(DEFAULT_SWARM_PROMPT.contains("model"));
    assert!(DEFAULT_SWARM_PROMPT.contains("effort"));
    assert!(DEFAULT_SWARM_PROMPT.contains("only the root session may spawn agents"));
    assert!(DEFAULT_SWARM_PROMPT.contains("swarm-deep"));
}

#[test]
fn test_non_selfdev_prompt_leaves_selfdev_guidance_to_the_tool_schema() {
    let prompt = build_system_prompt(None, &[]);
    assert!(!prompt.contains("Self-Development Access"));
    assert!(!prompt.contains("You have access to the `selfdev` tool in all sessions"));
    assert!(!prompt.contains("You are working on the jcode codebase itself."));
}

#[test]
fn test_selfdev_prompt_uses_full_selfdev_instructions() {
    let prompt = build_system_prompt_with_selfdev(None, &[], true);
    assert!(prompt.contains("You are working on the jcode codebase itself."));
    assert!(prompt.contains("launched from the TUI/root jcode context"));
    assert!(prompt.contains("selfdev build target=tui"));
    assert!(!prompt.contains("Self-Development Access"));
}

#[test]
fn test_selfdev_prompt_uses_desktop_focus_for_desktop_working_dir() {
    let desktop_dir = std::path::Path::new("/tmp/jcode/crates/jcode-desktop2/src");
    let (prompt, _info) = build_system_prompt_full(None, &[], true, None, Some(desktop_dir));
    assert!(prompt.contains("launched from the jcode-desktop2"));
    assert!(prompt.contains("selfdev build target=desktop2"));
    assert!(!prompt.contains("launched from the TUI/root jcode context"));
}

#[test]
fn test_split_selfdev_prompt_defaults_to_tui_focus_for_repo_root() {
    let repo_dir = std::path::Path::new("/tmp/jcode");
    let (split, _info) = build_system_prompt_split(None, &[], true, None, Some(repo_dir));
    assert!(
        split
            .static_part
            .contains("launched from the TUI/root jcode context")
    );
    assert!(split.static_part.contains("selfdev build target=tui"));
}

#[test]
fn test_selfdev_prompt_prefers_publish_flow_for_active_builds() {
    let prompt = build_system_prompt_with_selfdev(None, &[], true);
    assert!(prompt.contains("selfdev build"));
    assert!(prompt.contains("cancel-build"));
    assert!(prompt.contains("selfdev reload"));
    assert!(prompt.contains("fallback when `selfdev build` is not appropriate"));
    assert!(prompt.contains("scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode"));
    assert!(prompt.contains("remote build host is configured"));
    assert!(prompt.contains("Do not wait for user input"));
}

#[test]
fn test_selfdev_prompt_template_placeholders_are_resolved() {
    let static_prompt = build_selfdev_prompt_static();
    let dynamic_prompt = build_selfdev_prompt();
    assert!(!static_prompt.contains("__DEBUG_SOCKET_BLOCK__"));
    assert!(!dynamic_prompt.contains("__DEBUG_SOCKET_BLOCK__"));
    assert!(!static_prompt.contains("__SELFDEV_PRODUCT_FOCUS__"));
    assert!(!dynamic_prompt.contains("__SELFDEV_PRODUCT_FOCUS__"));
    assert_eq!(static_prompt, dynamic_prompt);
}

#[test]
fn split_prompt_estimated_tokens_is_positive_when_populated() {
    let (split, _info) = build_system_prompt_split(None, &[], false, None, None);
    assert!(split.chars() > 0);
    assert!(split.estimated_tokens() > 0);
}

#[test]
fn swarm_effort_directive_is_appended_only_for_swarm_sentinel() {
    assert!(is_swarm_effort("swarm"));
    assert!(is_swarm_effort("  Swarm "));
    assert!(!is_swarm_effort("xhigh"));

    let mut split = SplitSystemPrompt {
        static_part: "base".to_string(),
        dynamic_part: String::new(),
    };
    append_swarm_effort_directive(&mut split, Some("xhigh"));
    assert!(!split.dynamic_part.contains("Swarm Effort"));

    append_swarm_effort_directive(&mut split, Some("swarm"));
    assert!(split.dynamic_part.contains("# Swarm Effort"));
    assert!(split.dynamic_part.contains("swarm` tool"));

    // None / empty effort should not inject.
    let mut other = SplitSystemPrompt::default();
    append_swarm_effort_directive(&mut other, None);
    assert!(other.dynamic_part.is_empty());
}

#[test]
fn swarm_deep_effort_injects_task_graph_directive() {
    use crate::prompt::is_deep_swarm_effort;

    assert!(is_swarm_effort("swarm-deep"));
    assert!(is_deep_swarm_effort("swarm-deep"));
    assert!(is_deep_swarm_effort("  Swarm-Deep "));
    assert!(!is_deep_swarm_effort("swarm"));
    assert!(!is_deep_swarm_effort("xhigh"));

    // Deep sentinel injects the DAG-first task-graph directive, not the light one.
    let mut split = SplitSystemPrompt::default();
    append_swarm_effort_directive(&mut split, Some("swarm-deep"));
    assert!(split.dynamic_part.contains("# Deep Task Graph"));
    assert!(split.dynamic_part.contains("swarm task_graph"));
    assert!(!split.dynamic_part.contains("# Swarm Effort"));

    // Light sentinel still injects the fan-out directive, not the deep one.
    let mut light = SplitSystemPrompt::default();
    append_swarm_effort_directive(&mut light, Some("swarm"));
    assert!(light.dynamic_part.contains("# Swarm Effort"));
    assert!(!light.dynamic_part.contains("# Deep Task Graph"));
}

#[test]
fn classify_effort_distinguishes_reasoning_from_swarm_modes() {
    use crate::prompt::{EffortKind, classify_effort, is_swarm_mode_effort};

    // Plain reasoning levels are not swarm modes.
    for level in ["none", "minimal", "low", "medium", "high", "xhigh", "max"] {
        assert_eq!(classify_effort(level), EffortKind::Reasoning, "{level}");
        assert!(!is_swarm_mode_effort(level), "{level}");
    }

    assert_eq!(classify_effort("swarm"), EffortKind::SwarmLight);
    assert_eq!(classify_effort("swarm-deep"), EffortKind::SwarmDeep);
    assert!(is_swarm_mode_effort("swarm"));
    assert!(is_swarm_mode_effort("  Swarm-Deep "));
    assert!(EffortKind::SwarmLight.is_swarm_mode());
    assert!(EffortKind::SwarmDeep.is_swarm_mode());
    assert!(!EffortKind::Reasoning.is_swarm_mode());
}

#[test]
fn test_selfdev_prompt_uses_desktop2_focus_for_desktop2_working_dir() {
    let desktop2_dir = std::path::Path::new("/tmp/jcode/crates/jcode-desktop2/src");
    let (prompt, _info) = build_system_prompt_full(None, &[], true, None, Some(desktop2_dir));
    assert!(prompt.contains("launched from the jcode-desktop2"));
    assert!(prompt.contains("selfdev build target=desktop2"));
    assert!(!prompt.contains("launched from the TUI/root jcode context"));
}

#[test]
fn project_system_prompt_file_replaces_default_base_prompt() {
    use crate::prompt::load_base_system_prompt;

    let dir = std::env::temp_dir().join(format!("jcode-sysprompt-{}", std::process::id()));
    let jcode_dir = dir.join(".jcode");
    std::fs::create_dir_all(&jcode_dir).unwrap();
    std::fs::write(
        jcode_dir.join("system-prompt.md"),
        "You are a custom agent.\n",
    )
    .unwrap();

    assert_eq!(
        load_base_system_prompt(Some(&dir)),
        "You are a custom agent."
    );

    let (prompt, _info) = build_system_prompt_full(None, &[], false, None, Some(&dir));
    assert!(prompt.contains("You are a custom agent."));
    assert!(!prompt.contains("Jcode is open source"));

    // Empty override falls back to the built-in default.
    std::fs::write(jcode_dir.join("system-prompt.md"), "   \n").unwrap();
    assert_eq!(load_base_system_prompt(Some(&dir)), DEFAULT_SYSTEM_PROMPT);

    std::fs::remove_dir_all(&dir).ok();
}

// === Roadmap P1: prompt assembly contract ===

struct JcodeHomeGuard {
    prev: Option<std::ffi::OsString>,
}

impl JcodeHomeGuard {
    fn set(path: &std::path::Path) -> Self {
        let prev = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", path);
        Self { prev }
    }
}

impl Drop for JcodeHomeGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(value) => crate::env::set_var("JCODE_HOME", value),
            None => crate::env::remove_var("JCODE_HOME"),
        }
    }
}

fn contract_test_project() -> (
    std::sync::MutexGuard<'static, ()>,
    tempfile::TempDir,
    tempfile::TempDir,
    JcodeHomeGuard,
) {
    let guard = crate::storage::lock_test_env();
    let home = tempfile::TempDir::new().unwrap();
    let home_guard = JcodeHomeGuard::set(home.path());
    let project = tempfile::TempDir::new().unwrap();
    (guard, home, project, home_guard)
}

#[test]
fn assembly_layer_order_matches_contract() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::write(dir.join("AGENTS.md"), "project agents").unwrap();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "overlay").unwrap();
    std::fs::write(dir.join(".jcode/preferred-tools.md"), "prefer rg").unwrap();

    let skills = vec![SkillInfo {
        name: "demo".to_string(),
        description: "demo skill".to_string(),
    }];
    let (assembly, info) = build_prompt_assembly_with_capabilities(
        Some("skill prompt body"),
        &skills,
        false,
        Some("memory body"),
        Some(dir),
        PromptCapabilities { mermaid: true },
    );

    let static_ids: Vec<&str> = assembly.static_layers.iter().map(|l| l.id).collect();
    assert_eq!(
        static_ids,
        vec![
            "base",
            "capability:mermaid",
            "agents-md-project",
            "prompt-overlay-project",
            "preferred-tools-project",
            "skills-list",
        ],
        "static layer order must match the contract"
    );
    let dynamic_ids: Vec<&str> = assembly.dynamic_layers.iter().map(|l| l.id).collect();
    assert_eq!(dynamic_ids, vec!["memory", "active-skill"]);
    assert_eq!(assembly.version, PROMPT_ASSEMBLY_VERSION);
    assert!(assembly.digest.starts_with("prompt:1:"));
    assert_eq!(info.prompt_digest, assembly.digest);
    assert!(!info.layer_attribution.is_empty());
}

#[test]
fn assembly_selfdev_layer_sits_between_base_and_agents_md() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::write(dir.join("AGENTS.md"), "project agents").unwrap();

    let (assembly, _info) = build_prompt_assembly_with_capabilities(
        None,
        &[],
        true,
        None,
        Some(dir),
        PromptCapabilities { mermaid: false },
    );
    let static_ids: Vec<&str> = assembly.static_layers.iter().map(|l| l.id).collect();
    let base_pos = static_ids.iter().position(|id| *id == "base").unwrap();
    let selfdev_pos = static_ids.iter().position(|id| *id == "selfdev").unwrap();
    let agents_pos = static_ids
        .iter()
        .position(|id| *id == "agents-md-project")
        .unwrap();
    assert!(base_pos < selfdev_pos && selfdev_pos < agents_pos);
}

#[test]
fn assembly_split_is_byte_identical_to_legacy_split() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::write(dir.join("AGENTS.md"), "project agents").unwrap();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "overlay").unwrap();

    let skills = vec![SkillInfo {
        name: "demo".to_string(),
        description: "demo skill".to_string(),
    }];
    for &(skill_prompt, memory, selfdev) in &[
        (None, None, false),
        (Some("skill body"), Some("memory body"), false),
        (None, Some("memory body"), true),
    ] {
        let (assembly, _) = build_prompt_assembly_with_capabilities(
            skill_prompt,
            &skills,
            selfdev,
            memory,
            Some(dir),
            PromptCapabilities::current(),
        );
        let (legacy, _) = build_system_prompt_split_with_capabilities(
            skill_prompt,
            &skills,
            selfdev,
            memory,
            Some(dir),
            PromptCapabilities::current(),
        );
        let split = assembly.split();
        assert_eq!(split.static_part, legacy.static_part, "static mismatch");
        assert_eq!(split.dynamic_part, legacy.dynamic_part, "dynamic mismatch");
    }
}

#[test]
fn prompt_digest_is_reproducible_and_sensitive() {
    let layer =
        |id: &'static str, source: PromptLayerSource, mode: PromptLayerMode, content: &str| {
            PromptLayer {
                id,
                source,
                mode,
                content: content.to_string(),
            }
        };
    let base = vec![
        layer(
            "base",
            PromptLayerSource::Builtin,
            PromptLayerMode::Append,
            "aaa",
        ),
        layer(
            "agents-md-project",
            PromptLayerSource::ProjectFile(std::path::PathBuf::from("/x/AGENTS.md")),
            PromptLayerMode::Append,
            "bbb",
        ),
    ];
    let digest_a = prompt_digest(1, &base);
    let digest_b = prompt_digest(1, &base);
    assert_eq!(digest_a, digest_b, "identical inputs must reproduce");
    assert!(digest_a.starts_with("prompt:1:"));
    let hex_part = digest_a.trim_start_matches("prompt:1:");
    assert_eq!(hex_part.len(), 16);
    assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));

    // Content change changes the digest.
    let mut changed = base.clone();
    changed[1].content = "bbc".to_string();
    assert_ne!(prompt_digest(1, &changed), digest_a);

    // Version change changes the digest.
    assert_ne!(prompt_digest(2, &base), digest_a);

    // Source *kind* participates: builtin vs project-file with same content.
    let as_builtin = vec![layer(
        "base",
        PromptLayerSource::Builtin,
        PromptLayerMode::Append,
        "same",
    )];
    let as_project = vec![layer(
        "base",
        PromptLayerSource::ProjectFile(std::path::PathBuf::from("/y/base.md")),
        PromptLayerMode::Append,
        "same",
    )];
    assert_ne!(prompt_digest(1, &as_builtin), prompt_digest(1, &as_project));

    // Paths deliberately do not participate: same kind+content, different path.
    let path_one = vec![layer(
        "base",
        PromptLayerSource::ProjectFile(std::path::PathBuf::from("/m1/AGENTS.md")),
        PromptLayerMode::Append,
        "same",
    )];
    let path_two = vec![layer(
        "base",
        PromptLayerSource::ProjectFile(std::path::PathBuf::from("/m2/AGENTS.md")),
        PromptLayerMode::Append,
        "same",
    )];
    assert_eq!(prompt_digest(1, &path_one), prompt_digest(1, &path_two));

    // Runtime labels do not participate either (kind does).
    let rt_one = vec![layer(
        "memory",
        PromptLayerSource::Runtime("memory"),
        PromptLayerMode::Append,
        "same",
    )];
    let rt_two = vec![layer(
        "memory",
        PromptLayerSource::Runtime("other-label"),
        PromptLayerMode::Append,
        "same",
    )];
    assert_eq!(prompt_digest(1, &rt_one), prompt_digest(1, &rt_two));

    // Layer order participates.
    let reversed: Vec<PromptLayer> = base.iter().rev().cloned().collect();
    assert_ne!(prompt_digest(1, &reversed), digest_a);
}

#[test]
fn assembly_digest_changes_when_project_files_change() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "v1").unwrap();

    let (first, _) = build_prompt_assembly(None, &[], false, None, Some(dir));
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "v2-edited").unwrap();
    let (second, _) = build_prompt_assembly(None, &[], false, None, Some(dir));
    assert_ne!(
        first.digest, second.digest,
        "digest tracks static layer content"
    );
    assert!(second.static_text.contains("v2-edited"));
}

#[test]
fn frozen_assembly_capture_and_reuse() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "frozen-overlay").unwrap();

    let (assembly, info) = build_prompt_assembly(None, &[], false, Some("mem"), Some(dir));
    let frozen = FrozenPromptAssembly::capture(&assembly, &info);
    assert_eq!(frozen.version, PROMPT_ASSEMBLY_VERSION);
    assert_eq!(frozen.static_text, assembly.static_text);
    assert_eq!(frozen.digest, assembly.digest);
    assert_eq!(frozen.attribution, assembly.attribution());
    assert_eq!(frozen.context_info.prompt_digest, assembly.digest);

    // Reuse after a mid-session edit: frozen text/digest/attribution win.
    std::fs::write(dir.join(".jcode/prompt-overlay.md"), "edited-later").unwrap();
    let (later, _) = build_prompt_assembly(None, &[], false, Some("mem2"), Some(dir));
    let mut split = later.split();
    split.static_part = frozen.static_text.clone();
    assert!(split.static_part.contains("frozen-overlay"));
    assert!(!split.static_part.contains("edited-later"));
    assert!(split.dynamic_part.contains("mem2"));
    assert_eq!(frozen.digest, assembly.digest);
    assert_ne!(later.digest, frozen.digest);
}

#[test]
fn override_prompt_assembly_is_a_single_replace_layer() {
    let (assembly, info) = override_prompt_assembly("OVERRIDE BODY");
    assert_eq!(assembly.static_layers.len(), 1);
    assert_eq!(assembly.static_layers[0].id, "override");
    assert_eq!(assembly.static_layers[0].mode, PromptLayerMode::Replace);
    assert!(assembly.dynamic_layers.is_empty());
    assert_eq!(assembly.static_text, "OVERRIDE BODY");
    assert!(assembly.digest.starts_with("prompt:1:"));
    assert_eq!(info.system_prompt_chars, "OVERRIDE BODY".len());
    assert_eq!(info.total_chars, "OVERRIDE BODY".len());
    assert_eq!(info.prompt_digest, assembly.digest);
    assert_eq!(info.layer_attribution.len(), 1);
    assert_eq!(info.layer_attribution[0].source_label, "runtime: override");
}

#[test]
fn assembly_attribution_records_sources_and_chars() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::write(dir.join("AGENTS.md"), "agents body").unwrap();

    let (assembly, _) = build_prompt_assembly(None, &[], false, None, Some(dir));
    let attribution = assembly.attribution();
    let base = attribution.iter().find(|a| a.id == "base").unwrap();
    assert_eq!(base.source_label, "builtin");
    assert_eq!(base.chars, DEFAULT_SYSTEM_PROMPT.len());
    let agents = attribution
        .iter()
        .find(|a| a.id == "agents-md-project")
        .unwrap();
    assert!(agents.source_label.starts_with("project: "));
    let agents_layer = assembly
        .static_layers
        .iter()
        .find(|l| l.id == "agents-md-project")
        .unwrap();
    assert!(agents_layer.content.contains("agents body"));
    assert_eq!(agents.chars, agents_layer.content.len());
    // Dynamic layers are excluded from attribution (turn-scoped).
    let (with_dynamic, _) = build_prompt_assembly(None, &[], false, Some("mem"), Some(dir));
    assert_eq!(with_dynamic.attribution().len(), attribution.len());
}

#[test]
fn assembly_base_replacement_and_empty_fallback() {
    let (_guard, _home, project, _home_guard) = contract_test_project();
    let dir = project.path();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/system-prompt.md"), "Custom base.\n").unwrap();

    let (assembly, _) = build_prompt_assembly(None, &[], false, None, Some(dir));
    let base_layer = &assembly.static_layers[0];
    assert_eq!(base_layer.id, "base");
    assert_eq!(base_layer.mode, PromptLayerMode::Replace);
    assert_eq!(base_layer.content, "Custom base.");
    assert!(matches!(
        base_layer.source,
        PromptLayerSource::ProjectFile(_)
    ));
    assert!(!assembly.static_text.contains("Jcode is open source"));

    // Empty file falls back to the builtin default (Append mode, Builtin).
    std::fs::write(dir.join(".jcode/system-prompt.md"), "   \n").unwrap();
    let (fallback, _) = build_prompt_assembly(None, &[], false, None, Some(dir));
    let fallback_base = &fallback.static_layers[0];
    assert_eq!(fallback_base.mode, PromptLayerMode::Append);
    assert!(matches!(fallback_base.source, PromptLayerSource::Builtin));
    assert_eq!(fallback_base.content, DEFAULT_SYSTEM_PROMPT);
    assert_ne!(assembly.digest, fallback.digest);
}
