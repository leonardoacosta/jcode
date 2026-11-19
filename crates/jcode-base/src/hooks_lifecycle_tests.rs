use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn entry(command: &str) -> HookEntry {
    HookEntry {
        command: command.to_string(),
        ..HookEntry::default()
    }
}

fn config(event: LifecycleEvent, entries: Vec<HookEntry>) -> HooksConfig {
    let mut hooks = HooksConfig::default();
    match event {
        LifecycleEvent::SessionStart => hooks.lifecycle_session_start = entries,
        LifecycleEvent::PreToolUse => hooks.lifecycle_pre_tool_use = entries,
        LifecycleEvent::PostToolUse => hooks.lifecycle_post_tool_use = entries,
        LifecycleEvent::SubagentStop => hooks.lifecycle_subagent_stop = entries,
        LifecycleEvent::Stop => hooks.lifecycle_stop = entries,
        LifecycleEvent::PreCompact => hooks.lifecycle_pre_compact = entries,
    }
    hooks
}

#[cfg(unix)]
fn script(dir: &std::path::Path, name: &str, body: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_string_lossy().into_owned()
}

fn resolved(event: LifecycleEvent, entry: HookEntry) -> ResolvedHook {
    resolve_entries(&config(event, vec![entry]), event)
        .0
        .pop()
        .expect("entry resolves")
}

#[test]
fn default_modes_follow_event_semantics() {
    use LifecycleEvent::*;
    assert_eq!(PreToolUse.default_mode(), HookMode::Blocking);
    assert_eq!(Stop.default_mode(), HookMode::Blocking);
    assert_eq!(SessionStart.default_mode(), HookMode::Observe);
    assert_eq!(PreCompact.default_mode(), HookMode::Observe);
    assert_eq!(PostToolUse.default_mode(), HookMode::Async);
    assert_eq!(SubagentStop.default_mode(), HookMode::Async);
}

#[test]
fn blocking_mode_on_a_non_blocking_event_degrades_with_a_warning() {
    let hooks = config(
        LifecycleEvent::PostToolUse,
        vec![HookEntry {
            mode: Some("blocking".into()),
            ..entry("true")
        }],
    );
    let (resolved, warnings) = resolve_entries(&hooks, LifecycleEvent::PostToolUse);
    assert_eq!(resolved[0].mode, HookMode::Observe);
    assert!(
        warnings[0].contains("not supported for PostToolUse"),
        "{warnings:?}"
    );
}

#[test]
fn bad_entries_are_disabled_individually() {
    let hooks = config(
        LifecycleEvent::PreToolUse,
        vec![
            HookEntry {
                matcher: Some("(".into()),
                ..entry("true")
            },
            HookEntry {
                mode: Some("sometimes".into()),
                ..entry("true")
            },
            entry("  "),
            HookEntry {
                path: Some("src/[".into()),
                ..entry("true")
            },
            entry("true"),
        ],
    );
    let (resolved, warnings) = resolve_entries(&hooks, LifecycleEvent::PreToolUse);
    assert_eq!(resolved.len(), 1);
    assert_eq!(warnings.len(), 4, "{warnings:?}");
    assert!(warnings.iter().all(|w| w.contains("disabled")));
    assert_eq!(lifecycle_config_warnings(&hooks).len(), 4);
}

#[test]
fn matcher_is_whole_string_and_case_insensitive() {
    let hook = resolved(
        LifecycleEvent::PreToolUse,
        HookEntry {
            matcher: Some("bash|write".into()),
            ..entry("true")
        },
    );
    assert!(hook.applies(Some("Bash"), &[], None));
    assert!(hook.applies(Some("write"), &[], None));
    assert!(!hook.applies(Some("bash_extra"), &[], None));
    assert!(!hook.applies(Some("read"), &[], None));
    let star = resolved(
        LifecycleEvent::PreToolUse,
        HookEntry {
            matcher: Some("*".into()),
            ..entry("true")
        },
    );
    assert!(star.applies(Some("anything"), &[], None));
}

#[test]
fn path_glob_matches_absolute_and_cwd_relative_paths() {
    let hook = resolved(
        LifecycleEvent::PreToolUse,
        HookEntry {
            path: Some("src/**/*.rs".into()),
            ..entry("true")
        },
    );
    let cwd = Some("/work/repo");
    assert!(hook.applies(Some("edit"), &["/work/repo/src/a/b.rs".into()], cwd));
    assert!(hook.applies(Some("edit"), &["src/main.rs".into()], cwd));
    assert!(!hook.applies(Some("edit"), &["/work/repo/docs/a.md".into()], cwd));
    assert!(!hook.applies(Some("edit"), &["/work/repo/src/a.txt".into()], cwd));
    // A path rule never matches a call that names no paths.
    assert!(!hook.applies(Some("bash"), &[], cwd));
}

