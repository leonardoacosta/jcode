use super::*;

#[tokio::test]
async fn session_env_foreground_sessions_are_isolated() {
    use crate::session_env::{remove_session_env, set_session_env};
    for (sid, value) in [("env-bash-a", "proxy-a"), ("env-bash-b", "proxy-b")] {
        set_session_env(
            sid,
            std::collections::HashMap::from([("HTTPS_PROXY".into(), value.into())]),
        )
        .unwrap();
    }
    let mut a = make_ctx(None);
    a.session_id = "env-bash-a".into();
    let mut b = make_ctx(None);
    b.session_id = "env-bash-b".into();
    let tool = BashTool::new();
    let command = json!({"command": "printf '%s:%s' \"$HTTPS_PROXY\" \"$JCODE_SESSION_ID\""});
    let (a, b) = tokio::join!(tool.execute(command.clone(), a), tool.execute(command, b));
    let a = a.unwrap().output;
    let b = b.unwrap().output;
    assert!(a.contains("proxy-a:env-bash-a"), "{a}");
    assert!(!a.contains("proxy-b"));
    assert!(b.contains("proxy-b:env-bash-b"), "{b}");
    assert!(!b.contains("proxy-a"));
    remove_session_env("env-bash-a");
    remove_session_env("env-bash-b");
}

#[cfg(unix)]
#[tokio::test]
async fn session_env_reload_persistable_command_receives_overrides() {
    let mut ctx = make_agent_ctx(jcode_agent_runtime::InterruptSignal::new());
    ctx.session_id = "env-bash-detached".into();
    crate::session_env::set_session_env(
        &ctx.session_id,
        std::collections::HashMap::from([("HTTPS_PROXY".into(), "proxy-detached".into())]),
    )
    .unwrap();
    let result = BashTool::new()
        .execute(
            json!({"command": "printf '%s:%s' \"$HTTPS_PROXY\" \"$JCODE_SESSION_ID\""}),
            ctx,
        )
        .await
        .unwrap();
    assert!(
        result.output.contains("proxy-detached:env-bash-detached"),
        "{}",
        result.output
    );
    crate::session_env::remove_session_env("env-bash-detached");
}

#[tokio::test]
async fn session_env_updates_affect_only_new_launches_and_invalid_updates_keep_bash_working() {
    use crate::session_env::{remove_session_env, set_session_env};
    let sid = "env-bash-updates";
    let mut ctx = make_ctx(None);
    ctx.session_id = sid.into();
    let tool = BashTool::new();
    let input = json!({"command": "printf '%s' \"${SESSION_ENV_REGRESSION_PROBE-unset}\""});
    for value in ["first", "second"] {
        set_session_env(
            sid,
            std::collections::HashMap::from([(
                "SESSION_ENV_REGRESSION_PROBE".into(),
                value.into(),
            )]),
        )
        .unwrap();
        assert!(
            set_session_env(
                sid,
                std::collections::HashMap::from([(
                    "SESSION_ENV_REGRESSION_PROBE".into(),
                    "invalid\0value".into()
                ),])
            )
            .is_err()
        );
        let result = tool.execute(input.clone(), ctx.clone()).await.unwrap();
        assert!(result.output.contains(value), "{}", result.output);
    }
    set_session_env(sid, std::collections::HashMap::new()).unwrap();
    let result = tool.execute(input, ctx).await.unwrap();
    assert!(result.output.contains("unset"), "{}", result.output);
    remove_session_env(sid);
}

