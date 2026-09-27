use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const SCRIPT: &str = include_str!("../../../scripts/browser_profiles.py");

pub struct BrowserSession {
    child: Option<tokio::process::Child>,
    input: tokio::process::ChildStdin,
    output: BufReader<tokio::process::ChildStdout>,
    owner: String,
}

impl BrowserSession {
    pub async fn open(owner: String, url: &str) -> Result<Self> {
        let root = crate::storage::jcode_dir()?.join("browser-profiles");
        let python = std::env::var("JCODE_BROWSER_PYTHON").unwrap_or_else(|_| "python3".into());
        let mut child = tokio::process::Command::new(python)
            .args(["-u", "-c", SCRIPT])
            .env("JCODE_BROWSER_ROOT", root)
            .env("JCODE_BROWSER_OWNER_PID", std::process::id().to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("Start pinned browser runtime")?;
        let input = child.stdin.take().context("browser stdin")?;
        let output = BufReader::new(child.stdout.take().context("browser stdout")?);
        let mut session = Self {
            child: Some(child),
            input,
            output,
            owner,
        };
        session
            .request(
                serde_json::json!({"action": "provider_session", "_owner": session.owner,
            "url": url, "timeout_seconds": 300}),
            )
            .await?;
        Ok(session)
    }

    async fn request(&mut self, request: Value) -> Result<Value> {
        let mut bytes = serde_json::to_vec(&request)?;
        bytes.push(b'\n');
        self.input.write_all(&bytes).await?;
        self.input.flush().await?;
        let mut line = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(45),
            (&mut self.output)
                .take(4 * 1024 * 1024)
                .read_line(&mut line),
        )
        .await
        .context("Browser session timed out")??;
        let response: Value =
            serde_json::from_str(&line).context("Malformed browser session response")?;
        if response["ok"] != true {
            bail!(
                "{}",
                response["error"]
                    .as_str()
                    .unwrap_or("Browser session failed")
            );
        }
        Ok(response["result"].clone())
    }

    pub async fn evaluate(&mut self, script: &str) -> Result<Value> {
        self.request(serde_json::json!({"script": script})).await
    }

    pub async fn close(mut self) -> Result<()> {
        self.request(serde_json::json!({"close": true})).await?;
        if let Some(mut child) = self.child.take() {
            tokio::time::timeout(std::time::Duration::from_secs(20), child.wait()).await??;
        }
        invoke(&self.owner, serde_json::json!({"action":"close"}), false).await?;
        Ok(())
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            let owner = self.owner.clone();
            if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                runtime.spawn(async move {
                    if tokio::time::timeout(std::time::Duration::from_secs(20), child.wait())
                        .await
                        .is_err()
                    {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                    }
                    close_session(&owner).await;
                });
            }
        }
    }
}

pub async fn invoke(owner: &str, input: Value, user: bool) -> Result<Value> {
    let root = crate::storage::jcode_dir()?.join("browser-profiles");
    let python = std::env::var("JCODE_BROWSER_PYTHON").unwrap_or_else(|_| "python3".into());
    invoke_at(&root, &python, owner, input, user).await
}

pub async fn close_session(owner: &str) {
    let Ok(root) = crate::storage::jcode_dir() else {
        return;
    };
    if !root.join("browser-profiles/profiles.json").exists() {
        return;
    }
    if let Err(error) = invoke(owner, serde_json::json!({"action": "close"}), false).await {
        crate::logging::warn(&format!("Browser profile cleanup pending: {error}"));
    }
}

