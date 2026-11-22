use super::*;
use serde_json::json;

fn id(p: &Path) -> PathBuf {
    p.to_path_buf()
}

fn ok(tool: &str, input: Value) -> bool {
    allows(tool, &input, &id).is_ok()
}

fn bash(cmd: &str) -> bool {
    ok("bash", json!({ "command": cmd }))
}

#[test]
fn planning_and_inspection_tools_allowed() {
    for t in [
        "swarm",
        "todo",
        "schedule",
        "bg",
        "read",
        "agentgrep",
        "conversation_search",
        "session_search",
        "batch",
        "mcp",
        "mcp__linear__list_issues",
    ] {
        assert!(ok(t, json!({})), "{t}");
    }
}

#[test]
fn mutating_tools_denied() {
    for t in [
        "apply_patch",
        "selfdev",
        "browser",
        "open",
        "memory",
        "subagent",
        "unknown",
    ] {
        assert!(!ok(t, json!({})), "{t}");
    }
}

#[test]
fn edit_and_write_only_for_markdown() {
    for t in ["edit", "write"] {
        assert!(ok(t, json!({"file_path": "notes/plan.md"})));
        assert!(ok(t, json!({"file_path": "NOTES.MD"})));
        assert!(!ok(t, json!({"file_path": "src/main.rs"})));
        assert!(!ok(t, json!({"file_path": "notes.md.rs"})));
        assert!(!ok(t, json!({"file_path": "md"})));
        assert!(!ok(t, json!({})));
    }
}

#[test]
fn markdown_symlink_to_code_denied() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("main.rs");
    std::fs::write(&target, "fn main() {}").unwrap();
    let link = dir.path().join("plan.md");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(unix)]
    assert!(!ok("edit", json!({"file_path": link})));
}

#[test]
fn read_only_bash_allowed() {
    for c in [
        "git status",
        "git diff --stat && rg -n TODO src",
        "git log --oneline -5 | head -3",
        "git -C /tmp show HEAD",
        "git branch",
        "git branch -a",
        "git branch --list 'feat/*'",
        "ls -la; pwd",
        "cat a.rs | wc -l",
        "find . -name '*.rs'",
        "rg 'a > b' src",
        "echo 'a && b'",
        "cat a.rs 2>&1",
        "grep x f 2>/dev/null",
        "jq . < a.json",
        "tail -n 5 f || echo none",
    ] {
        assert!(bash(c), "{c}");
    }
}

#[test]
fn mutating_bash_denied() {
    for c in [
        "cargo build",
        "cargo test",
        "cat a.rs > b.rs",
        "cat a.rs >> b.rs",
        "echo x > f.md",
        "echo x &> f",
        "git push",
        "git commit -m x",
        "git checkout main",
        "git branch -D foo",
        "git branch newbranch",
        "git branch -m a b",
        "git diff --output=x",
        "rm -rf x",
        "sed -i s/a/b/ f",
        "find . -exec rm {} ;",
        "find . -delete",
        "ls $(rm x)",
        "ls `rm x`",
        "sleep 10 &",
        "ls & rm x",
        "ls && cargo build",
        "ls | xargs rm",
        "cat <<EOF",
        "cat <(ls)",
        "rg --pre ./x foo",
        "",
        "echo 'unterminated",
        "cat a 2>b",
        "FOO=1 ls",
        "env ls",
    ] {
        assert!(!bash(c), "{c}");
    }
    assert!(!ok("bash", json!({})));
}

