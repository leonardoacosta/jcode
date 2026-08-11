use super::{
    BrowserInput, BrowserProvider, add_metadata_field, attach_browser_metadata,
    render_browser_output,
};
use crate::tool::{ToolContext, ToolOutput};
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

const MIN_AGENT_BROWSER_VERSION: (u64, u64, u64) = (0, 27, 3);
const MAX_AGENT_BROWSER_MINOR_EXCLUSIVE: u64 = 35;
const NORMAL_TIMEOUT: Duration = Duration::from_secs(30);
const INSTALL_TIMEOUT: Duration = Duration::from_secs(600);
const MAX_STDIN_BYTES: usize = 1024 * 1024;
const MAX_STDOUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 256 * 1024;
const MAX_SCREENSHOT_BYTES: u64 = 25 * 1024 * 1024;
const IDLE_TIMEOUT_MS: &str = "1800000";

pub struct AgentBrowserProvider;

#[derive(Debug, Clone)]
struct TrustedExecutable {
    path: PathBuf,
    version: String,
    fingerprint: String,
}

#[derive(Debug)]
struct ProcessResult {
    status_success: bool,
    stdout: String,
    stderr: String,
}

static SESSION_LOCKS: OnceLock<Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>>> = OnceLock::new();

#[async_trait]
impl BrowserProvider for AgentBrowserProvider {
    fn id(&self) -> &'static str {
        "agent_browser"
    }

    fn supported_browsers(&self) -> &'static [&'static str] {
        &["chrome"]
    }

    async fn status(&self, _ctx: &ToolContext) -> Result<ToolOutput> {
        let status = chrome_status().await;
        Ok(match status {
            Ok((exe, doctor)) => ToolOutput::new(format!(
                "Chrome provider is ready via agent-browser {}. Doctor is non-installing and may clean stale daemon sidecars only.",
                exe.version
            ))
            .with_title("browser status")
            .with_metadata(json!({
                "ready": true,
                "backend": self.id(),
                "browser": "chrome",
                "version": exe.version,
                "executable": exe.path.display().to_string(),
                "fingerprint": exe.fingerprint,
                "doctor": summarize_doctor(&doctor),
                "doctor_side_effects": "may clean stale daemon socket, pid, and version sidecars"
            })),
            Err(err) => ToolOutput::new(format!(
                "Chrome provider is not ready: {err}. Install agent-browser, trust the executable with JCODE_AGENT_BROWSER_BIN, or run action='setup' with browser='chrome' when only the Chrome runtime is missing."
            ))
            .with_title("browser status")
            .with_metadata(json!({
                "ready": false,
                "backend": "unconfigured",
                "browser": "chrome",
                "error": err.to_string()
            })),
        })
    }

    async fn setup(&self) -> Result<ToolOutput> {
        match chrome_status().await {
            Ok((exe, doctor)) => Ok(ToolOutput::new(format!(
                "Chrome provider is already ready via agent-browser {}. No installation was run.",
                exe.version
            ))
            .with_title("browser setup")
            .with_metadata(json!({
                "ready": true,
                "backend": self.id(),
                "browser": "chrome",
                "version": exe.version,
                "executable": exe.path.display().to_string(),
                "fingerprint": exe.fingerprint,
                "doctor": summarize_doctor(&doctor),
                "installed": false
            }))),
            Err(before) => {
                let exe = discover_trusted_executable().await?;
                let result = run_agent_browser(
                    &exe,
                    &[],
                    &["install".to_string()],
                    None,
                    INSTALL_TIMEOUT,
                    MAX_STDOUT_BYTES,
                    MAX_STDERR_BYTES,
                )
                .await
                .with_context(|| "agent-browser install failed to start")?;
                let after = chrome_status().await;
                let ready = after.is_ok();
                Ok(ToolOutput::new(if ready {
                    "Chrome runtime installation completed and doctor now reports ready.".to_string()
                } else {
                    format!(
                        "agent-browser install completed or returned, but Chrome provider is still not ready. Before: {before}. After: {}",
                        after.err().map(|e| e.to_string()).unwrap_or_else(|| "unknown".into())
                    )
                })
                .with_title(if ready {
                    "browser setup"
                } else {
                    "browser setup (incomplete)"
                })
                .with_metadata(json!({
                    "ready": ready,
                    "backend": self.id(),
                    "browser": "chrome",
                    "install_status_success": result.status_success,
                    "install_stdout": bounded_text(&result.stdout, 4000),
                    "install_stderr": bounded_text(&result.stderr, 4000),
                })))
            }
        }
    }

    async fn ensure_ready(&self) -> Result<Option<String>> {
        chrome_status().await.map(|_| None).map_err(|err| {
            anyhow::anyhow!(
                "Chrome browser automation is not ready: {err}. Use browser action='status' with browser='chrome' for diagnostics, or action='setup' with browser='chrome' only when the trusted agent-browser CLI exists but the Chrome runtime is missing."
            )
        })
    }

    async fn execute(
        &self,
        action: &str,
        input: &BrowserInput,
        ctx: &ToolContext,
    ) -> Result<ToolOutput> {
        if action == "provider_command" {
            anyhow::bail!(
                "Chrome provider_command is not supported. The raw agent-browser CLI exposes auth, profile, setup, dashboard, and cross-session lifecycle commands that are intentionally not available through Jcode."
            );
        }
        reject_unsupported_targeting(input)?;
        if input.tab_id.is_some() {
            anyhow::bail!(
                "Chrome uses opaque tab_ref values such as 't1'; tab_id remains Firefox-only."
            );
        }

        let exe = discover_trusted_executable().await?;
        let profile = resolve_profile(input.profile.as_deref()).await?;
        let session = session_name_for_profile(&ctx.session_id, input.profile.as_deref());
        let lock = session_lock(&session).await;
        let _guard = lock.lock().await;
        let runtime = runtime_dir().await?;
        let config = neutral_config(&runtime).await?;
        let globals = global_args(&config, &session, profile.as_deref());

        if action == "screenshot" {
            let output = screenshot(&exe, &globals, input, ctx).await?;
            return Ok(attach_profile_metadata(
                attach_browser_metadata(output, self.id(), "chrome"),
                input.profile.as_deref(),
            ));
        }

        let (argv, stdin, title, sensitive) = map_action(action, input, ctx).await?;
        let stdout_limit = if action == "snapshot" || action == "interactables" {
            MAX_STDOUT_BYTES
        } else {
            1024 * 1024
        };
        let result = run_agent_browser(
            &exe,
            &globals,
            &argv,
            stdin.as_deref(),
            timeout_for(input),
            stdout_limit,
            MAX_STDERR_BYTES,
        )
        .await?;
        let value = parse_agent_browser_output(&result, &sensitive)
            .with_context(|| format!("agent-browser action '{action}' failed"))?;
        let normalized = normalize_action_result(action, value);
        Ok(attach_profile_metadata(
            attach_browser_metadata(
                render_browser_output(action, title, normalized),
                self.id(),
                "chrome",
            ),
            input.profile.as_deref(),
        ))
    }
}