#[test]
fn legacy_command_also_configured_as_lifecycle_runs_once() {
    let mut hooks = config(LifecycleEvent::PreToolUse, vec![entry("guard.sh")]);
    let mut legacy = vec!["guard.sh".to_string(), "other.sh".to_string()];
    drop_aliased_legacy(&hooks, "pre_tool", &mut legacy);
    assert_eq!(legacy, vec!["other.sh".to_string()]);
    hooks.lifecycle_pre_tool_use.clear();
    let mut legacy = vec!["guard.sh".to_string()];
    drop_aliased_legacy(&hooks, "pre_tool", &mut legacy);
    assert_eq!(legacy, vec!["guard.sh".to_string()]);
}

#[test]
fn payload_is_claude_code_compatible() {
    let payload = LifecycleCall::new(LifecycleEvent::PreToolUse, "ses_1")
        .cwd(Some("/w".into()))
        .transcript_path(Some("/t.json".into()))
        .field("tool_name", "bash")
        .payload();
    assert_eq!(payload["session_id"], "ses_1");
    assert_eq!(payload["transcript_path"], "/t.json");
    assert_eq!(payload["cwd"], "/w");
    assert_eq!(payload["hook_event_name"], "PreToolUse");
    assert_eq!(payload["tool_name"], "bash");
}

#[cfg(unix)]
#[tokio::test]
async fn pre_tool_use_exit_two_blocks_with_stderr_reason() {
    let temp = tempfile::TempDir::new().unwrap();
    let cmd = script(
        temp.path(),
        "b.sh",
        "#!/bin/sh\ncat >/dev/null\necho nope >&2\nexit 2\n",
    );
    let hooks = config(LifecycleEvent::PreToolUse, vec![entry(&cmd)]);
    let call = LifecycleCall::new(LifecycleEvent::PreToolUse, "s").subject("bash");
    let outcome = run_lifecycle_with(&hooks, call).await;
    assert_eq!(outcome.block.as_deref(), Some("nope"));
}

#[cfg(unix)]
#[tokio::test]
async fn pre_tool_use_json_rewrite_chains_and_deny_drops_it() {
    let temp = tempfile::TempDir::new().unwrap();
    let one = script(
        temp.path(),
        "one.sh",
        "#!/bin/sh\ncat >/dev/null\necho '{\"hookSpecificOutput\":{\"updatedInput\":{\"n\":1}}}'\n",
    );
    let two = script(
        temp.path(),
        "two.sh",
        "#!/bin/sh\ngrep -q '\"n\":1' && echo '{\"hookSpecificOutput\":{\"updatedInput\":{\"n\":2}}}'\n",
    );
    let hooks = config(LifecycleEvent::PreToolUse, vec![entry(&one), entry(&two)]);
    let call = || {
        LifecycleCall::new(LifecycleEvent::PreToolUse, "s")
            .field("tool_input", serde_json::json!({"n": 0}))
    };
    let outcome = run_lifecycle_with(&hooks, call()).await;
    assert_eq!(outcome.updated_input, Some(serde_json::json!({"n": 2})));

    let deny = script(temp.path(), "deny.sh", "#!/bin/sh\necho no >&2\nexit 2\n");
    let hooks = config(LifecycleEvent::PreToolUse, vec![entry(&one), entry(&deny)]);
    let outcome = run_lifecycle_with(&hooks, call()).await;
    assert!(outcome.block.is_some());
    assert_eq!(outcome.updated_input, None);
}

#[cfg(unix)]
#[tokio::test]
async fn timeout_fails_open_unless_fail_closed() {
    let temp = tempfile::TempDir::new().unwrap();
    let slow = script(temp.path(), "slow.sh", "#!/bin/sh\nsleep 30\n");
    let started = std::time::Instant::now();
    let quick = |fail_closed| HookEntry {
        timeout: 1,
        fail_closed,
        ..entry(&slow)
    };
    let open = config(LifecycleEvent::PreToolUse, vec![quick(false)]);
    let outcome =
        run_lifecycle_with(&open, LifecycleCall::new(LifecycleEvent::PreToolUse, "s")).await;
    assert!(outcome.block.is_none());
    assert!(started.elapsed() < Duration::from_secs(10));

    let closed = config(LifecycleEvent::PreToolUse, vec![quick(true)]);
    let outcome =
        run_lifecycle_with(&closed, LifecycleCall::new(LifecycleEvent::PreToolUse, "s")).await;
    assert!(outcome.block.as_deref().unwrap().contains("fail_closed"));
}