/// Argument-injection and write/execute vectors found in review. Each must be
/// denied; none may rely on a later parser stage to notice the injected word.
#[test]
fn bash_expansion_and_argv_injection_denied() {
    for c in [
        // Expansion can smuggle a find predicate or git option past the checks.
        r#"find . "$F""#,
        "find . $F",
        "find . ${F}",
        "find . `echo -delete`",
        "find . $(echo -delete)",
        "find . $((1+1))",
        "find . $'-delete'",
        r#"find . "${F}""#,
        r#"find . "$(echo -delete)""#,
        r#"find . "`echo -delete`""#,
        "sort -o$X f",
        "git -c $X log",
        "git log $X",
        "echo $HOME",
        r#"echo "$HOME""#,
        r#"echo "a$HOME""#,
        // Brace expansion builds `-delete` from unquoted parts.
        "find . {-delete,x}",
        "find . -{d,}elete",
        "ls {a,b}",
        // An unquoted glob can expand to a file named like an option.
        "find . *",
        "find * -name x",
        "rg foo *.rs",
        "rg foo ?",
        "git log [a-z]*",
        "git diff *",
        // Process substitution, both directions.
        "cat <(ls)",
        "cat >(tee x)",
        // Commands that write or execute are not on the allowlist at all.
        "sort -o out f",
        "sort -ofile f",
        "less file",
        "man ls",
        "env ls",
        "xargs rm",
        "awk 'BEGIN{system(\"rm x\")}'",
        "awk '{print > \"f\"}' x",
        "sed -i s/a/b/ f",
        "sed 'w out' f",
        "tee out",
        "dd of=out",
        "python3 -c 1",
        "bash -c ls",
        "sh -c ls",
        "/bin/rm x",
        "\\rm x",
        // find: every writing or executing predicate.
        "find . -fprint out",
        "find . -fprint0 out",
        "find . -fprintf out %p",
        "find . -fls out",
        "find . -exec rm {} +",
        "find . -execdir rm {} ;",
        "find . -ok rm {} ;",
        "find . -okdir rm {} ;",
        "find . '-delete'",
        r#"find . "-delete""#,
        "find . \\-delete",
        // git: output files and external programs.
        "git log --output=out",
        "git log --output out",
        "git show --output=out",
        "git diff --output=out",
        "git diff --ext-diff",
        "git diff --textconv",
        "git -c core.pager=x log",
        "git -c alias.x='!rm' x",
        "git --exec-path=/tmp log",
        "git config --global x y",
        "git fetch",
        // ripgrep runs external programs.
        "rg --pre ./x foo",
        "rg --pre=./x foo",
        "rg --pre-glob '*' foo",
        "rg --hostname-bin=./x foo",
        "rg --hostname-bin ./x foo",
    ] {
        assert!(!bash(c), "must be denied: {c}");
    }
}

/// The hardening must not break ordinary inspection commands.
#[test]
fn bash_literal_dollar_in_single_quotes_and_regex_anchors_still_allowed() {
    for c in [
        "rg 'foo$' src",
        "rg '\\$HOME' src",
        "rg 'a|b' src",
        "echo '${F}'",
        "echo '$(not run)'",
        r#"rg "end$" src"#,
        r#"echo "price: $ 5""#,
        r#"echo "a\$HOME""#,
        "jq '.a | {b}' f.json",
        "find . -name '*.rs' -not -path './target/*'",
        "git log --format='%h %s' -3",
        "git log --pretty=format:'%H'",
        "ls *.rs",
        "cat src/*.rs | wc -l",
        "find . -name '*.rs'",
        "rg 'a*' src",
        "rg \"a?\" src",
        "git log -- 'src/*'",
        "ls ~",
        "ls ~/dev",
    ] {
        assert!(bash(c), "must stay allowed: {c}");
    }
}