async fn chrome_status() -> Result<(TrustedExecutable, Value)> {
    let exe = discover_trusted_executable().await?;
    let runtime = runtime_dir().await?;
    let config = neutral_config(&runtime).await?;
    let globals = global_args(&config, "jcode-status", None);
    let result = run_agent_browser(
        &exe,
        &globals,
        &[
            "doctor".to_string(),
            "--json".to_string(),
            "--offline".to_string(),
        ],
        None,
        NORMAL_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
    )
    .await?;
    let doctor: Value = serde_json::from_str(result.stdout.trim())
        .with_context(|| "agent-browser doctor did not return JSON")?;
    let fail = doctor
        .pointer("/summary/fail")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let success = doctor
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(result.status_success && fail == 0);
    if !success || fail > 0 || !result.status_success {
        anyhow::bail!("agent-browser doctor reports failing checks");
    }
    Ok((exe, doctor))
}

async fn discover_trusted_executable() -> Result<TrustedExecutable> {
    let explicit = std::env::var_os("JCODE_AGENT_BROWSER_BIN");
    let explicit_override = explicit.is_some();
    let path = if let Some(value) = explicit {
        let p = PathBuf::from(value);
        if !p.is_absolute() {
            anyhow::bail!("JCODE_AGENT_BROWSER_BIN must be an absolute path");
        }
        p
    } else {
        find_on_path("agent-browser")?
    };
    let canonical = tokio::fs::canonicalize(&path)
        .await
        .with_context(|| format!("cannot canonicalize {}", path.display()))?;
    validate_executable_path(&canonical, explicit_override).await?;
    let version_result = run_raw_command(
        &canonical,
        &[],
        &["--version".to_string()],
        None,
        Duration::from_secs(5),
        64 * 1024,
        64 * 1024,
    )
    .await?;
    if !version_result.status_success {
        anyhow::bail!("agent-browser --version failed: {}", version_result.stderr);
    }
    let version = parse_version(version_result.stdout.trim())?;
    if !is_supported_version(&version) {
        anyhow::bail!(
            "agent-browser version {version} is incompatible; supported range is >=0.27.3,<0.35.0"
        );
    }
    let fingerprint = executable_fingerprint(&canonical).await?;
    Ok(TrustedExecutable {
        path: canonical,
        version,
        fingerprint,
    })
}