pub async fn invoke_at(
    root: &Path,
    python: &str,
    owner: &str,
    mut input: Value,
    user: bool,
) -> Result<Value> {
    if !cfg!(unix) {
        bail!("Local browser profiles currently require a Unix host");
    }
    let object = input
        .as_object_mut()
        .context("browser input must be an object")?;
    if object.keys().any(|key| key.starts_with('_')) {
        bail!("private browser arguments are unavailable");
    }
    object.insert("_owner".into(), owner.into());
    if object.get("action").and_then(Value::as_str) == Some("goal") {
        let route = crate::systemone::resolve().context("Resolve browser System One route")?;
        object.insert(
            "_systemone".into(),
            serde_json::json!({
                "endpoint": route.endpoint_url, "model": route.default_model, "key": route.api_key
            }),
        );
    }
    let mut command = tokio::process::Command::new(python);
    command.args(["-c", SCRIPT]);
    if user {
        command.arg("--user");
    }
    command
        .env("JCODE_BROWSER_ROOT", root)
        .env("JCODE_BROWSER_OWNER_PID", std::process::id().to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().context(
        "Browser Python unavailable; configure JCODE_BROWSER_PYTHON after explicit setup",
    )?;
    struct Cleanup(Option<tokio::process::Child>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                #[cfg(unix)]
                if let Some(pid) = child.id() {
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                }
                if let Ok(runtime) = tokio::runtime::Handle::try_current() {
                    runtime.spawn(async move {
                        if tokio::time::timeout(std::time::Duration::from_secs(20), child.wait())
                            .await
                            .is_err()
                        {
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                        }
                    });
                }
            }
        }
    }
    let mut stdin = child.stdin.take().context("browser stdin unavailable")?;
    stdin.write_all(&serde_json::to_vec(&input)?).await?;
    stdin.shutdown().await?;
    drop(stdin);
    let stdout = child.stdout.take().context("browser stdout unavailable")?;
    let mut cleanup = Cleanup(Some(child));
    let mut bytes = Vec::new();
    let result = tokio::time::timeout(std::time::Duration::from_secs(330), async {
        stdout
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .await?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(std::io::Error::other("Browser output exceeded 4 MiB"));
        }
        cleanup.0.as_mut().unwrap().wait().await
    })
    .await;
    match result {
        Ok(Ok(status)) if status.success() => {
            cleanup.0 = None;
        }
        _ => {
            bail!("Browser runtime failed or timed out; profile retained for safe recovery");
        }
    }
    if bytes.len() > 4 * 1024 * 1024 {
        bail!("Browser output exceeded 4 MiB");
    }
    let response: Value =
        serde_json::from_slice(&bytes).context("Malformed browser runtime output")?;
    if response["ok"] != true {
        bail!(
            "{}",
            response["error"]
                .as_str()
                .unwrap_or("Browser runtime failed")
        );
    }
    Ok(response["result"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn profile_metadata_round_trip() {
        let root = tempfile::tempdir().unwrap();
        let result = invoke_at(
            root.path(),
            "python3",
            "test-owner",
            serde_json::json!({
                "action": "create", "profile": "rust-test", "lifetime": "persistent"
            }),
            false,
        )
        .await
        .unwrap();
        assert_eq!(result["profile"]["label"], "rust-test");
        let result = invoke_at(
            root.path(),
            "python3",
            "test-owner",
            serde_json::json!({"action": "list"}),
            false,
        )
        .await
        .unwrap();
        assert_eq!(result["profiles"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn runtime_errors_fail_closed() {
        let root = tempfile::tempdir().unwrap();
        let input = serde_json::json!({"action":"list"});
        let error = invoke_at(
            root.path(),
            "/missing-jcode-python",
            "owner",
            input.clone(),
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("Python unavailable"));
        let error = invoke_at(
            root.path(),
            "python3",
            "owner",
            serde_json::json!({"action":"list", "_owner":"other"}),
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("private browser"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let fake = root.path().join("fake-python");
            std::fs::write(&fake, "#!/bin/sh\ncat >/dev/null\nprintf 'not-json'\n").unwrap();
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o700)).unwrap();
            let error = invoke_at(root.path(), fake.to_str().unwrap(), "owner", input, false)
                .await
                .unwrap_err();
            assert!(error.to_string().contains("Malformed"));
        }
    }
}