#[cfg(unix)]
#[tokio::test]
async fn timeout_kills_the_hook_process_group() {
    let temp = tempfile::TempDir::new().unwrap();
    let marker = temp.path().join("survived");
    let body = format!(
        "#!/bin/sh\n(sleep 3; touch {}) &\nsleep 30\n",
        marker.display()
    );
    let cmd = script(temp.path(), "g.sh", &body);
    let hooks = config(
        LifecycleEvent::PreToolUse,
        vec![HookEntry {
            timeout: 1,
            ..entry(&cmd)
        }],
    );
    run_lifecycle_with(&hooks, LifecycleCall::new(LifecycleEvent::PreToolUse, "s")).await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    assert!(!marker.exists(), "background child outlived the timeout");
}

#[cfg(unix)]
#[tokio::test]
async fn observe_hooks_ignore_exit_two() {
    let temp = tempfile::TempDir::new().unwrap();
    let cmd = script(temp.path(), "o.sh", "#!/bin/sh\necho x >&2\nexit 2\n");
    let hooks = config(LifecycleEvent::SessionStart, vec![entry(&cmd)]);
    let outcome = run_lifecycle_with(
        &hooks,
        LifecycleCall::new(LifecycleEvent::SessionStart, "s"),
    )
    .await;
    assert!(outcome.block.is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn stop_hook_json_block_returns_reason_and_continue_false_halts() {
    let temp = tempfile::TempDir::new().unwrap();
    let block = script(
        temp.path(),
        "sb.sh",
        "#!/bin/sh\ncat >/dev/null\necho '{\"decision\":\"block\",\"reason\":\"run tests\"}'\n",
    );
    let halt = script(
        temp.path(),
        "sh.sh",
        "#!/bin/sh\necho '{\"continue\":false,\"stopReason\":\"enough\"}'\n",
    );
    let outcome = run_lifecycle_with(
        &config(LifecycleEvent::Stop, vec![entry(&block)]),
        LifecycleCall::new(LifecycleEvent::Stop, "s"),
    )
    .await;
    assert_eq!(outcome.block.as_deref(), Some("run tests"));
    let outcome = run_lifecycle_with(
        &config(LifecycleEvent::Stop, vec![entry(&halt)]),
        LifecycleCall::new(LifecycleEvent::Stop, "s"),
    )
    .await;
    assert_eq!(outcome.halt.as_deref(), Some("enough"));
}

struct CountingOwner {
    claimed: Arc<AtomicUsize>,
    fail: bool,
}

struct Lease;
impl ChildLease for Lease {
    fn set_pid(&mut self, _pid: u32) {}
}

impl ChildOwnership for CountingOwner {
    fn own(&self) -> anyhow::Result<Box<dyn ChildLease>> {
        if self.fail {
            anyhow::bail!("generation revoked");
        }
        self.claimed.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Lease))
    }
}