fn find_on_path(name: &str) -> Result<PathBuf> {
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths) {
        if dir.as_os_str().is_empty() || dir == Path::new(".") || !dir.is_absolute() {
            continue;
        }
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("agent-browser executable was not found on PATH")
}

async fn validate_executable_path(path: &Path, explicit: bool) -> Result<()> {
    let meta = tokio::fs::metadata(path).await?;
    if !meta.is_file() {
        anyhow::bail!("{} is not a regular executable", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        if mode & 0o111 == 0 {
            anyhow::bail!("{} is not executable", path.display());
        }
        if mode & 0o022 != 0 {
            anyhow::bail!("{} is group/world writable", path.display());
        }
    }
    if !explicit {
        let repository = std::env::current_dir()
            .ok()
            .and_then(|p| p.canonicalize().ok())
            .and_then(repository_root);
        if let Some(repository) = repository
            && is_repository_local(path, &repository)
        {
            anyhow::bail!(
                "refusing repository-local agent-browser from PATH: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn repository_root(mut path: PathBuf) -> Option<PathBuf> {
    loop {
        if path.join(".git").exists() {
            return Some(path);
        }
        if !path.pop() {
            return None;
        }
    }
}

fn is_repository_local(executable: &Path, repository: &Path) -> bool {
    executable.starts_with(repository)
}

async fn executable_fingerprint(path: &Path) -> Result<String> {
    let meta = tokio::fs::metadata(path).await?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    Ok(format!("{}:{}:{}", path.display(), meta.len(), modified))
}

fn parse_version(output: &str) -> Result<String> {
    let version = output
        .split_whitespace()
        .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .ok_or_else(|| anyhow::anyhow!("could not parse agent-browser version from '{output}'"))?;
    Ok(version.to_string())
}

fn is_supported_version(version: &str) -> bool {
    let mut nums = version.split('.').filter_map(|p| p.parse::<u64>().ok());
    let Some(major) = nums.next() else {
        return false;
    };
    let Some(minor) = nums.next() else {
        return false;
    };
    let Some(patch) = nums.next() else {
        return false;
    };
    (major, minor, patch) >= MIN_AGENT_BROWSER_VERSION
        && major == 0
        && minor < MAX_AGENT_BROWSER_MINOR_EXCLUSIVE
}

async fn session_lock(session: &str) -> std::sync::Arc<Mutex<()>> {
    let locks = SESSION_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = locks.lock().await;
    guard
        .entry(session.to_string())
        .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
        .clone()
}

pub(super) fn session_name(session_id: &str) -> String {
    let mut readable: String = session_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .collect();
    if readable.is_empty() {
        readable = "session".to_string();
    }
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    let digest = hasher.finalize();
    let suffix = digest[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("jcode-{readable}-{suffix}")
}

pub(super) fn session_name_for_profile(session_id: &str, profile: Option<&str>) -> String {
    match profile {
        Some(profile) => session_name(&format!("{session_id}\0profile:{profile}")),
        None => session_name(session_id),
    }
}

fn validate_profile_name(profile: &str) -> Result<&str> {
    if profile.is_empty()
        || profile.len() > 64
        || !profile
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        anyhow::bail!(
            "browser profile must be a 1-64 character name containing only letters, numbers, '.', '-', or '_'; filesystem paths are not accepted"
        );
    }
    Ok(profile)
}

async fn resolve_profile(profile: Option<&str>) -> Result<Option<String>> {
    let Some(profile) = profile else {
        return Ok(None);
    };
    let profile = validate_profile_name(profile)?;
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    if let Some(data_home) = data_home {
        let custom = data_home.join("agent-browser/profiles").join(profile);
        if tokio::fs::metadata(&custom)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
        {
            return Ok(Some(
                tokio::fs::canonicalize(&custom)
                    .await?
                    .display()
                    .to_string(),
            ));
        }
    }
    Ok(Some(profile.to_string()))
}

async fn runtime_dir() -> Result<PathBuf> {
    let base = std::env::var_os("JCODE_AGENT_BROWSER_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("jcode-agent-browser"));
    tokio::fs::create_dir_all(&base).await?;
    Ok(base)
}

async fn neutral_config(runtime: &Path) -> Result<PathBuf> {
    let path = runtime.join("agent-browser-jcode.json");
    tokio::fs::write(
        &path,
        b"{\n  \"profile\": null,\n  \"state\": null,\n  \"autoConnect\": false,\n  \"engine\": \"chrome\"\n}\n",
    )
    .await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok(path)
}

fn global_args(config: &Path, session: &str, profile: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "--config".into(),
        config.display().to_string(),
        "--session".into(),
        session.to_string(),
        "--engine".into(),
        "chrome".into(),
        "--json".into(),
    ];
    if let Some(profile) = profile {
        args.push("--profile".into());
        args.push(profile.to_string());
    }
    args
}

fn attach_profile_metadata(mut output: ToolOutput, profile: Option<&str>) -> ToolOutput {
    if let Some(profile) = profile {
        add_metadata_field(&mut output, "profile", json!(profile));
        add_metadata_field(&mut output, "credential_bearing_profile", json!(true));
    }
    output
}

#[cfg(test)]
pub(super) async fn close_live_session(session_id: &str, profile: Option<&str>) -> Result<()> {
    let exe = discover_trusted_executable().await?;
    let runtime = runtime_dir().await?;
    let config = neutral_config(&runtime).await?;
    let profile_arg = resolve_profile(profile).await?;
    let session = session_name_for_profile(session_id, profile);
    let globals = global_args(&config, &session, profile_arg.as_deref());
    let result = run_agent_browser(
        &exe,
        &globals,
        &["close".into(), "--all".into()],
        None,
        NORMAL_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
    )
    .await?;
    if !result.status_success {
        anyhow::bail!(
            "failed to close live agent-browser session: {}",
            result.stderr
        );
    }
    Ok(())
}

fn timeout_for(input: &BrowserInput) -> Duration {
    let ms = input.timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000);
    Duration::from_millis(ms)
}

async fn map_action(
    action: &str,
    input: &BrowserInput,
    ctx: &ToolContext,
) -> Result<(Vec<String>, Option<Vec<u8>>, String, Vec<String>)> {
    let mut sensitive = sensitive_values(input);
    let title = format!("browser {action}");
    let batch = |commands: Vec<Vec<String>>, sensitive: Vec<String>| -> Result<_> {
        let bytes = serde_json::to_vec(&commands)?;
        if bytes.len() > MAX_STDIN_BYTES {
            anyhow::bail!("agent-browser batch stdin exceeds limit");
        }
        Ok((
            vec!["batch".into(), "--bail".into(), "--json".into()],
            Some(bytes),
            title.clone(),
            sensitive,
        ))
    };

    match action {
        "list_tabs" => Ok((vec!["tab".into(), "list".into()], None, title, sensitive)),
        "new_tab" => {
            let mut cmd = vec!["tab".into(), "new".into()];
            if let Some(url) = &input.url {
                cmd.push(url.clone());
            }
            Ok((cmd, None, title, sensitive))
        }
        "select_tab" => {
            let tab = input
                .tab_ref
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("tab_ref is required for Chrome select_tab"))?;
            Ok((vec!["tab".into(), tab.into()], None, title, sensitive))
        }
        "get_active_tab" => Ok((vec!["tab".into(), "list".into()], None, title, sensitive)),
        "open" => {
            let url = input
                .url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("url is required for open"))?;
            if input.new_tab.unwrap_or(false) {
                batch(
                    vec![vec!["tab".into(), "new".into(), url.into()]],
                    sensitive,
                )
            } else {
                batch(vec![vec!["open".into(), url.into()]], sensitive)
            }
        }
        "snapshot" => Ok((vec!["snapshot".into()], None, title, sensitive)),
        "interactables" => Ok((vec!["snapshot".into(), "-i".into()], None, title, sensitive)),
        "get_content" => {
            let what = match input.format.as_deref().unwrap_or("text") {
                "html" => "html",
                "title" => "title",
                _ => "text",
            };
            Ok((vec!["get".into(), what.into()], None, title, sensitive))
        }
        "click" => {
            let cmd = if let (Some(x), Some(y)) = (input.x, input.y) {
                vec![
                    vec!["mouse".into(), "move".into(), x.to_string(), y.to_string()],
                    vec!["mouse".into(), "down".into()],
                    vec!["mouse".into(), "up".into()],
                ]
            } else if let Some(selector) = input.selector.as_deref().or(input.text.as_deref()) {
                vec![vec!["click".into(), selector.into()]]
            } else {
                anyhow::bail!("click requires selector, text, or x/y coordinates");
            };
            batch(cmd, sensitive)
        }
        "type" => {
            let text = input
                .text
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("text is required for type"))?;
            let mut commands = Vec::new();
            if let Some(selector) = &input.selector {
                if input.clear.unwrap_or(false) {
                    commands.push(vec!["fill".into(), selector.clone(), text.into()]);
                } else {
                    commands.push(vec!["type".into(), selector.clone(), text.into()]);
                }
            } else {
                commands.push(vec!["keyboard".into(), "type".into(), text.into()]);
            }
            if input.submit.unwrap_or(false) {
                commands.push(vec!["press".into(), "Enter".into()]);
            }
            batch(commands, sensitive)
        }
        "fill_form" => {
            let fields = input
                .fields
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("fields are required for fill_form"))?;
            let mut commands = Vec::new();
            for field in fields {
                if let Some(checked) = field.checked {
                    commands.push(vec![
                        if checked { "check" } else { "uncheck" }.into(),
                        field.selector.clone(),
                    ]);
                }
                if let Some(value) = &field.value {
                    commands.push(vec!["fill".into(), field.selector.clone(), value.clone()]);
                }
            }
            if input.submit.unwrap_or(false) {
                commands.push(vec!["press".into(), "Enter".into()]);
            }
            batch(commands, sensitive)
        }
        "select" => {
            let selector = input
                .selector
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("selector is required for select"))?;
            let value = input
                .text
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("text is required for select"))?;
            batch(
                vec![vec!["select".into(), selector.into(), value.into()]],
                sensitive,
            )
        }
        "wait" => {
            if let Some(selector) = input
                .selector
                .as_deref()
                .or(input.text.as_deref())
                .or(input.contains.as_deref())
            {
                Ok((vec!["wait".into(), selector.into()], None, title, sensitive))
            } else if let Some(ms) = input.timeout_ms {
                Ok((vec!["wait".into(), ms.to_string()], None, title, sensitive))
            } else {
                anyhow::bail!("wait requires selector, text, contains, or timeout_ms");
            }
        }
        "eval" => {
            let script = input
                .script
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("script is required for eval"))?;
            batch(vec![vec!["eval".into(), script.into()]], sensitive)
        }
        "scroll" => {
            if let Some(selector) = &input.selector {
                batch(
                    vec![vec!["scrollintoview".into(), selector.clone()]],
                    sensitive,
                )
            } else {
                let dy = input.y.unwrap_or(600.0);
                let dx = input.x.unwrap_or(0.0);
                batch(
                    vec![vec![
                        "mouse".into(),
                        "wheel".into(),
                        dy.to_string(),
                        dx.to_string(),
                    ]],
                    sensitive,
                )
            }
        }
        "upload" => {
            let selector = input
                .selector
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("selector is required for upload"))?;
            let path = input
                .path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("path is required for upload"))?;
            let resolved = validate_upload_path(ctx, path).await?;
            sensitive.push(resolved.display().to_string());
            batch(
                vec![vec![
                    "upload".into(),
                    selector.into(),
                    resolved.display().to_string(),
                ]],
                sensitive,
            )
        }
        "press" => {
            let key = input
                .key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("key is required for press"))?;
            let mut commands = Vec::new();
            if let Some(selector) = &input.selector {
                commands.push(vec!["focus".into(), selector.clone()]);
            }
            commands.push(vec!["press".into(), key.into()]);
            batch(commands, sensitive)
        }
        other => anyhow::bail!("Unsupported Chrome browser action: {other}"),
    }
}

