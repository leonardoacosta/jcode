//! Process-level regressions for clap's CLI exit status and output streams.
use std::process::{Command, Output};

fn invoke(args: &[&str]) -> Output {
    let root = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jcode"))
        .current_dir(root.path())
        .env_clear()
        .env("HOME", root.path())
        .env("JCODE_HOME", root.path().join("jcode"))
        .env("JCODE_NO_TELEMETRY", "1")
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .expect("run jcode CLI");
    assert!(
        !root.path().join("jcode").exists(),
        "parsing must exit before creating application state"
    );
    output
}

#[test]
fn policy_cli_is_read_only_and_defaults_fail_closed() {
    let output = invoke(&["policy"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["tasks"]["unattended"], false);
    assert_eq!(report["worktrees"]["allow_creation"], false);
    assert_eq!(report["sources"]["tasks.unattended"], "default");
}

#[test]
fn policy_cli_diagnostic_file_failure_is_read_only() {
    let output = invoke(&["policy", "--admission", "missing-snapshot.json"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn policy_cli_preserves_fixture_config_and_diagnostic_snapshot() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("jcode");
    std::fs::create_dir(&home).unwrap();
    let config = "# preserve comments\n[tasks]\nunattended = true\nmax_attempts = 2\n";
    std::fs::write(home.join("config.toml"), config).unwrap();
    let snapshot = serde_json::json!({
        "scope": "bounded", "authorization": {"scope": "bounded", "max_attempts": 1,
        "max_runtime_seconds": 100, "max_spend_microusd": 100},
        "dependencies_ready": true, "workspace_ready": true, "ownership_verified": true
    })
    .to_string();
    let input = root.path().join("snapshot.json");
    std::fs::write(&input, &snapshot).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jcode"))
        .current_dir(root.path())
        .env_clear()
        .env("HOME", root.path())
        .env("JCODE_HOME", &home)
        .env("JCODE_NO_TELEMETRY", "1")
        .args(["policy", "--admission"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["diagnostic_only"], true);
    assert_eq!(report["diagnostic_admission"]["state"], "ready");
    assert_eq!(report["diagnostic_admission"]["remaining_attempts"], 1);
    assert_eq!(
        report["sources"]["tasks.unattended"],
        home.join("config.toml").display().to_string()
    );
    assert_eq!(
        std::fs::read_to_string(home.join("config.toml")).unwrap(),
        config
    );
    assert_eq!(std::fs::read_to_string(input).unwrap(), snapshot);
    assert_eq!(
        std::fs::read_dir(&home).unwrap().count(),
        1,
        "no migration/log/session files"
    );
}

#[test]
fn unknown_subcommand_exits_with_usage_error() {
    let output = invoke(&["definitely-not-a-command"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unrecognized subcommand 'definitely-not-a-command'"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn unknown_option_exits_with_usage_error() {
    let output = invoke(&["--definitely-not-an-option"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unexpected argument '--definitely-not-an-option'"));
    assert!(stderr.contains("Usage:"));
}

#[test]
fn help_exits_successfully_on_stdout() {
    let output = invoke(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn version_exits_successfully_on_stdout() {
    let output = invoke(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("jcode ")
    );
}
