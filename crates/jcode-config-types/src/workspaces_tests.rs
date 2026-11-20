use super::*;
use crate::HookEntry;

fn hook(command: &str) -> HookEntry {
    HookEntry {
        command: command.to_string(),
        ..HookEntry::default()
    }
}

fn rule(json: serde_json::Value) -> WorkspaceRule {
    serde_json::from_value(json).expect("rule")
}

#[test]
fn unknown_tracker_warns_naming_rule_and_skips_hints() {
    let config: WorkspacesConfig = serde_json::from_value(serde_json::json!({
        "rules": [
            {"name": "good", "origin": "github.com/a/**", "tracker": "linear"},
            {"name": "typo", "origin": "github.com/b/**", "tracker": "jira",
             "branch_template": "x/<id>"},
        ]
    }))
    .unwrap();
    let warnings = config.warnings();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("typo") && warnings[0].contains("jira"));
    assert!(config.rules[0].hints_valid());
    assert!(!config.rules[1].hints_valid());
}

#[test]
fn empty_name_or_origin_warns() {
    let config: WorkspacesConfig =
        serde_json::from_value(serde_json::json!({"rules": [{"tracker": "ado"}]})).unwrap();
    let warnings = config.warnings().join("\n");
    assert!(warnings.contains("`name` is empty"), "{warnings}");
    assert!(warnings.contains("`origin` is empty"), "{warnings}");
}

#[test]
fn tracker_is_case_insensitive_and_optional() {
    assert_eq!(rule(serde_json::json!({})).tracker_kind(), Ok(None));
    assert_eq!(
        rule(serde_json::json!({"tracker": " ADO "})).tracker_kind(),
        Ok(Some(Tracker::Ado))
    );
    assert_eq!(
        rule(serde_json::json!({"tracker": "none"})).tracker_kind(),
        Ok(Some(Tracker::None))
    );
}

#[test]
fn spawn_model_prefers_role_slot_over_workspace_swarm_model() {
    let rule = rule(serde_json::json!({
        "swarm_model": "ws-default",
        "roles": {"research": "ws-research", "edit": "  "},
    }));
    assert_eq!(
        rule.spawn_model(Some("Research")),
        Some(("ws-research".into(), Some("research".into())))
    );
    // A blank slot, an unknown role and no role all fall back to `swarm_model`.
    for role in [Some("edit"), Some("bogus"), None] {
        assert_eq!(
            rule.spawn_model(role),
            Some(("ws-default".into(), None)),
            "{role:?}"
        );
    }
    assert_eq!(WorkspaceRule::default().spawn_model(Some("research")), None);
}

#[test]
fn workspace_hooks_append_and_matching_command_replaces() {
    let global = HooksConfig {
        lifecycle_pre_tool_use: vec![hook("keep"), hook("shared")],
        pre_tool: Some(crate::HookCommands::one("legacy")),
        ..Default::default()
    };
    let mut replacement = hook("shared");
    replacement.timeout = 99;
    let workspace = HooksConfig {
        lifecycle_pre_tool_use: vec![replacement, hook("extra")],
        lifecycle_stop: vec![hook("on-stop")],
        ..Default::default()
    };

    let merged = global.with_workspace_hooks(&workspace);
    let commands: Vec<_> = merged
        .lifecycle_pre_tool_use
        .iter()
        .map(|entry| (entry.command.as_str(), entry.timeout))
        .collect();
    assert_eq!(commands, [("keep", 10), ("shared", 99), ("extra", 10)]);
    assert_eq!(merged.lifecycle_stop.len(), 1);
    assert!(merged.lifecycle_session_start.is_empty());
    // Global legacy keys survive untouched; the global config is not mutated.
    assert!(merged.pre_tool.is_some());
    assert_eq!(global.lifecycle_pre_tool_use.len(), 2);
}