fn reject_unsupported_targeting(input: &BrowserInput) -> Result<()> {
    if input.window_id.is_some() || input.frame_id.is_some() || input.all_frames.unwrap_or(false) {
        anyhow::bail!(
            "Chrome agent-browser does not support direct window_id/frame_id/all_frames targeting through Jcode. Use snapshot refs exposed by the Chrome snapshot instead."
        );
    }
    Ok(())
}

fn sensitive_values(input: &BrowserInput) -> Vec<String> {
    let mut values = Vec::new();
    for opt in [
        input.url.as_ref(),
        input.text.as_ref(),
        input.contains.as_ref(),
        input.script.as_ref(),
        input.key.as_ref(),
        input.path.as_ref(),
    ] {
        if let Some(v) = opt
            && !v.is_empty()
        {
            values.push(v.clone());
        }
    }
    if let Some(fields) = &input.fields {
        for field in fields {
            if let Some(value) = &field.value
                && !value.is_empty()
            {
                values.push(value.clone());
            }
        }
    }
    values
}

async fn validate_upload_path(ctx: &ToolContext, raw: &str) -> Result<PathBuf> {
    let path = ctx.resolve_path(Path::new(raw));
    let canonical = tokio::fs::canonicalize(&path)
        .await
        .with_context(|| format!("upload path does not exist: {}", path.display()))?;
    let meta = tokio::fs::symlink_metadata(&canonical).await?;
    if !meta.is_file() {
        anyhow::bail!("upload path is not a regular file: {}", canonical.display());
    }
    Ok(canonical)
}

