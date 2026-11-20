use super::*;
use std::ffi::OsString;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?}");
}

/// Disposable JCODE_HOME and an empty global git config, serialized on the
/// shared test-env lock.
struct Env {
    _guard: std::sync::MutexGuard<'static, ()>,
    home: tempfile::TempDir,
    prev: Vec<(&'static str, Option<OsString>)>,
    cwd: Option<PathBuf>,
}

impl Env {
    fn new(rules_toml: &str) -> Self {
        let guard = crate::storage::lock_test_env();
        let home = tempfile::TempDir::new().unwrap();
        let mut prev = Vec::new();
        let mut set = |key: &'static str, value: &Path| {
            prev.push((key, std::env::var_os(key)));
            crate::env::set_var(key, value);
        };
        set("JCODE_HOME", home.path());
        let gitconfig = home.path().join("gitconfig");
        std::fs::write(&gitconfig, "").unwrap();
        set("GIT_CONFIG_GLOBAL", &gitconfig);
        std::fs::write(home.path().join("config.toml"), rules_toml).unwrap();
        crate::config::Config::invalidate_cache();
        Self {
            _guard: guard,
            home,
            prev,
            cwd: std::env::current_dir().ok(),
        }
    }

    fn global_gitconfig(&self) -> PathBuf {
        self.home.path().join("gitconfig")
    }

    /// A repo at `<home>/<name>` with the given origin and no user identity.
    fn repo(&self, name: &str, origin: Option<&str>) -> PathBuf {
        let dir = self.home.path().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        if let Some(origin) = origin {
            git(&dir, &["remote", "add", "origin", origin]);
        }
        dir
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (key, value) in self.prev.drain(..).rev() {
            match value {
                Some(value) => crate::env::set_var(key, value),
                None => crate::env::remove_var(key),
            }
        }
        // A test may chdir into a directory that dies with `home`.
        if let Some(cwd) = self.cwd.take() {
            let _ = std::env::set_current_dir(cwd);
        }
        crate::config::Config::invalidate_cache();
    }
}