#[tokio::test]
async fn session_env_background_snapshots_before_deferred_launch() {
    use crate::session_env::{remove_session_env, set_session_env};
    for clear in [false, true] {
        let sid = format!("env-bash-race-{clear}");
        let original =
            std::collections::HashMap::from([("HTTPS_PROXY".into(), "proxy-original".into())]);
        set_session_env(&sid, original).unwrap();
        let (reached_tx, reached_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        SESSION_ENV_LAUNCH_GATES
            .lock()
            .unwrap()
            .insert(sid.clone(), (reached_tx, release_rx));
        let mut ctx = make_ctx(None);
        ctx.session_id = sid.clone();
        let result = BashTool::new()
            .execute(
                json!({
                    "command": "printf '%s:%s\\n' \"$HTTPS_PROXY\" \"$JCODE_SESSION_ID\"",
                    "run_in_background": true, "notify": false,
                }),
                ctx,
            )
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(10), reached_rx)
            .await
            .unwrap()
            .unwrap();
        if clear {
            set_session_env(&sid, std::collections::HashMap::new()).unwrap();
        } else {
            set_session_env(
                &sid,
                std::collections::HashMap::from([("HTTPS_PROXY".into(), "proxy-replaced".into())]),
            )
            .unwrap();
        }
        // Teardown must not erase the snapshot of this already-accepted job.
        remove_session_env(&sid);
        release_tx.send(()).unwrap();
        let metadata = result.metadata.unwrap();
        let task_id = metadata["task_id"].as_str().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                let status = crate::background::global().status(task_id).await.unwrap();
                if status.status != BackgroundTaskStatus::Running {
                    assert_eq!(status.status, BackgroundTaskStatus::Completed);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let output = crate::background::global().output(task_id).await.unwrap();
        assert!(
            output.contains(&format!("proxy-original:{sid}")),
            "{output}"
        );
        assert!(!output.contains("proxy-replaced"));
    }
}

/// An active overnight coordinator's bash children get the hardened git/rg
/// environment (the one the policy probes with), whatever the session sets;
/// every other session keeps its own overrides. Covers all three spawn paths.
#[tokio::test]
async fn coordinator_bash_children_get_the_hardened_git_environment() {
    use crate::session_env::{remove_session_env, set_session_env};
    let _home = crate::tool::coordinator_policy::tests::Home::new();
    let coordinator = "env-bash-coordinator";
    crate::tool::coordinator_policy::tests::write_run(
        coordinator,
        "overnight_env_bash_run",
        jcode_overnight_core::OvernightRunStatus::Running,
    );
    crate::tool::coordinator_policy::install_for_overnight(coordinator, "overnight_env_bash_run");
    for sid in [coordinator, "env-bash-plain"] {
        set_session_env(
            sid,
            std::collections::HashMap::from([
                ("GIT_DIR".into(), "/tmp/elsewhere".into()),
                ("GIT_EXTERNAL_DIFF".into(), "/bin/hook".into()),
                ("RIPGREP_CONFIG_PATH".into(), "/tmp/rg.conf".into()),
                ("HTTPS_PROXY".into(), "proxy".into()),
            ]),
        )
        .unwrap();
    }
    let probe = "printf '%s|%s|%s|%s|%s|%s' \"${GIT_DIR-unset}\" \"${GIT_EXTERNAL_DIFF-unset}\" \
                 \"${RIPGREP_CONFIG_PATH-unset}\" \"$GIT_CONFIG_NOSYSTEM\" \"$GIT_CONFIG_KEY_0\" \"$HTTPS_PROXY\"";
    let run = |sid: &'static str, extra: serde_json::Value| async move {
        let mut ctx = make_ctx(None);
        ctx.session_id = sid.into();
        let mut input = json!({ "command": probe });
        input
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        BashTool::new().execute(input, ctx).await.unwrap()
    };
    let hardened = "unset|unset|unset|1|core.fsmonitor|proxy";
    let result = run(coordinator, json!({})).await;
    assert_eq!(result.output.trim(), hardened, "foreground");
    let result = run("env-bash-plain", json!({})).await;
    assert_eq!(
        result.output.trim(),
        "/tmp/elsewhere|/bin/hook|/tmp/rg.conf|||proxy",
        "a non-coordinator session keeps its overrides"
    );

    // Background path.
    let result = run(
        coordinator,
        json!({"run_in_background": true, "notify": false}),
    )
    .await;
    let task_id = result.metadata.unwrap()["task_id"]
        .as_str()
        .unwrap()
        .to_string();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let status = crate::background::global().status(&task_id).await.unwrap();
            if status.status != BackgroundTaskStatus::Running {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let output = crate::background::global().output(&task_id).await.unwrap();
    assert!(output.contains(hardened), "background: {output}");

    // Detached (reload-persistable) path.
    #[cfg(unix)]
    {
        let mut ctx = make_agent_ctx(jcode_agent_runtime::InterruptSignal::new());
        ctx.session_id = coordinator.into();
        let result = BashTool::new()
            .execute(json!({ "command": probe }), ctx)
            .await
            .unwrap();
        assert!(
            result.output.contains(hardened),
            "detached: {}",
            result.output
        );
    }
    remove_session_env(coordinator);
    remove_session_env("env-bash-plain");
    crate::tool::coordinator_policy::remove(coordinator);
}