async fn screenshot(
    exe: &TrustedExecutable,
    globals: &[String],
    input: &BrowserInput,
    ctx: &ToolContext,
) -> Result<ToolOutput> {
    let runtime = runtime_dir().await?;
    let mut hasher = Sha256::new();
    hasher.update(ctx.session_id.as_bytes());
    hasher.update(ctx.tool_call_id.as_bytes());
    let name = hasher.finalize()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let path = runtime.join(format!("screenshot-{name}.png"));
    let _ = tokio::fs::remove_file(&path).await;
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await?;
    drop(file);

    let mut args = vec!["screenshot".to_string()];
    if let Some(selector) = &input.selector {
        args.push(selector.clone());
    }
    args.push(path.display().to_string());
    let result = run_agent_browser(
        exe,
        globals,
        &args,
        None,
        timeout_for(input),
        1024 * 1024,
        MAX_STDERR_BYTES,
    )
    .await;
    let bytes = match result {
        Ok(result) if result.status_success => read_valid_png(&path).await,
        Ok(result) => Err(anyhow::anyhow!(
            "agent-browser screenshot failed: {}",
            bounded_text(&result.stderr, 1000)
        )),
        Err(err) => Err(err),
    };
    let _ = tokio::fs::remove_file(&path).await;
    let bytes = bytes?;
    Ok(ToolOutput::new("Captured Chrome browser screenshot.")
        .with_title("browser screenshot")
        .with_metadata(json!({"saved": true, "bytes": bytes.len()}))
        .with_labeled_image("image/png", STANDARD.encode(bytes), "browser screenshot"))
}