#[test]
fn legacy_hook_keys_in_workspace_warn() {
    let config: WorkspacesConfig = serde_json::from_value(serde_json::json!({
        "rules": [{"name": "w", "origin": "h/**", "hooks": {"pre_tool": "gate.sh"}}]
    }))
    .unwrap();
    let warnings = config.warnings().join("\n");
    assert!(warnings.contains("workspace rule \"w\""), "{warnings}");
    assert!(warnings.contains("legacy keys are ignored"), "{warnings}");
}

#[test]
fn has_lifecycle_entries_is_per_event() {
    let config: WorkspacesConfig = serde_json::from_value(serde_json::json!({
        "rules": [{"name": "w", "origin": "h/**",
                   "hooks": {"PreToolUse": [{"command": "x"}]}}]
    }))
    .unwrap();
    assert!(config.has_lifecycle_entries("PreToolUse"));
    assert!(!config.has_lifecycle_entries("Stop"));
    assert!(!WorkspacesConfig::default().has_lifecycle_entries("PreToolUse"));
}

#[test]
fn board_binding_parses_and_defaults() {
    let r = rule(serde_json::json!({
        "name": "personal", "origin": "github.com/me/**", "tracker": "linear",
        "board": {"backend": "Linear", "team": " team-id ", "project": "p1"}
    }));
    let board = r.board.expect("board");
    assert_eq!(board.backend_kind(), Some(BoardBackend::Linear));
    assert_eq!(board.team_id(), "team-id");
    assert_eq!(board.project_id(), Some("p1"));
    assert_eq!(board.server_name(), "Linear");
    assert!(
        rule(serde_json::json!({"name": "n", "origin": "o"}))
            .board
            .is_none()
    );
}

#[test]
fn board_binding_warns_on_unknown_backend_and_missing_team() {
    let config: WorkspacesConfig = serde_json::from_value(serde_json::json!({
        "rules": [{"name": "w", "origin": "github.com/a/**",
                   "board": {"backend": "jira", "team": " "}}]
    }))
    .unwrap();
    let warnings = config.warnings();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(warnings.iter().any(|w| w.contains("jira")));
    assert!(warnings.iter().any(|w| w.contains("`team`")));
    assert_eq!(BoardBackend::parse("none"), None);
}

#[test]
fn linear_board_team_must_look_like_a_team_id() {
    let rule = |backend: &str, team: &str| -> Vec<String> {
        let config: WorkspacesConfig = serde_json::from_value(serde_json::json!({
            "rules": [{"name": "w", "origin": "github.com/a/**",
                       "board": {"backend": backend, "team": team}}]
        }))
        .unwrap();
        config.warnings()
    };
    for bad in ["PRICE", "Priceless", "a465c3cf"] {
        let w = rule("linear", bad);
        assert_eq!(w.len(), 1, "{bad}: {w:?}");
        assert!(w[0].contains("not a team id"), "{w:?}");
    }
    assert!(rule("linear", "a465c3cf-3a11-4b7a-a2a0-439a489281f5").is_empty());
    // ADO team identifiers have no such shape rule.
    assert!(rule("ado", "Brown Project").is_empty());
}

#[test]
fn memory_retention_is_off_by_default_and_zero_age_never_deletes() {
    let default = crate::MemoryRetentionConfig::default();
    assert!(!default.enabled && !default.auto_purge);
    assert!(!default.auto_purge_active());
    let on = crate::MemoryRetentionConfig {
        enabled: true,
        auto_purge: true,
        inactive_max_age_days: 30,
    };
    assert!(on.auto_purge_active());
    let enabled_only = crate::MemoryRetentionConfig {
        auto_purge: false,
        ..on.clone()
    };
    assert!(
        !enabled_only.auto_purge_active(),
        "enabled alone is not auto"
    );
    let zero_age = crate::MemoryRetentionConfig {
        inactive_max_age_days: 0,
        ..on
    };
    assert!(!zero_age.auto_purge_active(), "zero days is refused");
}
