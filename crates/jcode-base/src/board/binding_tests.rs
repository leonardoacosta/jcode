use super::*;
use std::ffi::OsString;
use std::process::{Command, Stdio};

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

/// Disposable JCODE_HOME with a global config, serialized on the shared
/// test-env lock. Mirrors `workspace_tests::Env`.
struct Env {
    _guard: std::sync::MutexGuard<'static, ()>,
    home: tempfile::TempDir,
    prev: Vec<(&'static str, Option<OsString>)>,
    cwd: Option<PathBuf>,
}

impl Env {
    fn new(config_toml: &str) -> Self {
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
        std::fs::write(home.path().join("config.toml"), config_toml).unwrap();
        crate::config::Config::invalidate_cache();
        Self {
            _guard: guard,
            home,
            prev,
            cwd: std::env::current_dir().ok(),
        }
    }

    fn repo(&self, name: &str, origin: &str) -> PathBuf {
        let dir = self.home.path().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["remote", "add", "origin", origin]);
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
name = "personal"
origin = "github.com/me/**"
tracker = "linear"
[workspaces.rules.board]
backend = "linear"
team = "team-personal"

[[workspaces.rules]]
name = "priceless"
origin = "github.com/priceless/**"
tracker = "linear"
[workspaces.rules.board]
backend = "Linear"
team = "team-priceless"
project = "proj-1"
server = "LinearAlt"

[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
tracker = "ado"
[workspaces.rules.board]
backend = "ado"
team = "brown-team"

[[workspaces.rules]]
name = "brown-linear"
origin = "dev.azure.com/browndrift/**"
tracker = "ado"
[workspaces.rules.board]
backend = "linear"
team = "BROWN"

[[workspaces.rules]]
name = "Brown"
origin = "github.com/brownname/**"
[workspaces.rules.board]
backend = "linear"
team = "BROWN"

[[workspaces.rules]]
name = "nobinding"
origin = "github.com/nobinding/**"

[[workspaces.rules]]
name = "noteam"
origin = "github.com/noteam/**"
[workspaces.rules.board]
backend = "linear"

[[workspaces.rules]]
name = "badbackend"
origin = "github.com/badbackend/**"
[workspaces.rules.board]
backend = "jira"
team = "t"

[[workspaces.rules]]
name = "mismatch"
origin = "github.com/mismatch/**"
tracker = "ado"
[workspaces.rules.board]
backend = "linear"
team = "t"

[[workspaces.rules]]
name = "wrongtracker"
origin = "github.com/wrongtracker/**"
tracker = "linear"
[workspaces.rules.board]
backend = "ado"
team = "t"

[[workspaces.rules]]
name = "amb-a"
origin = "github.com/amb/**"
[workspaces.rules.board]
backend = "linear"
team = "a"

[[workspaces.rules]]
name = "amb-b"
origin = "github.com/amb/**"
[workspaces.rules.board]
backend = "linear"
team = "b"
"#;

fn resolve_ok(env: &Env, repo: &str, origin: &str) -> BoardScope {
    let dir = env.repo(repo, origin);
    resolve_board(&dir).unwrap_or_else(|e| panic!("{origin}: {e}"))
}

fn resolve_err(env: &Env, repo: &str, origin: &str) -> BoardUnavailable {
    let dir = env.repo(repo, origin);
    resolve_board(&dir).expect_err(origin)
}

#[test]
fn personal_and_priceless_linear_resolve_with_canonical_root() {
    let env = Env::new(RULES);
    let dir = env.repo("p", "https://github.com/me/x");
    let scope = resolve_board(&dir).unwrap();
    assert_eq!(scope.world, "personal");
    assert_eq!(scope.backend, BoardBackend::Linear);
    assert_eq!(scope.team, "team-personal");
    assert_eq!(scope.project, None);
    assert_eq!(scope.server, "Linear");
    assert!(!scope.read_only());
    assert_eq!(scope.canonical_root, dir.canonicalize().unwrap());
    assert_eq!(scope.scope_label(), "Linear · team-personal · team-wide");

    let scope = resolve_ok(&env, "pl", "https://github.com/priceless/y");
    assert_eq!(scope.world, "priceless");
    assert_eq!(scope.project.as_deref(), Some("proj-1"));
    assert_eq!(scope.server, "LinearAlt");
    assert_eq!(
        scope.scope_label(),
        "Linear · team-priceless · project proj-1"
    );
}

#[test]
fn subdirectory_resolves_to_repository_root() {
    let env = Env::new(RULES);
    let dir = env.repo("p", "https://github.com/me/x");
    let sub = dir.join("a/b");
    std::fs::create_dir_all(&sub).unwrap();
    let scope = resolve_board(&sub).unwrap();
    assert_eq!(scope.canonical_root, dir.canonicalize().unwrap());
}

#[cfg(unix)]
#[test]
fn symlink_path_resolves_to_same_canonical_root() {
    let env = Env::new(RULES);
    let dir = env.repo("p", "https://github.com/me/x");
    let link = env.home.path().join("link");
    std::os::unix::fs::symlink(&dir, &link).unwrap();
    let via_link = resolve_board(&link).unwrap();
    let direct = resolve_board(&dir).unwrap();
    assert_eq!(via_link, direct);
    assert_eq!(via_link.canonical_root, dir.canonicalize().unwrap());
}

#[test]
fn worktree_path_resolves_through_its_own_origin() {
    let env = Env::new(RULES);
    let main = env.repo("main", "https://github.com/me/x");
    git(
        &main,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@e",
            "commit",
            "--allow-empty",
            "-qm",
            "i",
        ],
    );
    let wt = env.home.path().join("wt");
    git(&main, &["worktree", "add", "-q", wt.to_str().unwrap()]);
    let scope = resolve_board(&wt).unwrap();
    assert_eq!(scope.world, "personal");
    assert_eq!(scope.team, "team-personal");
    assert_eq!(scope.canonical_root, wt.canonicalize().unwrap());
}

#[test]
fn unknown_world_has_no_board_and_never_falls_back() {
    let env = Env::new(RULES);
    assert_eq!(
        resolve_err(&env, "u", "https://github.com/stranger/x"),
        BoardUnavailable::UnknownWorld
    );
    // Not a repository at all.
    let plain = env.home.path().join("plain");
    std::fs::create_dir_all(&plain).unwrap();
    assert_eq!(resolve_board(&plain), Err(BoardUnavailable::UnknownWorld));
}

#[test]
fn empty_config_has_no_board() {
    let env = Env::new("");
    assert_eq!(
        resolve_err(&env, "u", "https://github.com/me/x"),
        BoardUnavailable::UnknownWorld
    );
}

#[test]
fn project_local_config_cannot_add_a_board() {
    let env = Env::new("");
    let dir = env.repo("r", "https://github.com/me/x");
    let evil = "[[workspaces.rules]]\nname = \"evil\"\norigin = \"**\"\n[workspaces.rules.board]\nbackend = \"linear\"\nteam = \"t\"\n";
    std::fs::write(dir.join("config.toml"), evil).unwrap();
    std::fs::create_dir_all(dir.join(".jcode")).unwrap();
    std::fs::write(dir.join(".jcode/config.toml"), evil).unwrap();
    // The loader would see these if it read project-relative config.
    std::env::set_current_dir(&dir).unwrap();
    crate::config::Config::invalidate_cache();
    assert!(crate::config::config().workspaces.is_empty());
    assert_eq!(resolve_board(&dir), Err(BoardUnavailable::UnknownWorld));
}

#[test]
fn ambiguous_rules_with_differing_boards_refuse() {
    let env = Env::new(RULES);
    match resolve_err(&env, "a", "https://github.com/amb/x") {
        BoardUnavailable::Ambiguous { worlds } => {
            assert_eq!(worlds, vec!["amb-a".to_string(), "amb-b".to_string()]);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn identical_overlapping_rules_are_not_ambiguous() {
    let toml = r#"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/**"
[workspaces.rules.board]
backend = "linear"
team = "t"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/x"
[workspaces.rules.board]
backend = "linear"
team = "t"
"#;
    let env = Env::new(toml);
    assert_eq!(resolve_ok(&env, "r", "https://github.com/me/x").team, "t");
}

#[test]
fn invalid_config_denies_instead_of_using_defaults() {
    let env = Env::new("[[workspaces.rules]\nname = ");
    let dir = env.repo("r", "https://github.com/me/x");
    assert!(matches!(
        resolve_board(&dir),
        Err(BoardUnavailable::InvalidConfig(_))
    ));
}

#[test]
fn binding_refusals() {
    let env = Env::new(RULES);
    assert!(matches!(
        resolve_err(&env, "a", "https://github.com/nobinding/x"),
        BoardUnavailable::NoBinding { .. }
    ));
    assert!(matches!(
        resolve_err(&env, "b", "https://github.com/noteam/x"),
        BoardUnavailable::MissingTeam { .. }
    ));
    match resolve_err(&env, "c", "https://github.com/badbackend/x") {
        BoardUnavailable::UnknownBackend { value, .. } => assert_eq!(value, "jira"),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        resolve_err(&env, "d", "https://github.com/wrongtracker/x"),
        BoardUnavailable::TrackerMismatch { .. }
    ));
}

#[test]
fn brown_with_linear_board_is_refused() {
    let env = Env::new(RULES);
    // tracker = "ado" rule with a linear board.
    let e = resolve_err(&env, "a", "https://dev.azure.com/browndrift/p/_git/r");
    assert!(
        matches!(e, BoardUnavailable::BrownRequiresAdo { .. }),
        "{e:?}"
    );
    assert!(e.to_string().contains("Brown boards require ADO"));
    // name = "Brown" (case-insensitive) with no tracker.
    let e = resolve_err(&env, "b", "https://github.com/brownname/x");
    assert!(
        matches!(e, BoardUnavailable::BrownRequiresAdo { .. }),
        "{e:?}"
    );
    // tracker = "ado" + linear board outranks the generic tracker mismatch.
    let e = resolve_err(&env, "c", "https://github.com/mismatch/x");
    assert!(
        matches!(e, BoardUnavailable::BrownRequiresAdo { .. }),
        "{e:?}"
    );
}

#[test]
fn ado_resolves_read_only() {
    let env = Env::new(RULES);
    let scope = resolve_ok(&env, "a", "https://dev.azure.com/brown/p/_git/r");
    assert_eq!(scope.backend, BoardBackend::Ado);
    assert!(scope.read_only());
    assert_eq!(scope.team, "brown-team");
    assert_eq!(scope.scope_label(), "ADO · brown-team · team-wide");
}

#[test]
fn identity_mismatch_denies_board() {
    let toml = r#"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/**"
expected_email = "me@example.com"
[workspaces.rules.board]
backend = "linear"
team = "t"
"#;
    let env = Env::new(toml);
    assert!(matches!(
        resolve_err(&env, "r", "https://github.com/me/x"),
        BoardUnavailable::IdentityUnverified { .. }
    ));
}

#[test]
fn linear_board_on_an_azure_devops_host_is_refused_whatever_the_world_is_called() {
    let toml = r#"
[[workspaces.rules]]
name = "acme"
origin = "dev.azure.com/acme/**"
[workspaces.rules.board]
backend = "linear"
team = "t"

[[workspaces.rules]]
name = "acme-ssh"
origin = "ssh.dev.azure.com/**"
[workspaces.rules.board]
backend = "linear"
team = "t"

[[workspaces.rules]]
name = "acme-vs"
origin = "acme.visualstudio.com/**"
tracker = "linear"
[workspaces.rules.board]
backend = "linear"
team = "t"

[[workspaces.rules]]
name = "acme-ado"
origin = "dev.azure.com/adoboard/**"
[workspaces.rules.board]
backend = "ado"
team = "t"
"#;
    let env = Env::new(toml);
    for (n, url) in [
        ("a", "https://dev.azure.com/acme/p/_git/r"),
        ("b", "git@ssh.dev.azure.com:v3/acme/p/r"),
        ("c", "https://acme.visualstudio.com/p/_git/r"),
    ] {
        let dir = env.repo(n, url);
        match resolve_board(&dir) {
            Err(BoardUnavailable::AzureHostRequiresAdo { world }) => assert!(!world.is_empty()),
            other => panic!("{url}: {other:?}"),
        }
    }
    let e = BoardUnavailable::AzureHostRequiresAdo { world: "w".into() };
    assert!(
        e.to_string()
            .contains("Azure DevOps repositories require an ADO board")
    );
    // An ADO board on an Azure host still resolves.
    assert_eq!(
        resolve_ok(&env, "d", "https://dev.azure.com/adoboard/p/_git/r").backend,
        BoardBackend::Ado
    );
}

#[test]
fn overlapping_rules_with_different_expected_email_are_ambiguous() {
    let toml = r#"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/**"
[workspaces.rules.board]
backend = "linear"
team = "t"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/x"
expected_email = "me@example.com"
[workspaces.rules.board]
backend = "linear"
team = "t"
"#;
    let env = Env::new(toml);
    assert!(matches!(
        resolve_err(&env, "r", "https://github.com/me/x"),
        BoardUnavailable::Ambiguous { .. }
    ));
}

#[test]
fn overlapping_rules_with_the_same_expected_email_resolve_only_when_identity_matches() {
    // Identity is checked on every overlapping rule (they now must agree on
    // expected_email, so one verdict covers all of them).
    let toml = r#"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/**"
expected_email = "me@example.com"
[workspaces.rules.board]
backend = "linear"
team = "t"
[[workspaces.rules]]
name = "w"
origin = "github.com/me/x"
expected_email = "me@example.com"
[workspaces.rules.board]
backend = "linear"
team = "t"
"#;
    let env = Env::new(toml);
    let dir = env.repo("r", "https://github.com/me/x");
    assert!(matches!(
        resolve_board(&dir),
        Err(BoardUnavailable::IdentityUnverified { .. })
    ));
    git(&dir, &["config", "user.email", "me@example.com"]);
    crate::config::Config::invalidate_cache();
    assert_eq!(resolve_board(&dir).unwrap().team, "t");
}