async fn read_valid_png(path: &Path) -> Result<Vec<u8>> {
    let meta = tokio::fs::symlink_metadata(path).await?;
    if !meta.is_file() {
        anyhow::bail!("screenshot path is not a regular file");
    }
    if meta.len() > MAX_SCREENSHOT_BYTES {
        anyhow::bail!("screenshot exceeds {} bytes", MAX_SCREENSHOT_BYTES);
    }
    let bytes = tokio::fs::read(path).await?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        anyhow::bail!("screenshot output is not a PNG file");
    }
    Ok(bytes)
}

fn normalize_action_result(action: &str, value: Value) -> Value {
    match action {
        "get_active_tab" => normalize_active_tab(value),
        "list_tabs" | "new_tab" | "select_tab" => normalize_tabs(value),
        "snapshot" | "interactables" => normalize_snapshot(value),
        "get_content" => normalize_content(value),
        "eval" => normalize_eval(value),
        _ => value,
    }
}

fn normalize_tabs(value: Value) -> Value {
    let mut value = unwrap_data(value);
    add_tab_refs(&mut value);
    value
}

fn normalize_active_tab(value: Value) -> Value {
    let value = normalize_tabs(value);
    if let Some(arr) = value.as_array()
        && let Some(active) = arr
            .iter()
            .find(|v| v.get("active").and_then(Value::as_bool) == Some(true))
    {
        return active.clone();
    }
    value
}