#[test]
fn denial_survives_an_unwritable_event_log() {
    use jcode_overnight_core::OvernightRunStatus::Running;
    let _home = Home::new();
    let session = format!("coord-{}", uuid::Uuid::new_v4());
    let run = format!("run-{}", uuid::Uuid::new_v4());
    write_run(&session, &run, Running);
    install_for_overnight(&session, &run);
    // A directory where events.jsonl should be makes every append fail, even
    // for a privileged user.
    let run_dir = crate::overnight::run_dir(&run).unwrap();
    let events = run_dir.join("events.jsonl");
    let _ = std::fs::remove_file(&events);
    std::fs::create_dir_all(&events).unwrap();
    let manifest = crate::overnight::load_manifest(&run).unwrap();
    assert!(
        crate::overnight::record_event(&manifest, "x", String::new(), json!({}), false).is_err(),
        "the event log really is unwritable"
    );

    let err = enforce(
        &session,
        "edit",
        &json!({"file_path": "src/main.rs"}),
        false,
        &id,
    )
    .unwrap_err();
    assert!(
        err.contains("denied for the overnight coordinator"),
        "{err}"
    );
    assert!(err.contains("swarm worker"), "{err}");
    // Still restricted afterwards.
    assert!(
        enforce(
            &session,
            "bash",
            &json!({"command": "cargo build"}),
            false,
            &id
        )
        .is_err()
    );
}

pub(in crate::tool) struct Home {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev: Option<std::ffi::OsString>,
    _dir: tempfile::TempDir,
}

impl Home {
    pub(in crate::tool) fn new() -> Self {
        let guard = crate::storage::lock_test_env();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", dir.path());
        Self {
            _guard: guard,
            prev,
            _dir: dir,
        }
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(v) => crate::env::set_var("JCODE_HOME", v),
            None => crate::env::remove_var("JCODE_HOME"),
        }
    }
}

pub(in crate::tool) fn write_run(
    session: &str,
    run_id: &str,
    status: jcode_overnight_core::OvernightRunStatus,
) {
    let run_dir = crate::overnight::run_dir(run_id).unwrap();
    let now = chrono::Utc::now();
    let mut manifest: jcode_overnight_core::OvernightManifest = serde_json::from_value(json!({
        "version": 1,
        "run_id": run_id,
        "parent_session_id": "parent",
        "coordinator_session_id": session,
        "coordinator_session_name": "coordinator",
        "started_at": now,
        "target_wake_at": now,
        "handoff_ready_at": now,
        "post_wake_grace_until": now,
        "status": "running",
        "provider_name": "test-provider",
        "model": "test-model",
        "max_agents_guidance": 2,
        "process_id": 1,
        "run_dir": run_dir,
        "events_path": run_dir.join("events.jsonl"),
        "human_log_path": run_dir.join("run.log"),
        "review_path": run_dir.join("review.html"),
        "review_notes_path": run_dir.join("review-notes.md"),
        "preflight_path": run_dir.join("preflight.json"),
        "task_cards_dir": run_dir.join("task-cards"),
        "issue_drafts_dir": run_dir.join("issue-drafts"),
        "validation_dir": run_dir.join("validation"),
        "last_activity_at": now,
    }))
    .unwrap();
    manifest.status = status;
    crate::overnight::save_manifest(&manifest).unwrap();
}

#[test]
fn install_denies_then_terminal_run_restores() {
    use jcode_overnight_core::OvernightRunStatus::*;
    let _home = Home::new();
    let session = format!("coord-{}", uuid::Uuid::new_v4());
    let run = format!("run-{}", uuid::Uuid::new_v4());
    assert!(!is_coordinator(&session));
    write_run(&session, &run, Running);
    install_for_overnight(&session, &run);
    assert!(is_coordinator(&session));
    let err = enforce(
        &session,
        "edit",
        &json!({"file_path": "src/main.rs"}),
        false,
        &id,
    )
    .unwrap_err();
    assert!(err.contains("swarm worker"), "{err}");
    assert!(enforce(&session, "edit", &json!({"file_path": "a.md"}), false, &id).is_ok());
    assert!(enforce(&session, "mcp__x__y", &json!({}), true, &id).is_ok());
    assert!(
        enforce(
            &session,
            "bash",
            &json!({"command": "cargo build"}),
            false,
            &id
        )
        .is_err()
    );

    // Event recorded.
    let manifest = crate::overnight::load_manifest(&run).unwrap();
    let events = crate::overnight::read_events(&manifest).unwrap();
    let denied: Vec<_> = events
        .iter()
        .filter(|e| e.kind == "coordinator_tool_denied")
        .collect();
    assert_eq!(denied.len(), 2);
    assert_eq!(denied[0].details["tool"], "edit");
    assert!(
        denied[0].details["reason"]
            .as_str()
            .unwrap()
            .contains("*.md")
    );

    write_run(&session, &run, Completed);
    assert!(
        enforce(
            &session,
            "edit",
            &json!({"file_path": "src/main.rs"}),
            false,
            &id
        )
        .is_ok()
    );
    assert!(!is_coordinator(&session));
}