const RULES: &str = r#"
[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
tracker = "ado"
branch_template = "feat/<id>-<slug>"
skills = ["triage-ado"]
expected_email = "me@brown.example"
swarm_model = "brown-model"

[[workspaces.rules]]
name = "personal"
origin = "github.com/me/*"
tracker = "linear"

[[workspaces.rules]]
name = "typo"
origin = "github.com/typo/**"
tracker = "jira"
branch_template = "must-not-appear"
"#;

#[test]
fn sanitize_strips_credentials_ports_and_suffixes() {
    for (raw, want) in [
        (
            "https://dev.azure.com/brown/proj/_git/repo",
            "dev.azure.com/brown/proj/_git/repo",
        ),
        (
            "https://user:s3cret@dev.azure.com/brown/p/_git/r.git",
            "dev.azure.com/brown/p/_git/r",
        ),
        ("https://tok@github.com/me/x/", "github.com/me/x"),
        ("ssh://git@host.example:2222/a/b.git", "host.example/a/b"),
        ("git@github.com:me/x.git", "github.com/me/x"),
        ("https://github.com/me/x?token=zzz#frag", "github.com/me/x"),
        // A raw `/` in the password must not leak into host or path.
        ("https://u:pa/ss@host.example/x/y.git", "host.example/x/y"),
        ("ssh://git@[::1]:2222/a/b.git", "[::1]/a/b"),
        ("git@[2001:db8::1]:a/b.git", "[2001:db8::1]/a/b"),
    ] {
        let got = sanitize_origin(raw);
        assert_eq!(got.as_deref(), Some(want), "{raw}");
        let got = got.unwrap();
        for secret in ["s3cret", "tok@", "token=", "zzz", "pa/ss", "ss@"] {
            assert!(!got.contains(secret), "{raw} leaked {secret}");
        }
    }
    for raw in [
        "",
        "/srv/git/repo.git",
        "../relative",
        r"C:\repo\x",
        r"c:\repo",
        "https://host:notaport/x",
        "https://@/x",
        // Password with a raw `?`: cut as a query, leaves a bogus port.
        "https://u:pa?ss@host.example/x",
    ] {
        assert_eq!(sanitize_origin(raw), None, "{raw}");
    }
}

#[test]
fn first_matching_rule_wins_and_globs_respect_segments() {
    let rules: WorkspacesConfig = toml::from_str(
        r#"
[[workspaces.rules]]
name = "narrow"
origin = "github.com/me/private-*"
[[workspaces.rules]]
name = "wide"
origin = "github.com/me/**"
[[workspaces.rules]]
name = ""
origin = "github.com/**"
[[workspaces.rules]]
name = "noorigin"
origin = ""
"#,
    )
    .map(|doc: toml::Table| {
        doc["workspaces"]
            .clone()
            .try_into::<WorkspacesConfig>()
            .unwrap()
    })
    .unwrap();
    let name = |origin: &str| match_rule(&rules, origin).map(|rule| rule.name.clone());
    assert_eq!(name("github.com/me/private-x"), Some("narrow".into()));
    assert_eq!(name("GitHub.com/Me/other/deep"), Some("wide".into()));
    // `*` does not cross `/`; the unnamed and empty-origin rules never match.
    assert_eq!(name("github.com/other/x"), None);
    assert_eq!(name("gitlab.com/me/x"), None);
}

#[test]
fn detects_workspace_from_origin_and_ignores_credentials() {
    let env = Env::new(RULES);
    let repo = env.repo(
        "brown-repo",
        Some("https://svc:hunter2@dev.azure.com/brown/proj/_git/repo"),
    );
    let resolved = resolve(&repo).expect("brown");
    assert_eq!(resolved.name(), "brown");
    // Only the sanitized origin is retained anywhere.
    let kept = format!("{:?}", resolved.facts);
    assert!(!kept.contains("hunter2") && !kept.contains("svc"), "{kept}");

    let sub = repo.join("a/b");
    std::fs::create_dir_all(&sub).unwrap();
    assert_eq!(resolve(&sub).unwrap().name(), "brown");
}

#[test]
fn unknown_without_repo_origin_or_match() {
    let env = Env::new(RULES);
    let no_origin = env.repo("noorigin", None);
    let elsewhere = env.repo("elsewhere", Some("https://example.org/x/y"));
    let plain = env.home.path().join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    for dir in [
        &no_origin,
        &elsewhere,
        &plain,
        &env.home.path().join("missing"),
    ] {
        assert!(resolve(dir).is_none(), "{dir:?}");
        assert_eq!(badge(dir, true), None);
        assert_eq!(prompt_block(dir, None), None);
    }
}

#[test]
fn no_rules_configured_never_runs_git() {
    let env = Env::new("");
    let repo = env.repo("r", Some("https://github.com/me/x"));
    let before = git_spawn_count();
    assert!(resolve(&repo).is_none());
    assert_eq!(git_spawn_count(), before);
}

#[test]
fn identity_hit_mismatch_unset_and_cache_reuse() {
    let env = Env::new(RULES);
    let repo = env.repo(
        "brown-repo",
        Some("https://dev.azure.com/brown/proj/_git/repo"),
    );
    let global = env.global_gitconfig();

    // Unset: reported, never invented.
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Missing);
    assert_eq!(
        badge(&repo, true).as_deref(),
        Some("ws: brown · brown-repo: tracker:ado ?id")
    );

    // A global config edit invalidates the cache.
    std::fs::write(&global, "[user]\n\temail = other@example.com\n").unwrap();
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Mismatch);
    assert_eq!(
        badge(&repo, true).as_deref(),
        Some("ws: brown · brown-repo: tracker:ado !id")
    );
    let prompt = prompt_block(&repo, None).unwrap();
    assert!(prompt.contains("differs from this workspace's expected_email"));

    // Repo-local config wins over the global one, and also invalidates.
    git(&repo, &["config", "user.email", "ME@brown.example"]);
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Match);
    assert_eq!(
        badge(&repo, true).as_deref(),
        Some("ws: brown · brown-repo: tracker:ado")
    );

    // Neither config file changed: no git process at all.
    let before = git_spawn_count();
    for _ in 0..5 {
        assert_eq!(resolve(&repo).unwrap().identity(), Identity::Match);
        let _ = badge(&repo, true);
        let _ = prompt_block(&repo, None);
    }
    assert_eq!(git_spawn_count(), before, "cache hit must not spawn git");

    // The check never wrote to Git config.
    assert_eq!(
        std::fs::read_to_string(&global).unwrap(),
        "[user]\n\temail = other@example.com\n"
    );
}

#[test]
fn rule_without_expected_email_has_no_identity_marker() {
    let env = Env::new(RULES);
    let repo = env.repo("p", Some("git@github.com:me/thing.git"));
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::NotChecked);
    assert_eq!(
        badge(&repo, true).as_deref(),
        Some("ws: personal · p: tracker:linear")
    );
}