fn normalize_snapshot(value: Value) -> Value {
    let value = unwrap_data(value);
    if value.is_string() {
        json!({"content": value.as_str().unwrap_or_default()})
    } else {
        value
    }
}

fn normalize_content(value: Value) -> Value {
    let value = unwrap_data(value);
    if value.is_string() {
        json!({"content": value.as_str().unwrap_or_default()})
    } else {
        value
    }
}

fn normalize_eval(value: Value) -> Value {
    let value = unwrap_data(value);
    if value.get("result").is_some() {
        value
    } else {
        json!({"result": value})
    }
}

fn unwrap_data(mut value: Value) -> Value {
    if value.get("success").and_then(Value::as_bool) == Some(true)
        && let Some(data) = value.get_mut("data")
    {
        return data.take();
    }
    value
}

fn add_tab_refs(value: &mut Value) {
    match value {
        Value::Array(arr) => {
            for item in arr {
                add_tab_refs(item);
            }
        }
        Value::Object(map) => {
            if let Some(id) = map
                .get("id")
                .or_else(|| map.get("tabId"))
                .and_then(Value::as_str)
                .map(str::to_string)
            {
                map.insert("tab_ref".into(), json!(id));
            }
            for child in map.values_mut() {
                add_tab_refs(child);
            }
        }
        _ => {}
    }
}

fn parse_agent_browser_output(result: &ProcessResult, sensitive: &[String]) -> Result<Value> {
    let stdout = redact_text(&result.stdout, sensitive);
    let stderr = redact_text(&result.stderr, sensitive);
    if !result.status_success {
        anyhow::bail!(safe_error_text(&stderr, &stdout));
    }
    if stdout.trim().is_empty() {
        return Ok(json!({"ok": true}));
    }
    let mut value: Value = serde_json::from_str(stdout.trim()).or_else(|_| {
        Ok::<_, anyhow::Error>(json!({"raw": bounded_text(stdout.trim(), 16 * 1024)}))
    })?;
    scrub_value(&mut value, sensitive);
    if value.get("success").and_then(Value::as_bool) == Some(false) {
        anyhow::bail!(serde_json::to_string(&value).unwrap_or_else(|_| "provider error".into()));
    }
    Ok(value)
}

fn scrub_value(value: &mut Value, sensitive: &[String]) {
    match value {
        Value::String(s) => *s = redact_text(s, sensitive),
        Value::Array(arr) => {
            for item in arr {
                scrub_value(item, sensitive);
            }
        }
        Value::Object(map) => {
            map.remove("command");
            for item in map.values_mut() {
                scrub_value(item, sensitive);
            }
        }
        _ => {}
    }
}

fn redact_text(text: &str, sensitive: &[String]) -> String {
    let mut out = text.to_string();
    for value in sensitive {
        if !value.is_empty() {
            out = out.replace(value, "<redacted>");
        }
    }
    out
}

fn safe_error_text(stderr: &str, stdout: &str) -> String {
    let text = if stderr.is_empty() { stdout } else { stderr };
    bounded_text(text, 4000)
}

fn bounded_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = text.chars().take(max_chars).collect::<String>();
    out.push_str("...<truncated>");
    out
}

fn summarize_doctor(doctor: &Value) -> Value {
    json!({
        "summary": doctor.get("summary").cloned().unwrap_or(Value::Null),
        "success": doctor.get("success").cloned().unwrap_or(Value::Null),
    })
}