#[cfg(unix)]
#[tokio::test]
async fn revoked_owner_prevents_the_hook_from_running() {
    let temp = tempfile::TempDir::new().unwrap();
    let marker = temp.path().join("ran");
    let cmd = script(
        temp.path(),
        "r.sh",
        &format!("#!/bin/sh\ntouch {}\n", marker.display()),
    );
    let hooks = config(LifecycleEvent::Stop, vec![entry(&cmd)]);
    let claimed = Arc::new(AtomicUsize::new(0));
    let call = |fail| {
        LifecycleCall::new(LifecycleEvent::Stop, "s").owner(Some(Arc::new(CountingOwner {
            claimed: claimed.clone(),
            fail,
        })))
    };
    let outcome = run_lifecycle_with(&hooks, call(true)).await;
    assert!(outcome.admission_error.unwrap().contains("revoked"));
    assert!(!marker.exists());
    let outcome = run_lifecycle_with(&hooks, call(false)).await;
    assert!(outcome.admission_error.is_none());
    assert_eq!(claimed.load(Ordering::SeqCst), 1);
    assert!(marker.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn json_ask_blocks_context_is_collected_and_observe_ignores_decisions() {
    let temp = tempfile::TempDir::new().unwrap();
    let ask = script(
        temp.path(),
        "ask.sh",
        "#!/bin/sh\necho '{\"hookSpecificOutput\":{\"permissionDecision\":\"ask\",\"permissionDecisionReason\":\"confirm\",\"additionalContext\":\"note\"}}'\n",
    );
    let outcome = run_lifecycle_with(
        &config(LifecycleEvent::PreToolUse, vec![entry(&ask)]),
        LifecycleCall::new(LifecycleEvent::PreToolUse, "s"),
    )
    .await;
    assert!(outcome.block.as_deref().unwrap().contains("confirm"));
    assert_eq!(outcome.additional_context, vec!["note".to_string()]);

    let deny = script(
        temp.path(),
        "deny.sh",
        "#!/bin/sh\necho '{\"decision\":\"block\",\"reason\":\"x\"}'\n",
    );
    let outcome = run_lifecycle_with(
        &config(LifecycleEvent::SessionStart, vec![entry(&deny)]),
        LifecycleCall::new(LifecycleEvent::SessionStart, "s").subject("startup"),
    )
    .await;
    assert_eq!(outcome, LifecycleOutcome::default());
}

#[cfg(unix)]
#[tokio::test]
async fn async_hook_does_not_delay_the_caller() {
    let temp = tempfile::TempDir::new().unwrap();
    let slow = script(temp.path(), "slow.sh", "#!/bin/sh\nsleep 30\n");
    let hooks = config(
        LifecycleEvent::SubagentStop,
        vec![HookEntry {
            timeout: 1,
            ..entry(&slow)
        }],
    );
    let started = std::time::Instant::now();
    let outcome = run_lifecycle_with(
        &hooks,
        LifecycleCall::new(LifecycleEvent::SubagentStop, "s"),
    )
    .await;
    assert_eq!(outcome, LifecycleOutcome::default());
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[cfg(unix)]
#[tokio::test]
async fn fail_closed_blocks_on_exit_one_and_bad_json_only_when_set() {
    let temp = tempfile::TempDir::new().unwrap();
    let exit_one = script(temp.path(), "e1.sh", "#!/bin/sh\ncat >/dev/null\nexit 1\n");
    let bad_json = script(
        temp.path(),
        "bj.sh",
        "#!/bin/sh\ncat >/dev/null\necho '{not json'\n",
    );
    let run = |cmd: &str, fail_closed| {
        let hooks = config(
            LifecycleEvent::PreToolUse,
            vec![HookEntry {
                fail_closed,
                ..entry(cmd)
            }],
        );
        async move {
            run_lifecycle_with(&hooks, LifecycleCall::new(LifecycleEvent::PreToolUse, "s")).await
        }
    };
    // Exit 1 fails: closed blocks, open continues.
    let closed = run(&exit_one, true).await;
    assert!(closed.block.as_deref().unwrap().contains("fail_closed"));
    assert!(run(&exit_one, false).await.block.is_none());
    // Unparseable JSON is "no decision" from a clean exit: never a block.
    assert!(run(&bad_json, true).await.block.is_none());
    assert!(run(&bad_json, false).await.block.is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn failed_stop_hook_never_becomes_a_continuation() {
    let temp = tempfile::TempDir::new().unwrap();
    let crash = script(temp.path(), "c.sh", "#!/bin/sh\ncat >/dev/null\nexit 1\n");
    let slow = script(temp.path(), "s.sh", "#!/bin/sh\nsleep 30\n");
    for (cmd, timeout) in [(&crash, 10), (&slow, 1)] {
        let hooks = config(
            LifecycleEvent::Stop,
            vec![HookEntry {
                fail_closed: true,
                timeout,
                ..entry(cmd)
            }],
        );
        let outcome =
            run_lifecycle_with(&hooks, LifecycleCall::new(LifecycleEvent::Stop, "s")).await;
        assert!(outcome.block.is_none(), "{outcome:?}");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn recursion_guard_is_set_on_children_and_suppresses_dispatch() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::TempDir::new().unwrap();
    let seen = temp.path().join("seen");
    let cmd = script(
        temp.path(),
        "g.sh",
        &format!(
            "#!/bin/sh\ncat >/dev/null\nprintf '%s' \"$JCODE_HOOKS_DISABLED\" > {}\n",
            seen.display()
        ),
    );
    let hooks = config(LifecycleEvent::SessionStart, vec![entry(&cmd)]);
    run_lifecycle_with(
        &hooks,
        LifecycleCall::new(LifecycleEvent::SessionStart, "s"),
    )
    .await;
    assert_eq!(std::fs::read_to_string(&seen).unwrap(), "1");

    // A process that already carries the guard never dispatches hooks.
    std::fs::remove_file(&seen).unwrap();
    let previous = std::env::var_os("JCODE_HOOKS_DISABLED");
    crate::env::set_var("JCODE_HOOKS_DISABLED", "1");
    assert!(!lifecycle_configured(LifecycleEvent::SessionStart));
    let outcome =
        run_lifecycle(LifecycleCall::new(LifecycleEvent::PreToolUse, "s").subject("bash")).await;
    dispatch_lifecycle_detached(LifecycleCall::new(LifecycleEvent::SessionStart, "s"));
    match previous {
        Some(value) => crate::env::set_var("JCODE_HOOKS_DISABLED", value),
        None => crate::env::remove_var("JCODE_HOOKS_DISABLED"),
    }
    assert_eq!(outcome, LifecycleOutcome::default());
    assert!(!seen.exists());
}