#[test]
fn prompt_block_states_tracker_bounded_and_skips_invalid_rule_hints() {
    let env = Env::new(RULES);
    let brown = env.repo("b", Some("https://dev.azure.com/brown/p/_git/r"));
    let block = prompt_block(&brown, None).unwrap();
    assert!(block.starts_with("# Workspace\n"), "{block}");
    assert!(block.contains("ADO (Azure DevOps) is the configured tracker"));
    assert!(block.contains("Branch template: feat/<id>-<slug>"));
    assert!(block.contains("Skills: triage-ado"));
    assert!(block.contains("not authoritative"));

    let typo = env.repo("t", Some("https://github.com/typo/x"));
    let block = prompt_block(&typo, None).unwrap();
    assert!(block.contains("typo"));
    assert!(
        !block.contains("must-not-appear") && !block.contains("tracker"),
        "{block}"
    );
}

#[test]
fn prompt_hint_block_is_bounded() {
    let long = "x".repeat(2000);
    let env = Env::new(&format!(
        "[[workspaces.rules]]\nname = \"w\"\norigin = \"github.com/**\"\ntracker = \"linear\"\nbranch_template = \"{long}\"\nskills = [{}]\n",
        (0..200)
            .map(|i| format!("\"skill-{i}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    let repo = env.repo("r", Some("https://github.com/a/b"));
    let block = prompt_block(&repo, None).unwrap();
    assert!(block.chars().count() < 700, "{}", block.chars().count());
}

#[test]
fn worktree_shares_identity_and_other_repo_does_not() {
    let env = Env::new(RULES);
    let main = env.repo("main", Some("https://dev.azure.com/brown/p/_git/r"));
    git(&main, &["config", "user.email", "me@brown.example"]);
    git(&main, &["config", "user.name", "T"]);
    git(&main, &["commit", "--allow-empty", "-q", "-m", "init"]);
    let wt = env.home.path().join("wt");
    git(&main, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    let personal = env.repo("personal", Some("https://github.com/me/x"));
    let stranger = env.repo("stranger", Some("https://example.org/a/b"));
    let twin = env.repo("twin", Some("https://dev.azure.com/brown/p/_git/other"));

    record_provenance("worker-1", Some(&main));
    let prov = provenance("worker-1").expect("provenance");
    assert_eq!(prov.workspace, "brown");

    let check = |dir: &Path| check_target(&prov, resolve(dir).as_ref());
    assert_eq!(check(&wt), TargetCheck::SameRepository);
    assert_eq!(check(&twin), TargetCheck::SameWorkspace);
    assert_eq!(
        check(&personal),
        TargetCheck::DifferentWorkspace("personal".into())
    );
    assert_eq!(check(&stranger), TargetCheck::Unknown);

    // Worktree resolves by its own facts to the same workspace and identity.
    let in_wt = resolve(&wt).unwrap();
    assert_eq!(in_wt.name(), "brown");
    assert_eq!(in_wt.identity(), Identity::Match);

    // No provenance is recorded for a coordinator outside any workspace.
    record_provenance("worker-2", Some(&stranger));
    assert!(provenance("worker-2").is_none());
}

#[test]
fn worker_prompt_reports_mismatch_without_inheriting_coordinator_rules() {
    let env = Env::new(RULES);
    let brown = env.repo("brown", Some("https://dev.azure.com/brown/p/_git/r"));
    let stranger = env.repo("stranger", Some("https://example.org/a/b"));
    let personal = env.repo("personal", Some("https://github.com/me/x"));
    record_provenance("w", Some(&brown));
    let prov = provenance("w").unwrap();

    let unknown = prompt_block(&stranger, Some(&prov)).unwrap();
    assert!(
        unknown.contains("outside any matched workspace"),
        "{unknown}"
    );
    assert!(
        !unknown.contains("ADO") && !unknown.contains("feat/<id>"),
        "{unknown}"
    );

    let other = prompt_block(&personal, Some(&prov)).unwrap();
    assert!(other.contains("only personal's rules apply"), "{other}");
    assert!(
        other.contains("Linear") && !other.contains("ADO"),
        "{other}"
    );
}

#[test]
fn workspace_override_applies_to_target_not_coordinator() {
    let env = Env::new(RULES);
    let brown = env.repo("brown", Some("https://dev.azure.com/brown/p/_git/r"));
    let personal = env.repo("personal", Some("https://github.com/me/x"));
    let stranger = env.repo("stranger", Some("https://example.org/a/b"));

    let over = spawn_model_override(Some(&brown), None).unwrap();
    assert_eq!(
        over,
        ModelOverride {
            model: "brown-model".into(),
            workspace: "brown".into(),
            role: None
        }
    );
    // A worker targeting another workspace or an unknown dir never gets it.
    assert_eq!(spawn_model_override(Some(&personal), None), None);
    assert_eq!(spawn_model_override(Some(&stranger), None), None);
    assert_eq!(spawn_model_override(None, None), None);
}

#[test]
fn workspace_hooks_scoped_to_workspace_and_event() {
    let env = Env::new(
        r#"
[[hooks.PreToolUse]]
command = "global-gate"

[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
[[workspaces.rules.hooks.PreToolUse]]
command = "brown-gate"
"#,
    );
    let brown = env.repo("brown", Some("https://dev.azure.com/brown/p/_git/r"));
    let personal = env.repo("personal", Some("https://github.com/me/x"));
    let commands = |hooks: &HooksConfig| -> Vec<String> {
        hooks
            .lifecycle_entries("PreToolUse")
            .iter()
            .map(|e| e.command.clone())
            .collect()
    };

    let merged = lifecycle_hooks_for(brown.to_str(), "PreToolUse").expect("brown hooks");
    assert_eq!(commands(&merged), ["global-gate", "brown-gate"]);
    // Scoped: a personal repo, an unknown dir and another event get nothing extra.
    assert!(lifecycle_hooks_for(personal.to_str(), "PreToolUse").is_none());
    assert!(lifecycle_hooks_for(None, "PreToolUse").is_none());
    assert!(lifecycle_hooks_for(brown.to_str(), "Stop").is_none());
}

#[test]
fn project_local_config_cannot_add_workspace_rules() {
    let env = Env::new("");
    let repo = env.repo("r", Some("https://github.com/me/x"));
    std::fs::write(
        repo.join("config.toml"),
        "[[workspaces.rules]]\nname = \"evil\"\norigin = \"**\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(repo.join(".jcode")).unwrap();
    std::fs::write(
        repo.join(".jcode/config.toml"),
        "[[workspaces.rules]]\nname = \"evil\"\norigin = \"**\"\n",
    )
    .unwrap();
    std::env::set_current_dir(&repo).unwrap();
    crate::config::Config::invalidate_cache();
    assert!(resolve(&repo).is_none());
    assert!(crate::config::config().workspaces.is_empty());
}

#[test]
fn rules_are_reread_after_config_change() {
    let env = Env::new("");
    let repo = env.repo("r", Some("https://github.com/me/x"));
    assert!(resolve(&repo).is_none());
    std::fs::write(
        env.home.path().join("config.toml"),
        "[[workspaces.rules]]\nname = \"fresh\"\norigin = \"github.com/me/**\"\n",
    )
    .unwrap();
    crate::config::Config::invalidate_cache();
    assert_eq!(resolve(&repo).unwrap().name(), "fresh");
}

#[test]
fn nonblocking_probe_never_blocks_and_converges() {
    let env = Env::new(RULES);
    let repo = env.repo("r", Some("https://github.com/me/x"));
    // First call returns immediately (no facts yet) and probes in background.
    let _ = badge(&repo, false);
    let mut seen = None;
    for _ in 0..200 {
        seen = badge(&repo, false);
        if seen.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert_eq!(seen.as_deref(), Some("ws: personal · r: tracker:linear"));
}

#[cfg(unix)]
#[test]
fn run_lifecycle_runs_workspace_hook_only_in_matching_workspace() {
    let marker_dir = tempfile::TempDir::new().unwrap();
    let ws_marker = marker_dir.path().join("brown-ran");
    let global_marker = marker_dir.path().join("global-ran");
    let env = Env::new(&format!(
        r#"
[[hooks.PreToolUse]]
command = "touch {global}"

[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
[[workspaces.rules.hooks.PreToolUse]]
command = "touch {ws}"
"#,
        global = global_marker.display(),
        ws = ws_marker.display(),
    ));
    crate::env::remove_var("JCODE_HOOKS_DISABLED");
    let brown = env.repo("brown", Some("https://dev.azure.com/brown/p/_git/r"));
    let personal = env.repo("personal", Some("https://github.com/me/x"));
    let rt = tokio::runtime::Runtime::new().unwrap();
    let run = |cwd: &Path| {
        let call = crate::hooks::LifecycleCall::new(crate::hooks::LifecycleEvent::PreToolUse, "s")
            .cwd(cwd.to_str().map(str::to_owned))
            .subject("bash");
        rt.block_on(crate::hooks::run_lifecycle(call))
    };

    run(&personal);
    assert!(global_marker.exists(), "global hook runs everywhere");
    assert!(
        !ws_marker.exists(),
        "brown hook must not run in a personal repo"
    );

    run(&brown);
    assert!(ws_marker.exists(), "brown hook runs in the brown workspace");
}

#[test]
fn prompt_block_snapshot() {
    let env = Env::new(RULES);
    let brown = env.repo("b", Some("https://dev.azure.com/brown/p/_git/r"));
    git(&brown, &["config", "user.email", "me@brown.example"]);
    assert_eq!(
        prompt_block(&brown, None).as_deref(),
        Some(
            "# Workspace\n\
             Workspace (configured from git origin rules, not authoritative): brown\n\
             Tracker: ADO (Azure DevOps) is the configured tracker.\n\
             Branch template: feat/<id>-<slug>\n\
             Skills: triage-ado"
        )
    );
    git(&brown, &["config", "user.email", "x@example.com"]);
    let block = prompt_block(&brown, None).unwrap();
    assert!(block.ends_with("jcode does not change it."), "{block}");
}

#[test]
fn include_targets_and_worktree_config_feed_the_stamp() {
    let env = Env::new(RULES);
    let repo = env.repo("brown", Some("https://dev.azure.com/brown/p/_git/r"));
    let included = env.home.path().join("gitconfig-brown");
    std::fs::write(&included, "[user]\n\temail = old@example.com\n").unwrap();
    std::fs::write(
        env.global_gitconfig(),
        format!(
            "[includeIf \"gitdir:{}/\"]\n\tpath = {}\n",
            env.home.path().display(),
            included.display()
        ),
    )
    .unwrap();
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Mismatch);

    // Only the included file changes: the badge must follow without a restart.
    std::fs::write(&included, "[user]\n\temail = me@brown.example\n").unwrap();
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Match);

    // A relative include resolves against the including file's directory.
    std::fs::write(
        env.global_gitconfig(),
        "[include]\n\tpath = gitconfig-brown\n",
    )
    .unwrap();
    std::fs::write(&included, "[user]\n\temail = rel@example.com\n").unwrap();
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Mismatch);
    std::fs::write(&included, "[user]\n\temail = ME@brown.example\n").unwrap();
    assert_eq!(resolve(&repo).unwrap().identity(), Identity::Match);

    // config.worktree is part of the stamp.
    let paths = find_repo(&repo).unwrap();
    let before = config_stamp(&paths);
    std::fs::write(
        paths.git_dir.join("config.worktree"),
        "[user]\n\temail = w@x\n",
    )
    .unwrap();
    assert_ne!(before, config_stamp(&paths));
}

#[cfg(unix)]
#[test]
fn hung_git_is_killed_at_the_timeout() {
    let env = Env::new("");
    let started = Instant::now();
    let out = git_output_with(
        env.home.path(),
        &["-c", "alias.hang=!sleep 30", "hang"],
        Duration::from_millis(200),
    );
    assert_eq!(out, None);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    // A normal call is unaffected.
    assert!(git_output_with(env.home.path(), &["--version"], GIT_TIMEOUT).is_some());
}

#[test]
fn badge_cache_serves_repeat_frames_without_reprobing() {
    let env = Env::new(RULES);
    let repo = env.repo("r", Some("https://github.com/me/x"));
    // Warm the facts so the cached badge has a value.
    assert!(resolve(&repo).is_some());
    BADGES.lock().unwrap().clear();
    assert_eq!(
        badge_cached(&repo).as_deref(),
        Some("ws: personal · r: tracker:linear")
    );
    // Remove the repo: within the TTL the frame still reads the cached string
    // and does no filesystem work.
    std::fs::remove_dir_all(repo.join(".git")).unwrap();
    assert_eq!(
        badge_cached(&repo).as_deref(),
        Some("ws: personal · r: tracker:linear")
    );
    BADGES.lock().unwrap().clear();
    assert_eq!(badge_cached(&repo), None);
}

#[test]
fn config_warnings_cover_workspace_hook_entries() {
    let rules: WorkspacesConfig = toml::from_str::<toml::Table>(
        r#"
[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/**"
[[workspaces.rules.hooks.PreToolUse]]
command = "gate"
matcher = "("
fail_closed = true
"#,
    )
    .map(|doc| doc["workspaces"].clone().try_into().unwrap())
    .unwrap();
    let warnings = config_warnings(&rules);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("brown") && w.contains("disabled")),
        "{warnings:?}"
    );
}

#[test]
fn glob_regexes_are_compiled_once() {
    let rules: WorkspacesConfig = WorkspacesConfig {
        rules: vec![WorkspaceRule {
            name: "n".into(),
            origin: "cache-test.example/**".into(),
            ..Default::default()
        }],
    };
    assert!(match_rule(&rules, "cache-test.example/a").is_some());
    let cache = GLOBS.lock().unwrap();
    assert!(cache.contains_key("cache-test.example/**"));
}
