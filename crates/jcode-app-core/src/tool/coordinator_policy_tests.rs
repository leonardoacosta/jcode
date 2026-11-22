use super::*;
use serde_json::json;

fn id(p: &Path) -> PathBuf {
    p.to_path_buf()
}

fn ok(tool: &str, input: Value) -> bool {
    allows(tool, &input, &id).is_ok()
}

/// Throwaway git repository with an isolated environment: no system or global
/// git configuration, so a developer's own `core.pager`, hooks or filters can
/// never change a test outcome.
struct Repo {
    dir: tempfile::TempDir,
}

const ISOLATED_GIT_ENV: [(&str, &str); 4] = [
    ("GIT_CONFIG_GLOBAL", "/dev/null"),
    ("GIT_CONFIG_NOSYSTEM", "1"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_PAGER", "cat"),
];

fn isolate_probe() {
    PROBE_ENV.with(|env| {
        *env.borrow_mut() = ISOLATED_GIT_ENV
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
    });
}

impl Repo {
    fn new() -> Self {
        let repo = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q", "."]);
        std::fs::write(repo.path().join("f"), "a\n").unwrap();
        repo.git(&["add", "f"]);
        repo.git(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "x",
        ]);
        isolate_probe();
        repo
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Run real git here, isolated, stdin closed and stdout piped like the
    /// bash tool, so a configured program that really runs leaves its marker.
    fn git(&self, args: &[&str]) -> std::process::Output {
        let mut command = std::process::Command::new("git");
        command
            .current_dir(self.path())
            .args(args)
            .stdin(std::process::Stdio::null());
        for (key, value) in ISOLATED_GIT_ENV {
            command.env(key, value);
        }
        command.output().unwrap()
    }

    fn config(&self, key: &str, value: &str) {
        assert!(self.git(&["config", key, value]).status.success(), "{key}");
    }

    /// Executable script that records `label` in the repo's marker file.
    fn script(&self, name: &str, label: &str) -> String {
        let path = self.path().join(name);
        let marker = self.marker_path();
        std::fs::write(
            &path,
            format!("#!/bin/sh\necho {label} >> '{}'\ncat\n", marker.display()),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path.display().to_string()
    }

    fn marker_path(&self) -> PathBuf {
        // Outside the worktree, so it never shows up as an untracked file.
        self.dir.path().parent().unwrap().join(format!(
            "marker-{}",
            self.dir.path().file_name().unwrap().to_string_lossy()
        ))
    }

    fn ran(&self) -> Vec<String> {
        std::fs::read_to_string(self.marker_path())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn reset_marker(&self) {
        let _ = std::fs::remove_file(self.marker_path());
    }

    /// Make the index stat-dirty so `git status` refreshes it.
    fn touch(&self) {
        std::fs::write(self.path().join("f"), "a\n").unwrap();
    }

    fn allows(&self, command: &str) -> Result<(), String> {
        let root = self.path().to_path_buf();
        allows("bash", &json!({ "command": command }), &move |p| {
            root.join(p)
        })
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.marker_path());
    }
}

thread_local! {
    static CLEAN_REPO: Repo = Repo::new();
}

/// Bash allowlist decision with git resolved in a clean throwaway repo.
fn bash(cmd: &str) -> bool {
    CLEAN_REPO.with(|repo| repo.allows(cmd).is_ok())
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
        "git diff --stat --no-textconv --no-ext-diff && rg -n TODO src",
        "git log --oneline -5 --no-textconv --no-ext-diff | head -3",
        "git -C /tmp show HEAD --no-ext-diff --no-textconv",
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

/// `git diff/log/show` run configured textconv and external diff programs
/// without any flag; `blame` and `grep` can too. Only the explicit opt-out
/// spelling passes, and `blame`/`grep` are not allowlisted at all.
#[test]
fn git_commands_that_can_run_configured_programs_denied_without_opt_out() {
    for c in [
        "git diff",
        "git diff HEAD~1",
        "git diff --stat",
        "git diff --cached",
        "git log",
        "git log -p",
        "git log -G foo",
        "git log -S foo --oneline",
        "git log --oneline -5",
        "git show",
        "git show HEAD",
        "git show HEAD:file",
        // One of the two flags is not enough.
        "git diff --no-textconv",
        "git diff --no-ext-diff",
        "git log -p --no-textconv",
        "git show --no-ext-diff",
        // Abbreviated or inverted spellings do not count, and re-enabling wins.
        "git diff --no-text --no-ext",
        "git diff --no-textconv --no-ext-diff --textconv",
        "git diff --no-textconv --no-ext-diff --ext-diff",
        "git diff --no-textconv --no-ext-diff --ext-d",
        "git diff --no-textconv --no-ext-diff --textc",
        "git log --no-textconv --no-ext-diff --output=out",
        // `blame` and `grep` run textconv / pagers and are not allowlisted.
        "git blame f",
        "git blame --no-textconv f",
        "git grep foo",
        "git grep -p foo",
        "git grep --textconv foo",
        "git grep -O foo",
        "git grep --open-files-in-pager=sh foo",
        "git --no-pager grep foo",
        "git -C /tmp blame f",
        // Pipelines and chains are checked segment by segment.
        "git status && git diff",
        "git log --oneline | head -3",
    ] {
        assert!(!bash(c), "must be denied: {c}");
    }
    let reason = allows("bash", &json!({"command": "git diff"}), &id).unwrap_err();
    assert!(
        reason.contains("--no-textconv") && reason.contains("--no-ext-diff"),
        "the denial tells the coordinator what to add: {reason}"
    );
}

#[test]
fn git_commands_with_both_opt_out_flags_allowed() {
    for c in [
        "git diff --no-textconv --no-ext-diff",
        "git diff --no-ext-diff --no-textconv HEAD~1 -- src",
        "git diff --stat --no-textconv --no-ext-diff",
        "git --no-pager diff --no-textconv --no-ext-diff",
        "git -C /tmp log -p --no-textconv --no-ext-diff -3",
        "git show HEAD --no-textconv --no-ext-diff",
        "git log --oneline --no-textconv --no-ext-diff | head -3",
        "git diff --exit-code --no-textconv --no-ext-diff",
        "git status",
        "git status --short",
        "git branch -a",
    ] {
        assert!(bash(c), "must be allowed: {c}");
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
        "git log --format='%h %s' -3 --no-textconv --no-ext-diff",
        "git log --pretty=format:'%H' --no-textconv --no-ext-diff",
        "ls *.rs",
        "cat src/*.rs | wc -l",
        "find . -name '*.rs'",
        "rg 'a*' src",
        "rg \"a?\" src",
        "git log --no-textconv --no-ext-diff -- 'src/*'",
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

/// Reproduction: a clean filter and `core.fsmonitor` run programs from plain
/// `git status`/`git diff`. The policy must deny them, naming the key, while
/// the same commands stay allowed in a repository with clean configuration.
#[test]
fn git_with_filter_or_fsmonitor_configured_is_denied_but_clean_repo_is_allowed() {
    let clean = Repo::new();
    for c in [
        "git status",
        "git diff --no-textconv --no-ext-diff",
        "git log --no-textconv --no-ext-diff",
        "git show --no-textconv --no-ext-diff",
        "git -C . status",
    ] {
        assert!(clean.allows(c).is_ok(), "clean repo allows: {c}");
    }

    let repo = Repo::new();
    std::fs::write(repo.path().join(".gitattributes"), "f filter=p\n").unwrap();
    repo.config("filter.p.clean", &repo.script("clean.sh", "clean-filter"));
    repo.config("core.fsmonitor", &repo.script("fsmon.sh", "fsmonitor"));

    // The danger is real: ordinary git runs both programs.
    repo.touch();
    repo.reset_marker();
    repo.git(&["status"]);
    let ran = repo.ran();
    assert!(ran.contains(&"clean-filter".to_string()), "{ran:?}");
    assert!(ran.contains(&"fsmonitor".to_string()), "{ran:?}");

    // The policy denies git in this repo, so neither can be reached.
    for c in [
        "git status",
        "git --no-optional-locks status",
        "git diff --no-textconv --no-ext-diff",
        "git log --no-textconv --no-ext-diff",
        "git show --no-textconv --no-ext-diff",
        "git -C . status",
        "git log --no-textconv --no-ext-diff | head -3",
    ] {
        let err = repo.allows(c).unwrap_err();
        assert!(
            err.contains("filter.p.clean") || err.contains("core.fsmonitor"),
            "{c}: {err}"
        );
    }
    repo.reset_marker();
    assert!(repo.ran().is_empty());

    // Unrelated segments in a pipeline do not hide it.
    assert!(repo.allows("ls && git status").is_err());
    // Other commands are untouched by git configuration.
    assert!(repo.allows("ls").is_ok());
    assert!(repo.allows("rg foo").is_ok());
}

#[test]
fn each_filter_kind_and_fsmonitor_hook_denied_while_disabled_fsmonitor_allowed() {
    for (key, value, denied) in [
        ("filter.x.clean", "prog", true),
        ("filter.x.smudge", "prog", true),
        ("filter.x.process", "prog", true),
        ("core.fsmonitor", "/bin/hook", true),
        ("core.fsmonitor", "true", true),
        ("core.fsmonitor", "false", false),
        ("core.fsmonitor", "0", false),
        ("core.fsmonitor", "off", false),
        // Not a program key, so unrelated configuration never blocks git.
        ("filter.x.required", "true", false),
        ("core.editor", "vim", false),
        ("user.name", "me", false),
    ] {
        let repo = Repo::new();
        repo.config(key, value);
        let result = repo.allows("git status");
        assert_eq!(result.is_err(), denied, "{key}={value}: {result:?}");
    }
}

#[test]
fn included_and_environment_configuration_is_seen_by_the_probe() {
    // `include.path` pulls a program key in from another file.
    let repo = Repo::new();
    let extra = repo.path().join("extra.cfg");
    std::fs::write(&extra, "[core]\n\tfsmonitor = /bin/hook\n").unwrap();
    repo.config("include.path", &extra.display().to_string());
    let err = repo.allows("git status").unwrap_err();
    assert!(err.contains("core.fsmonitor"), "{err}");

    // `GIT_CONFIG_COUNT` style injection reaches the bash tool's git too.
    let repo = Repo::new();
    PROBE_ENV.with(|env| {
        let mut env = env.borrow_mut();
        env.push(("GIT_CONFIG_COUNT".into(), "1".into()));
        env.push(("GIT_CONFIG_KEY_0".into(), "filter.e.clean".into()));
        env.push(("GIT_CONFIG_VALUE_0".into(), "/bin/hook".into()));
    });
    let err = repo.allows("git status").unwrap_err();
    assert!(err.contains("filter.e.clean"), "{err}");
    isolate_probe();
    assert!(repo.allows("git status").is_ok());
}

#[test]
fn gpg_program_denied_only_where_signatures_are_verified() {
    let repo = Repo::new();
    repo.config("gpg.program", "/bin/hook");
    let flags = "--no-textconv --no-ext-diff";
    for sub in ["log", "show"] {
        let err = repo.allows(&format!("git {sub} {flags}")).unwrap_err();
        assert!(err.contains("gpg.program"), "{sub}: {err}");
    }
    assert!(repo.allows("git status").is_ok());
    assert!(repo.allows(&format!("git diff {flags}")).is_ok());
    repo.git(&["config", "--unset", "gpg.program"]);
    repo.config("gpg.ssh.program", "/bin/hook");
    assert!(repo.allows(&format!("git log {flags}")).is_err());
}

#[test]
fn index_refresh_hook_requires_no_optional_locks_and_hookspath_is_honoured() {
    let repo = Repo::new();
    let hook = repo.path().join(".git/hooks/post-index-change");
    std::fs::write(
        &hook,
        format!(
            "#!/bin/sh\necho hook >> '{}'\n",
            repo.marker_path().display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Reproduction: an ordinary refresh runs the hook; `--no-optional-locks` does not.
    repo.touch();
    repo.reset_marker();
    repo.git(&["status"]);
    assert_eq!(repo.ran(), ["hook"]);
    repo.touch();
    repo.reset_marker();
    repo.git(&["--no-optional-locks", "status"]);
    assert!(repo.ran().is_empty());

    let err = repo.allows("git status").unwrap_err();
    assert!(err.contains("--no-optional-locks"), "{err}");
    assert!(repo.allows("git diff --no-textconv --no-ext-diff").is_err());
    assert!(repo.allows("git --no-optional-locks status").is_ok());
    assert!(
        repo.allows("git --no-optional-locks diff --no-textconv --no-ext-diff")
            .is_ok()
    );
    // `log` and `show` never refresh the index.
    assert!(repo.allows("git log --no-textconv --no-ext-diff").is_ok());

    // `core.hooksPath` moves the hook directory: the probe follows it.
    let empty = tempfile::tempdir().unwrap();
    repo.config("core.hooksPath", &empty.path().display().to_string());
    assert!(repo.allows("git status").is_ok());
    std::fs::copy(&hook, empty.path().join("post-index-change")).unwrap();
    assert!(repo.allows("git status").is_err());
}

#[test]
fn initialized_submodule_requires_ignore_submodules_all() {
    let sub = Repo::new();
    let repo = Repo::new();
    let added = std::process::Command::new("git")
        .current_dir(repo.path())
        .args(["-c", "protocol.file.allow=always", "submodule", "add", "-q"])
        .arg(sub.path())
        .arg("sm")
        .envs(ISOLATED_GIT_ENV)
        .output()
        .unwrap();
    assert!(added.status.success(), "{added:?}");

    // Reproduction: the submodule's own fsmonitor runs for a plain `status`.
    let child = repo.path().join("sm");
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .current_dir(&child)
            .args(args)
            .envs(ISOLATED_GIT_ENV)
            .output()
            .unwrap()
    };
    let script = repo.script("fsmon.sh", "submodule-fsmonitor");
    assert!(run(&["config", "core.fsmonitor", &script]).status.success());
    std::fs::write(child.join("f"), "dirty\n").unwrap();
    repo.reset_marker();
    repo.git(&["status"]);
    assert_eq!(repo.ran(), ["submodule-fsmonitor"]);
    repo.reset_marker();
    repo.git(&["status", "--ignore-submodules=all"]);
    assert!(repo.ran().is_empty());

    let flags = "--no-textconv --no-ext-diff";
    for c in [
        "git --no-optional-locks status".to_string(),
        format!("git diff {flags}"),
        format!("git --no-optional-locks status --ignore-submodules=untracked"),
        format!("git diff {flags} --ignore-submodules=none"),
    ] {
        let err = repo.allows(&c).unwrap_err();
        assert!(err.contains("--ignore-submodules=all"), "{c}: {err}");
    }
    for c in [
        "git --no-optional-locks status --ignore-submodules=all".to_string(),
        "git --no-optional-locks status --ignore-submodules".to_string(),
        format!("git diff {flags} --ignore-submodules=dirty"),
        // `log` and `show` do not enter submodules.
        format!("git log {flags}"),
    ] {
        assert!(repo.allows(&c).is_ok(), "{c}");
    }
    // A repository without submodules is unaffected.
    assert!(Repo::new().allows("git --no-optional-locks status").is_ok());
}

#[test]
fn dash_c_directory_is_probed_relative_to_the_base_directory() {
    let outer = tempfile::tempdir().unwrap();
    let repo = Repo::new();
    repo.config("core.fsmonitor", "/bin/hook");
    let clean = outer.path().join("clean");
    std::fs::create_dir(&clean).unwrap();
    let init = std::process::Command::new("git")
        .current_dir(&clean)
        .args(["init", "-q", "."])
        .envs(ISOLATED_GIT_ENV)
        .output()
        .unwrap();
    assert!(init.status.success());
    // From a clean cwd, `-C` into the configured repo is still denied...
    let dirty = repo.path().display().to_string();
    let result = allows(
        "bash",
        &json!({ "command": format!("git -C {dirty} status") }),
        &|p| clean.join(p),
    );
    assert!(result.unwrap_err().contains("core.fsmonitor"));
    // ...and a clean cwd alone is allowed.
    isolate_probe();
    assert!(
        allows("bash", &json!({"command": "git status"}), &|p| clean
            .join(p))
        .is_ok()
    );
}

/// Measured with real git: these configured programs cannot be reached from the
/// allowlisted commands in the bash tool's environment, so they do not block
/// git (`core.pager = delta` and `diff.external = difft` are common).
#[test]
fn configured_pager_credential_and_diff_programs_are_inert_and_not_blocked() {
    let repo = Repo::new();
    std::fs::write(repo.path().join(".gitattributes"), "f diff=probe\n").unwrap();
    let prog = repo.script("prog.sh", "RAN");
    for key in [
        "core.pager",
        "pager.log",
        "pager.diff",
        "pager.status",
        "credential.helper",
        "diff.external",
        "diff.probe.textconv",
        "diff.probe.command",
    ] {
        repo.config(key, &prog);
    }
    std::fs::write(repo.path().join("f"), "changed\n").unwrap();
    let flags = "--no-textconv --no-ext-diff";
    for c in [
        format!("git --no-optional-locks status"),
        format!("git diff {flags}"),
        format!("git log -p {flags}"),
        format!("git show {flags}"),
        "git branch -a".to_string(),
    ] {
        assert!(repo.allows(&c).is_ok(), "{c}");
        // And really running the allowed command executes nothing. Piped
        // stdout and GIT_PAGER=cat match the bash tool.
        let words: Vec<&str> = c.split_whitespace().skip(1).collect();
        repo.reset_marker();
        let mut command = std::process::Command::new("git");
        command
            .current_dir(repo.path())
            .args(&words)
            .stdin(std::process::Stdio::null())
            .env("GIT_EXTERNAL_DIFF", &prog)
            .envs(ISOLATED_GIT_ENV);
        command.output().unwrap();
        assert!(repo.ran().is_empty(), "{c} ran {:?}", repo.ran());
    }
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