#[test]
fn restored_session_found_through_manifest() {
    use jcode_overnight_core::OvernightRunStatus::*;
    let _home = Home::new();
    let session = format!("coord-{}", uuid::Uuid::new_v4());
    let run = format!("run-{}", uuid::Uuid::new_v4());
    write_run(&session, &run, CancelRequested);
    assert!(is_coordinator(&session));
    assert!(enforce(&session, "write", &json!({"file_path": "x.rs"}), false, &id).is_err());
    remove(&session);
    write_run(&session, &run, Cancelled);
    // Cached negative result is only set after a miss; terminal manifest means no restriction.
    assert!(!is_coordinator(&session));
}

#[test]
fn unrelated_session_unrestricted() {
    let _home = Home::new();
    let session = format!("plain-{}", uuid::Uuid::new_v4());
    assert!(
        enforce(
            &session,
            "bash",
            &json!({"command": "cargo build"}),
            false,
            &id
        )
        .is_ok()
    );
}

#[test]
fn run_bookkeeping_allowed_but_escapes_denied() {
    use jcode_overnight_core::OvernightRunStatus::Running;
    let _home = Home::new();
    let session = format!("coord-{}", uuid::Uuid::new_v4());
    let run = format!("run-{}", uuid::Uuid::new_v4());
    write_run(&session, &run, Running);
    install_for_overnight(&session, &run);
    let run_dir = crate::overnight::run_dir(&run).unwrap();
    std::fs::create_dir_all(run_dir.join("task-cards")).unwrap();
    let call =
        |tool: &str, path: &Path| enforce(&session, tool, &json!({"file_path": path}), false, &id);

    for tool in ["edit", "write"] {
        // Existing and not-yet-created files inside the run dir.
        assert!(call(tool, &run_dir.join("task-cards/x.json")).is_ok());
        assert!(call(tool, &run_dir.join("validation/new/out.txt")).is_ok());
        // Escapes and unrelated paths.
        assert!(call(tool, &run_dir.join("../escape.rs")).is_err());
        assert!(call(tool, &run_dir.join("task-cards/../../escape.json")).is_err());
        assert!(call(tool, &run_dir.parent().unwrap().join("other-run/x.json")).is_err());
        assert!(call(tool, Path::new("/tmp/outside.json")).is_err());
        // Markdown stays allowed anywhere.
        assert!(call(tool, Path::new("/tmp/outside.md")).is_ok());
    }
    // Bookkeeping exemption is edit/write only.
    assert!(
        enforce(
            &session,
            "apply_patch",
            &json!({"file_path": run_dir.join("a.json")}),
            false,
            &id
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn symlink_out_of_run_dir_denied() {
    use jcode_overnight_core::OvernightRunStatus::Running;
    let _home = Home::new();
    let session = format!("coord-{}", uuid::Uuid::new_v4());
    let run = format!("run-{}", uuid::Uuid::new_v4());
    write_run(&session, &run, Running);
    install_for_overnight(&session, &run);
    let run_dir = crate::overnight::run_dir(&run).unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(outside.path(), run_dir.join("link")).unwrap();
    let path = run_dir.join("link/x.json");
    assert!(enforce(&session, "write", &json!({"file_path": path}), false, &id).is_err());
}
