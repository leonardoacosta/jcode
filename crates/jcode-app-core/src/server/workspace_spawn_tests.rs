use super::*;

fn git(dir: &std::path::Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("git")
        .success();
    assert!(ok, "git {args:?}");
}

const CONFIG: &str = r#"
[agents.roles]
research = "global-research"

[[workspaces.rules]]
name = "priceless"
origin = "github.com/priceless/**"
swarm_model = "ws-swarm"
[workspaces.rules.roles]
research = "ws-research"

[[workspaces.rules]]
name = "brown"
origin = "dev.azure.com/brown/**"
"#;

async fn resolve(explicit: Option<&str>, dir: Option<&str>, role: Option<&str>) -> Option<String> {
    resolve_requested_model(explicit.map(str::to_string), dir, role).await
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn model_precedence_explicit_then_workspace_then_global_role_slot() {
    let _env = crate::storage::lock_test_env();
    let home = tempfile::TempDir::new().unwrap();
    let previous = [
        ("JCODE_HOME", std::env::var_os("JCODE_HOME")),
        ("GIT_CONFIG_GLOBAL", std::env::var_os("GIT_CONFIG_GLOBAL")),
    ];
    crate::env::set_var("JCODE_HOME", home.path());
    std::fs::write(home.path().join("gitconfig"), "").unwrap();
    crate::env::set_var("GIT_CONFIG_GLOBAL", home.path().join("gitconfig"));
    std::fs::write(home.path().join("config.toml"), CONFIG).unwrap();
    crate::config::Config::invalidate_cache();
    let repo = |name: &str, origin: &str| {
        let dir = home.path().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["remote", "add", "origin", origin]);
        dir.display().to_string()
    };
    let priceless = repo("priceless", "https://github.com/priceless/app");
    let brown = repo("brown", "https://dev.azure.com/brown/p/_git/r");
    let stranger = repo("stranger", "https://example.org/a/b");

    // Explicit wins over everything.
    assert_eq!(
        resolve(Some("explicit"), Some(&priceless), Some("research")).await,
        Some("explicit".into())
    );
    // Workspace role slot beats workspace swarm_model and the global slot.
    assert_eq!(
        resolve(None, Some(&priceless), Some("research")).await,
        Some("ws-research".into())
    );
    // Workspace swarm_model when no role slot matches.
    assert_eq!(
        resolve(None, Some(&priceless), None).await,
        Some("ws-swarm".into())
    );
    assert_eq!(
        resolve(None, Some(&priceless), Some("edit")).await,
        Some("ws-swarm".into())
    );
    // A workspace with no override falls back to the global role slot.
    assert_eq!(
        resolve(None, Some(&brown), Some("research")).await,
        Some("global-research".into())
    );
    // Target outside any workspace, or no target: global slot, else inherit.
    assert_eq!(
        resolve(None, Some(&stranger), Some("research")).await,
        Some("global-research".into())
    );
    assert_eq!(resolve(None, Some(&stranger), None).await, None);
    assert_eq!(resolve(None, None, None).await, None);
    // Blank explicit is "unset".
    assert_eq!(
        resolve(Some("  "), Some(&priceless), None).await,
        Some("ws-swarm".into())
    );

    // Provenance is keyed by worker and keeps only the coordinator's workspace.
    record_provenance("worker-x", Some(priceless.clone())).await;
    assert_eq!(
        crate::workspace::provenance("worker-x").unwrap().workspace,
        "priceless"
    );
    record_provenance("worker-y", Some(stranger)).await;
    assert!(crate::workspace::provenance("worker-y").is_none());

    for (key, value) in previous {
        match value {
            Some(value) => crate::env::set_var(key, value),
            None => crate::env::remove_var(key),
        }
    }
    crate::config::Config::invalidate_cache();
}
