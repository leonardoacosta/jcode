use super::*;
use crate::provider::Provider;
use crate::tool::Registry;
use async_trait::async_trait;
use std::sync::Arc;

struct NoCallProvider;

#[async_trait]
impl Provider for NoCallProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> anyhow::Result<crate::provider::EventStream> {
        anyhow::bail!("workspace prompt test must not call the provider")
    }
    fn name(&self) -> &str {
        "mock"
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(NoCallProvider)
    }
}

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

/// The workspace block is dynamic (it follows `user.email` edits), so it must
/// land in `dynamic_part` and never in the cacheable `static_part`.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn workspace_block_is_in_dynamic_part_only() {
    let _env = crate::storage::lock_test_env();
    let home = tempfile::TempDir::new().unwrap();
    let previous = [
        ("JCODE_HOME", std::env::var_os("JCODE_HOME")),
        ("GIT_CONFIG_GLOBAL", std::env::var_os("GIT_CONFIG_GLOBAL")),
    ];
    crate::env::set_var("JCODE_HOME", home.path());
    std::fs::write(home.path().join("gitconfig"), "").unwrap();
    crate::env::set_var("GIT_CONFIG_GLOBAL", home.path().join("gitconfig"));
    std::fs::write(
        home.path().join("config.toml"),
        "[[workspaces.rules]]\nname = \"brown\"\norigin = \"dev.azure.com/brown/**\"\ntracker = \"ado\"\n",
    )
    .unwrap();
    crate::config::Config::invalidate_cache();
    let repo = home.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            "https://dev.azure.com/brown/p/_git/r",
        ],
    );
    let other = home.path().join("other");
    std::fs::create_dir_all(&other).unwrap();

    let provider: Arc<dyn Provider> = Arc::new(NoCallProvider);
    let registry = Registry::new(Arc::clone(&provider)).await;
    let mut agent = Agent::new(provider, registry);
    agent.set_working_dir(&repo.display().to_string());

    let split = agent.build_system_prompt_split(None);
    assert!(
        split.dynamic_part.contains("# Workspace\n")
            && split.dynamic_part.contains("ADO (Azure DevOps)"),
        "{}",
        split.dynamic_part
    );
    assert!(!split.static_part.contains("# Workspace"));
    assert!(!split.static_part.contains("ADO (Azure DevOps)"));

    // A directory outside any workspace gets no block at all.
    agent.set_working_dir(&other.display().to_string());
    let split = agent.build_system_prompt_split(None);
    assert!(!split.dynamic_part.contains("# Workspace"));
    assert!(!split.static_part.contains("# Workspace"));

    for (key, value) in previous {
        match value {
            Some(value) => crate::env::set_var(key, value),
            None => crate::env::remove_var(key),
        }
    }
    crate::config::Config::invalidate_cache();
}