async fn run_agent_browser(
    exe: &TrustedExecutable,
    globals: &[String],
    args: &[String],
    stdin: Option<&[u8]>,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<ProcessResult> {
    let current = executable_fingerprint(&exe.path).await?;
    if current != exe.fingerprint {
        anyhow::bail!(
            "trusted agent-browser executable changed after readiness; retry status before executing Chrome actions"
        );
    }
    let mut all_args = globals.to_vec();
    all_args.extend(args.iter().cloned());
    run_raw_command(
        &exe.path,
        &[],
        &all_args,
        stdin,
        timeout,
        stdout_limit,
        stderr_limit,
    )
    .await
}

async fn run_raw_command(
    path: &Path,
    envs: &[(&str, &str)],
    args: &[String],
    stdin: Option<&[u8]>,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<ProcessResult> {
    if let Some(stdin) = stdin
        && stdin.len() > MAX_STDIN_BYTES
    {
        anyhow::bail!("agent-browser stdin exceeds limit");
    }
    let mut command = tokio::process::Command::new(path);
    command.args(args);
    command.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);
    command.current_dir(runtime_dir().await?);
    command.env_clear();
    for (key, value) in safe_base_env() {
        command.env(key, value);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    command.env("AGENT_BROWSER_IDLE_TIMEOUT_MS", IDLE_TIMEOUT_MS);

    let mut child = command.spawn()?;
    if let Some(bytes) = stdin
        && let Some(mut child_stdin) = child.stdin.take()
    {
        child_stdin.write_all(bytes).await?;
    }
    let stdout = child.stdout.take().context("missing child stdout")?;
    let stderr = child.stderr.take().context("missing child stderr")?;
    let stdout_task = tokio::spawn(read_limited(stdout, stdout_limit));
    let stderr_task = tokio::spawn(read_limited(stderr, stderr_limit));
    let status = match tokio::time::timeout(timeout, child.wait()).await {
        Ok(status) => status?,
        Err(_) => {
            let _ = child.kill().await;
            anyhow::bail!(
                "agent-browser command timed out after {}s",
                timeout.as_secs()
            );
        }
    };
    let stdout = stdout_task.await??;
    let stderr = stderr_task.await??;
    Ok(ProcessResult {
        status_success: status.success(),
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
    })
}

async fn read_limited(mut reader: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = vec![0u8; 8192];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            return Ok(out);
        }
        if out.len() + n > limit {
            anyhow::bail!("agent-browser output exceeded {} bytes", limit);
        }
        out.extend_from_slice(&buf[..n]);
    }
}

fn safe_base_env() -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for key in [
        "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "LANG", "LC_ALL", "NO_PROXY",
    ] {
        if let Ok(value) = std::env::var(key)
            && !key.starts_with("AGENT_BROWSER_")
        {
            vars.push((key.to_string(), value));
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_name_uses_hash_to_avoid_sanitized_collisions() {
        let a = session_name("a/b");
        let b = session_name("ab");
        assert_ne!(a, b);
        assert!(a.starts_with("jcode-ab-"));
    }

    #[test]
    fn version_range_accepts_probed_compatible_minors() {
        assert!(is_supported_version("0.27.3"));
        assert!(is_supported_version("0.27.9"));
        assert!(is_supported_version("0.34.0"));
        assert!(!is_supported_version("0.27.2"));
        assert!(!is_supported_version("0.35.0"));
    }

    #[test]
    fn scrub_removes_commands_and_sensitive_values() {
        let mut value = json!({"data": [{"command": ["fill", "#x", "secret"], "echo": "secret"}]});
        scrub_value(&mut value, &["secret".to_string()]);
        assert!(value.to_string().contains("<redacted>"));
        assert!(!value.to_string().contains("secret"));
        assert!(value.pointer("/data/0/command").is_none());
    }
}

#[test]
fn supported_versions_include_current_agent_browser_minor() {
    assert!(is_supported_version("0.27.3"));
    assert!(is_supported_version("0.34.0"));
    assert!(!is_supported_version("0.35.0"));
}

#[test]
fn executable_under_home_is_not_repository_local_when_repo_is_a_sibling() {
    let executable = Path::new("/home/example/.local/bin/agent-browser");
    let repository = Path::new("/home/example/dev/jcode");
    assert!(!is_repository_local(executable, repository));
    assert!(is_repository_local(
        Path::new("/home/example/dev/jcode/node_modules/.bin/agent-browser"),
        repository,
    ));
}

#[test]
fn profile_names_are_allowlisted_and_paths_are_rejected() {
    assert_eq!(validate_profile_name("social").unwrap(), "social");
    assert_eq!(validate_profile_name("Default").unwrap(), "Default");
    assert!(validate_profile_name("../social").is_err());
    assert!(validate_profile_name("/tmp/social").is_err());
    assert!(validate_profile_name("social/profile").is_err());
    assert!(validate_profile_name("social profile").is_err());
}
