//! Shared MCP Server Pool
//!
//! Manages a global pool of MCP server processes that are shared across
//! all jcode sessions. Instead of each session spawning its own set of
//! MCP servers (N sessions × M servers = N×M processes), sessions share
//! a single pool (M processes total).
//!
//! Sessions get lightweight `McpHandle` clones that can send concurrent
//! requests to shared server processes. Request/response correlation by
//! ID ensures no interference between sessions.

use super::client::{McpClient, McpHandle, StderrTail};
use super::protocol::{McpConfig, McpServerConfig, McpToolDef};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify, RwLock};

/// Marker in errors for attempts skipped by backoff/degraded state; callers
/// use it to avoid re-notifying the user.
pub const SUPPRESSED_MARKER: &str = "mcp reconnect suppressed";

const BACKOFF_BASE: Duration = Duration::from_secs(30);
const BACKOFF_CAP: Duration = Duration::from_secs(30 * 60);
/// Consecutive failures after which a server stops auto-reconnecting.
pub const DEGRADED_THRESHOLD: u32 = 5;

/// Reconnect delay after `failures` consecutive failures (30s doubling, 30min cap).
fn backoff_for(failures: u32) -> Duration {
    let exp = failures.saturating_sub(1).min(16);
    BACKOFF_BASE.saturating_mul(1u32 << exp).min(BACKOFF_CAP)
}

/// Why a connect attempt failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureClass {
    SpawnFailed,
    ExitedBeforeInit,
    InitTimeout,
    AuthRequired,
    RemoteHttpError,
    Other,
}

impl FailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpawnFailed => "spawn_failed",
            Self::ExitedBeforeInit => "exited_before_init",
            Self::InitTimeout => "init_timeout",
            Self::AuthRequired => "auth_required",
            Self::RemoteHttpError => "remote_http_error",
            Self::Other => "other",
        }
    }

    pub(crate) fn classify(error: &anyhow::Error) -> Self {
        let msg = format!("{error:#}").to_ascii_lowercase();
        if msg.contains("failed to spawn") {
            Self::SpawnFailed
        } else if msg.contains("exited") || msg.contains("connection closed") {
            Self::ExitedBeforeInit
        } else if msg.contains("timed out") || msg.contains("timeout") {
            Self::InitTimeout
        } else if msg.contains("401")
            || msg.contains("403")
            || msg.contains("unauthorized")
            || msg.contains("oauth")
        {
            Self::AuthRequired
        } else if msg.contains("http") || msg.contains("status") {
            Self::RemoteHttpError
        } else {
            Self::Other
        }
    }
}

pub(crate) fn redacted_stderr(lines: &[String], config: &McpServerConfig) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let mut line = line.clone();
            for secret in config
                .env
                .values()
                .chain(config.headers.values())
                .filter(|value| !value.is_empty())
            {
                line = line.replace(secret, "[REDACTED]");
            }
            crate::message::redact_secrets(&line)
        })
        .collect()
}

#[derive(Clone)]
struct FailedConnectRecord {
    message: String,
    failed_at: Instant,
    failures: u32,
    class: FailureClass,
    stderr_tail: Vec<String>,
    /// Config the failures were observed with; a change resets backoff.
    config_fingerprint: String,
}

impl FailedConnectRecord {
    fn degraded(&self) -> bool {
        self.failures >= DEGRADED_THRESHOLD
    }

    fn retry_after(&self) -> Duration {
        backoff_for(self.failures).saturating_sub(self.failed_at.elapsed())
    }
}

fn config_fingerprint(config: &McpServerConfig) -> String {
    serde_json::to_string(config).unwrap_or_default()
}

enum ConnectAttempt {
    Connected,
    Leader(Arc<Notify>),
    Wait(Arc<Notify>),
}

/// Global shared pool of MCP server processes.
///
/// Only one pool exists per jcode daemon. It owns the child processes
/// and hands out cheap `McpHandle` clones to sessions.
pub struct SharedMcpPool {
    clients: Mutex<HashMap<String, McpClient>>,
    handles: RwLock<HashMap<String, McpHandle>>,
    config: RwLock<McpConfig>,
    /// Directory against which the pool's default config was resolved. Keep it
    /// stable across reloads because the daemon may serve sessions in many dirs.
    config_dir: Option<std::path::PathBuf>,
    ref_counts: Mutex<HashMap<String, usize>>,
    connecting: Mutex<HashMap<String, Arc<Notify>>>,
    last_errors: RwLock<HashMap<String, FailedConnectRecord>>,
}

impl SharedMcpPool {
    /// Create a new shared pool with the given config
    pub fn new(config: McpConfig) -> Self {
        Self::new_for_dir(config, std::env::current_dir().ok())
    }

    fn new_for_dir(config: McpConfig, config_dir: Option<std::path::PathBuf>) -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            handles: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            config_dir,
            ref_counts: Mutex::new(HashMap::new()),
            connecting: Mutex::new(HashMap::new()),
            last_errors: RwLock::new(HashMap::new()),
        }
    }

    /// Create pool loading config from default locations
    pub fn from_default_config() -> Self {
        let config_dir = std::env::current_dir().ok();
        let config = McpConfig::load_for_dir(config_dir.as_deref());
        Self::new_for_dir(config, config_dir)
    }

    /// Connect to all configured servers.
    /// Returns (successes, failures).
    pub async fn connect_all(&self) -> (usize, Vec<(String, String)>) {
        let config = self.config.read().await;
        let mut connect_futures = Vec::new();

        for (name, server_config) in &config.servers {
            // Disabled servers stay configured but are never auto-spawned
            // (issue #436); they can still be connected on demand by name.
            if !server_config.autostarts(self.config_dir.as_deref()) {
                continue;
            }
            // Non-shared servers are owned per-session (with the session cwd)
            // and must never be spawned in the daemon-global pool (issue #557).
            if !server_config.shared {
                continue;
            }
            let name = name.clone();
            let server_config = server_config.clone();
            connect_futures.push(async move {
                let result = self.ensure_connected(name.clone(), server_config).await;
                (name, result)
            });
        }
        drop(config);

        let mut successes = 0;
        let mut failures = Vec::new();

        for (name, result) in futures::future::join_all(connect_futures).await {
            match result {
                Ok(new_connection) => {
                    if new_connection {
                        successes += 1;
                    }
                }
                Err(error_msg) => {
                    crate::logging::error(&format!(
                        "Failed to connect to MCP server '{}': {}",
                        name, error_msg
                    ));
                    failures.push((name, error_msg));
                }
            }
        }

        if successes == 0 {
            successes = self
                .handles
                .read()
                .await
                .values()
                .filter(|handle| !handle.is_closed())
                .count();
        }

        (successes, failures)
    }

    /// Connect to a specific server by name and config
    pub async fn connect_server(&self, name: &str, config: &McpServerConfig) -> Result<()> {
        self.ensure_connected(name.to_string(), config.clone())
            .await
            .map(|_| ())
            .map_err(|error_msg| anyhow::anyhow!(error_msg))
            .with_context(|| format!("Failed to connect to MCP server '{}'", name))
    }

    /// Forget failure history for `name` so the next connect is attempted
    /// immediately, clearing backoff and `degraded` state. Used by explicit
    /// user connects.
    pub async fn reset_failures(&self, name: &str) {
        self.last_errors.write().await.remove(name);
    }

    /// Disconnect a specific server
    pub async fn disconnect_server(&self, name: &str) {
        {
            let mut handles = self.handles.write().await;
            handles.remove(name);
        }
        {
            let mut clients = self.clients.lock().await;
            if let Some(mut client) = clients.remove(name) {
                client.shutdown().await;
            }
        }
        {
            let mut refs = self.ref_counts.lock().await;
            refs.remove(name);
        }
        {
            let mut errors = self.last_errors.write().await;
            errors.remove(name);
        }
    }

    /// Disconnect all servers
    pub async fn disconnect_all(&self) {
        {
            let mut handles = self.handles.write().await;
            handles.clear();
        }
        {
            let mut clients = self.clients.lock().await;
            for (_, mut client) in clients.drain() {
                client.shutdown().await;
            }
        }
        {
            let mut refs = self.ref_counts.lock().await;
            refs.clear();
        }
        {
            let mut errors = self.last_errors.write().await;
            errors.clear();
        }
    }

    /// Get handles for all connected servers (for a new session).
    /// Increments reference counts.
    pub async fn acquire_handles(&self, session_id: &str) -> HashMap<String, McpHandle> {
        let handles = self.handles.read().await;
        let result: HashMap<String, McpHandle> = handles
            .iter()
            .filter(|(_, handle)| !handle.is_closed())
            .map(|(name, handle)| (name.clone(), handle.clone()))
            .collect();

        let mut refs = self.ref_counts.lock().await;
        for name in result.keys() {
            *refs.entry(name.clone()).or_insert(0) += 1;
        }

        if !result.is_empty() {
            crate::logging::info(&format!(
                "MCP pool: session '{}' acquired {} server handle(s)",
                session_id,
                result.len()
            ));
        }

        result
    }

    /// Release handles when a session disconnects.
    /// Decrements reference counts.
    pub async fn release_handles(&self, session_id: &str, server_names: &[String]) {
        let mut refs = self.ref_counts.lock().await;
        for name in server_names {
            if let Some(count) = refs.get_mut(name) {
                *count = count.saturating_sub(1);
            }
        }

        if !server_names.is_empty() {
            crate::logging::info(&format!(
                "MCP pool: session '{}' released {} server handle(s)",
                session_id,
                server_names.len()
            ));
        }
    }

    /// Get a handle for a specific server
    pub async fn get_handle(&self, name: &str) -> Option<McpHandle> {
        let handles = self.handles.read().await;
        handles
            .get(name)
            .filter(|handle| !handle.is_closed())
            .cloned()
    }

    /// Get all available tools from all connected servers
    pub async fn all_tools(&self) -> Vec<(String, McpToolDef)> {
        let handles = self.handles.read().await;
        let mut tools = Vec::new();
        for (server_name, handle) in handles.iter() {
            if handle.is_closed() {
                continue;
            }
            for tool in handle.tools() {
                tools.push((server_name.clone(), tool));
            }
        }
        tools
    }

    /// Get list of connected server names
    pub async fn connected_servers(&self) -> Vec<String> {
        let handles = self.handles.read().await;
        handles
            .iter()
            .filter(|(_, handle)| !handle.is_closed())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: serde_json::Value,
    ) -> Result<super::protocol::ToolCallResult> {
        let handles = self.handles.read().await;
        let handle = handles
            .get(server)
            .filter(|handle| !handle.is_closed())
            .with_context(|| format!("MCP server '{}' not connected", server))?;
        handle.call_tool(tool, arguments).await
    }

    /// Reload config and reconnect all servers
    pub async fn reload(&self) -> (usize, Vec<(String, String)>) {
        self.disconnect_all().await;
        *self.config.write().await = McpConfig::load_for_dir(self.config_dir.as_deref());
        self.connect_all().await
    }

    /// Get current config
    pub async fn config(&self) -> McpConfig {
        self.config.read().await.clone()
    }

    /// Check if any servers are connected
    pub async fn has_connections(&self) -> bool {
        let handles = self.handles.read().await;
        handles.values().any(|handle| !handle.is_closed())
    }

    /// Get reference counts (for debugging)
    pub async fn ref_counts(&self) -> HashMap<String, usize> {
        self.ref_counts.lock().await.clone()
    }

    async fn begin_connect(&self, name: &str) -> ConnectAttempt {
        {
            let connecting = self.connecting.lock().await;
            if let Some(notify) = connecting.get(name) {
                return ConnectAttempt::Wait(Arc::clone(notify));
            }
        }

        // Acquire maps before reserving leadership so cancellation at a lock
        // wait cannot strand a reservation or retain global coordination.
        let mut clients = self.clients.lock().await;
        let mut handles = self.handles.write().await;
        let mut connecting = self.connecting.lock().await;
        if let Some(notify) = connecting.get(name) {
            return ConnectAttempt::Wait(Arc::clone(notify));
        }
        if handles.get(name).is_some_and(|handle| !handle.is_closed()) {
            return ConnectAttempt::Connected;
        }
        handles.remove(name);
        // Preserve reference counts held by existing sessions, but release the
        // dead transport before starting a fresh initialization handshake.
        clients.remove(name);

        let notify = Arc::new(Notify::new());
        connecting.insert(name.to_string(), Arc::clone(&notify));
        ConnectAttempt::Leader(notify)
    }

    async fn finish_connect(
        &self,
        name: &str,
        notify: Arc<Notify>,
        result: Result<McpClient>,
        config: &McpServerConfig,
    ) {
        match result {
            Ok(client) => {
                let handle = client.handle();
                {
                    let mut handles = self.handles.write().await;
                    handles.insert(name.to_string(), handle);
                }
                {
                    let mut clients = self.clients.lock().await;
                    clients.insert(name.to_string(), client);
                }
                if self.last_errors.write().await.remove(name).is_some() {
                    crate::logging::info(&format!("MCP: '{name}' recovered after failures"));
                }
            }
            Err(error) => {
                let fingerprint = config_fingerprint(config);
                let mut errors = self.last_errors.write().await;
                let previous = errors
                    .get(name)
                    .filter(|r| r.config_fingerprint == fingerprint)
                    .map_or(0, |r| r.failures);
                let record = FailedConnectRecord {
                    message: format!("{:#}", error),
                    failed_at: Instant::now(),
                    failures: previous + 1,
                    class: FailureClass::classify(&error),
                    stderr_tail: error
                        .downcast_ref::<StderrTail>()
                        .map(|t| t.0.clone())
                        .unwrap_or_default(),
                    config_fingerprint: fingerprint,
                };
                // One notice per health transition: first failure and degrade.
                if record.failures == 1 || record.failures == DEGRADED_THRESHOLD {
                    let state = if record.degraded() {
                        "degraded"
                    } else {
                        "failing"
                    };
                    crate::logging::event_rate_limited(
                        crate::logging::LogLevel::Error,
                        &format!("mcp_health:{name}:{state}"),
                        Duration::from_secs(1),
                        "MCP_SERVER_HEALTH",
                        vec![
                            ("server", name.to_string()),
                            ("state", state.to_string()),
                            ("class", record.class.as_str().to_string()),
                            ("failures", record.failures.to_string()),
                            ("error", record.message.clone()),
                            ("stderr_tail", record.stderr_tail.join("\n")),
                        ],
                    );
                }
                errors.insert(name.to_string(), record);
            }
        }

        {
            let mut connecting = self.connecting.lock().await;
            if connecting
                .get(name)
                .map(|current| Arc::ptr_eq(current, &notify))
                .unwrap_or(false)
            {
                connecting.remove(name);
            }
        }

        notify.notify_waiters();
    }

    async fn ensure_connected(
        &self,
        name: String,
        config: McpServerConfig,
    ) -> std::result::Result<bool, String> {
        if let Some(record) = self.recent_failure(&name, &config).await {
            if record.degraded() {
                return Err(format!(
                    "{} [{}] ({}: degraded after {} consecutive failures; run `mcp connect {}` or reload MCP to retry)",
                    record.message,
                    record.class.as_str(),
                    SUPPRESSED_MARKER,
                    record.failures,
                    name
                ));
            }
            let retry_after = record.retry_after().as_secs().max(1);
            crate::logging::debug(&format!(
                "MCP: Skipping reconnect to '{}' for {}s after {} failure(s)",
                name, retry_after, record.failures
            ));
            return Err(format!(
                "{} [{}] ({}: retry suppressed for ~{}s after {} failure(s))",
                record.message,
                record.class.as_str(),
                SUPPRESSED_MARKER,
                retry_after,
                record.failures
            ));
        }

        match self.begin_connect(&name).await {
            ConnectAttempt::Connected => Ok(false),
            ConnectAttempt::Wait(notify) => {
                self.wait_for_connect(&name, &notify).await;
                if self
                    .handles
                    .read()
                    .await
                    .get(&name)
                    .is_some_and(|handle| !handle.is_closed())
                {
                    Ok(false)
                } else {
                    let error = self
                        .last_errors
                        .read()
                        .await
                        .get(&name)
                        .map(|record| record.message.clone())
                        .unwrap_or_else(|| {
                            "Connection attempt did not produce a handle".to_string()
                        });
                    Err(error)
                }
            }
            ConnectAttempt::Leader(notify) => {
                let result = McpClient::connect(name.clone(), &config).await;
                let outcome = match &result {
                    Ok(_) => Ok(true),
                    Err(error) => Err(format!("{:#}", error)),
                };
                self.finish_connect(&name, notify, result, &config).await;
                outcome
            }
        }
    }

    async fn wait_for_connect(&self, name: &str, notify: &Arc<Notify>) {
        let notified = notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        // Registration must precede this check because notify_waiters does
        // not retain a permit when the leader finishes before we start waiting.
        if self
            .connecting
            .lock()
            .await
            .get(name)
            .is_some_and(|current| Arc::ptr_eq(current, notify))
        {
            notified.await;
        }
    }

    pub async fn health_snapshot(
        &self,
        config: &McpConfig,
    ) -> Vec<jcode_protocol::McpServerHealth> {
        let connecting: Vec<_> = self.connecting.lock().await.keys().cloned().collect();
        let connected: Vec<_> = self
            .handles
            .read()
            .await
            .iter()
            .filter(|(_, handle)| !handle.is_closed())
            .map(|(name, _)| name.clone())
            .collect();
        let errors = self.last_errors.read().await.clone();
        let mut names: Vec<_> = config.servers.keys().cloned().collect();
        names.sort();
        names.dedup();
        names
            .into_iter()
            .map(|name| {
                let cfg = config.servers.get(&name);
                let connected = connected.contains(&name);
                let failure = errors.get(&name).filter(|record| {
                    !connected
                        && cfg
                            .is_some_and(|cfg| record.config_fingerprint == config_fingerprint(cfg))
                });
                let state = if cfg.is_some_and(|cfg| !cfg.is_enabled()) {
                    "disabled"
                } else if connected {
                    "connected"
                } else if failure.is_some_and(FailedConnectRecord::degraded) {
                    "degraded"
                } else if failure.is_some() {
                    "backoff"
                } else if connecting.contains(&name) {
                    "connecting"
                } else if cfg.is_some_and(McpServerConfig::is_lazy) {
                    "lazy"
                } else {
                    "not connected"
                };
                let stderr_tail = failure
                    .zip(cfg)
                    .map(|(failure, cfg)| redacted_stderr(&failure.stderr_tail, cfg))
                    .unwrap_or_default();
                jcode_protocol::McpServerHealth {
                    name,
                    state: state.into(),
                    failure_class: failure.map(|failure| failure.class.as_str().into()),
                    stderr_tail,
                }
            })
            .collect()
    }

    async fn recent_failure(
        &self,
        name: &str,
        config: &McpServerConfig,
    ) -> Option<FailedConnectRecord> {
        if self
            .handles
            .read()
            .await
            .get(name)
            .is_some_and(|handle| !handle.is_closed())
        {
            return None;
        }

        self.last_errors
            .read()
            .await
            .get(name)
            .filter(|record| {
                record.config_fingerprint == config_fingerprint(config)
                    && (record.degraded() || !record.retry_after().is_zero())
            })
            .cloned()
    }
}

/// Global pool singleton
static SHARED_POOL: tokio::sync::OnceCell<Arc<SharedMcpPool>> = tokio::sync::OnceCell::const_new();

/// Initialize the global shared MCP pool. Call once at daemon startup.
pub async fn init_shared_pool() -> Arc<SharedMcpPool> {
    SHARED_POOL
        .get_or_init(|| async {
            let pool = SharedMcpPool::from_default_config();
            Arc::new(pool)
        })
        .await
        .clone()
}

/// Get the global shared pool, if initialized.
pub fn get_shared_pool() -> Option<Arc<SharedMcpPool>> {
    SHARED_POOL.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::McpConfig;

    #[tokio::test]
    async fn health_snapshot_reports_degraded_failure_without_config_secrets() {
        let config: McpConfig = serde_json::from_value(serde_json::json!({"mcpServers": {
            "broken": {"command": "missing", "env": {"CUSTOM_SECRET": "fixture-secret-value"}}
        }}))
        .unwrap();
        let fingerprint = config_fingerprint(&config.servers["broken"]);
        let pool = SharedMcpPool::new(config.clone());
        pool.last_errors.write().await.insert(
            "broken".into(),
            FailedConnectRecord {
                message: "missing".into(),
                failed_at: Instant::now(),
                failures: DEGRADED_THRESHOLD,
                class: FailureClass::SpawnFailed,
                stderr_tail: vec!["failed fixture-secret-value".into()],
                config_fingerprint: fingerprint,
            },
        );
        let record = pool.last_errors.read().await["broken"].clone();
        pool.last_errors
            .write()
            .await
            .insert("other-workspace".into(), record);
        let health = pool.health_snapshot(&config).await;
        assert_eq!(health.len(), 1);
        assert_eq!(health[0].state, "degraded");
        assert_eq!(health[0].failure_class.as_deref(), Some("spawn_failed"));
        assert_eq!(health[0].stderr_tail, ["failed [REDACTED]"]);
    }

    #[tokio::test]
    async fn issue_790_reload_reuses_default_config_directory() {
        let _guard = crate::storage::lock_test_env();
        let original_cwd = std::env::current_dir().expect("current cwd");
        let previous_home = std::env::var_os("JCODE_HOME");
        let home = tempfile::tempdir().expect("home tempdir");
        let first_project = tempfile::tempdir().expect("first project tempdir");
        let second_project = tempfile::tempdir().expect("second project tempdir");
        crate::env::set_var("JCODE_HOME", home.path());
        std::fs::write(
            first_project.path().join(".mcp.json"),
            r#"{"mcpServers":{"first":{"command":"first-server","shared":false}}}"#,
        )
        .expect("write first project config");
        std::fs::write(
            second_project.path().join(".mcp.json"),
            r#"{"mcpServers":{"second":{"command":"second-server","shared":false}}}"#,
        )
        .expect("write second project config");

        std::env::set_current_dir(first_project.path()).expect("set first project cwd");
        let pool = SharedMcpPool::from_default_config();
        let initially_loaded_first = pool.config().await.servers.contains_key("first");

        std::fs::write(
            first_project.path().join(".mcp.json"),
            r#"{"mcpServers":{"first-reloaded":{"command":"first-reloaded-server","shared":false}}}"#,
        )
        .expect("update first project config");

        std::env::set_current_dir(second_project.path()).expect("set second project cwd");
        let _ = pool.reload().await;
        let reloaded = pool.config().await;

        std::env::set_current_dir(original_cwd).expect("restore cwd");
        if let Some(previous_home) = previous_home {
            crate::env::set_var("JCODE_HOME", previous_home);
        } else {
            crate::env::remove_var("JCODE_HOME");
        }

        assert!(initially_loaded_first);
        assert!(!reloaded.servers.contains_key("first"));
        assert!(reloaded.servers.contains_key("first-reloaded"));
        assert!(!reloaded.servers.contains_key("second"));
    }

    fn test_config(command: &str) -> McpServerConfig {
        serde_json::from_value(serde_json::json!({ "command": command })).expect("config")
    }

    async fn fail(pool: &SharedMcpPool, name: &str, config: &McpServerConfig) {
        let ConnectAttempt::Leader(n) = pool.begin_connect(name).await else {
            panic!("expected leader");
        };
        pool.finish_connect(name, n, Err(anyhow::anyhow!("MCP server exited")), config)
            .await;
    }

    #[test]
    fn backoff_grows_and_caps() {
        assert_eq!(backoff_for(1), Duration::from_secs(30));
        assert_eq!(backoff_for(2), Duration::from_secs(60));
        assert_eq!(backoff_for(3), Duration::from_secs(120));
        assert_eq!(backoff_for(10), BACKOFF_CAP);
        assert_eq!(backoff_for(u32::MAX), BACKOFF_CAP);
    }

    #[tokio::test]
    async fn three_failures_suppress_for_at_least_120s() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = test_config("/nonexistent/jcode-mcp-bin");
        for _ in 0..3 {
            fail(&pool, "s", &config).await;
        }
        let record = pool.recent_failure("s", &config).await.expect("suppressed");
        assert_eq!(record.failures, 3);
        assert!(record.retry_after() > Duration::from_secs(110));
        let err = pool.connect_server("s", &config).await.unwrap_err();
        assert!(format!("{err:#}").contains(SUPPRESSED_MARKER));
    }

    #[tokio::test]
    async fn config_change_resets_backoff() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let old = test_config("old");
        for _ in 0..DEGRADED_THRESHOLD {
            fail(&pool, "s", &old).await;
        }
        let new = test_config("new");
        assert!(pool.recent_failure("s", &new).await.is_none());
        fail(&pool, "s", &new).await;
        assert_eq!(pool.last_errors.read().await["s"].failures, 1);
    }

    #[tokio::test]
    async fn degraded_after_threshold_and_reset_clears() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = test_config("/nonexistent/jcode-mcp-bin");
        for _ in 0..DEGRADED_THRESHOLD {
            fail(&pool, "s", &config).await;
        }
        let record = pool.recent_failure("s", &config).await.expect("degraded");
        assert!(record.degraded());
        assert_eq!(record.class, FailureClass::ExitedBeforeInit);
        // Even once the backoff window elapses, a degraded server stays off.
        pool.last_errors
            .write()
            .await
            .get_mut("s")
            .unwrap()
            .failed_at = Instant::now() - BACKOFF_CAP - Duration::from_secs(1);
        let err = format!("{:#}", pool.connect_server("s", &config).await.unwrap_err());
        assert!(
            err.contains("degraded") && err.contains("mcp connect s"),
            "{err}"
        );
        pool.reset_failures("s").await;
        assert!(pool.recent_failure("s", &config).await.is_none());
    }

    #[tokio::test]
    async fn successful_connect_resets_failures() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = crate::mcp::client::tests::fake_server_config();
        fail(&pool, "s", &config).await;
        pool.reset_failures("s").await; // simulate explicit connect
        pool.connect_server("s", &config).await.expect("connects");
        assert!(pool.last_errors.read().await.get("s").is_none());
        assert!(pool.connected_servers().await.contains(&"s".to_string()));
        pool.disconnect_all().await;
    }

    #[tokio::test]
    async fn many_sessions_share_one_attempt_while_backed_off() {
        // Ten sessions requesting a failing server: one spawn/notice, rest suppressed.
        let pool = Arc::new(SharedMcpPool::new(McpConfig::default()));
        let config = test_config("/nonexistent/jcode-mcp-bin");
        let mut tasks = Vec::new();
        for _ in 0..10 {
            let (pool, config) = (Arc::clone(&pool), config.clone());
            tasks.push(tokio::spawn(async move {
                pool.connect_server("s", &config).await
            }));
        }
        for t in tasks {
            assert!(t.await.unwrap().is_err());
        }
        let record = pool.last_errors.read().await["s"].clone();
        assert_eq!(record.failures, 1, "only one real attempt");
        assert_eq!(record.class, FailureClass::SpawnFailed);
    }

    #[tokio::test]
    async fn begin_connect_map_waits_release_coordination_and_cancel_cleanly() {
        for block_handles in [true, false] {
            let pool = SharedMcpPool::new(McpConfig::default());
            let handles = if block_handles {
                Some(pool.handles.write().await)
            } else {
                None
            };
            let clients = if block_handles {
                None
            } else {
                Some(pool.clients.lock().await)
            };
            {
                let attempt = pool.begin_connect("blocked");
                tokio::pin!(attempt);
                assert!(futures::poll!(attempt.as_mut()).is_pending());
                assert!(
                    pool.connecting.try_lock().is_ok(),
                    "map wait must not retain global connection coordination"
                );
            }
            assert!(pool.connecting.lock().await.is_empty());
            assert!(pool.handles.try_read().is_ok() || block_handles);
            drop(handles);
            drop(clients);
            assert!(matches!(
                pool.begin_connect("blocked").await,
                ConnectAttempt::Leader(_)
            ));
        }
    }

    #[tokio::test]
    async fn begin_connect_cancellation_waiting_for_coordination_leaves_no_reservation() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let connecting = pool.connecting.lock().await;
        {
            let attempt = pool.begin_connect("cancelled");
            tokio::pin!(attempt);
            assert!(futures::poll!(attempt.as_mut()).is_pending());
        }
        assert!(connecting.is_empty());
        drop(connecting);
        assert!(pool.handles.try_write().is_ok());
        assert!(pool.clients.try_lock().is_ok());
        assert!(matches!(
            pool.begin_connect("cancelled").await,
            ConnectAttempt::Leader(_)
        ));
    }

    #[tokio::test]
    async fn begin_connect_cancellation_at_leadership_recheck_releases_maps() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let clients = pool.clients.lock().await;
        let mut attempt = Box::pin(pool.begin_connect("cancelled"));
        assert!(futures::poll!(attempt.as_mut()).is_pending());
        let connecting = pool.connecting.lock().await;
        drop(clients);
        assert!(futures::poll!(attempt.as_mut()).is_pending());
        assert!(connecting.is_empty());
        drop(attempt);
        assert!(pool.clients.try_lock().is_ok());
        assert!(pool.handles.try_write().is_ok());
        drop(connecting);
        assert!(matches!(
            pool.begin_connect("cancelled").await,
            ConnectAttempt::Leader(_)
        ));
    }

    #[tokio::test]
    async fn health_snapshot_releases_handles_before_waiting_for_errors() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = McpConfig::default();
        let errors = pool.last_errors.write().await;
        {
            let snapshot = pool.health_snapshot(&config);
            tokio::pin!(snapshot);
            assert!(futures::poll!(snapshot.as_mut()).is_pending());
            assert!(
                pool.handles.try_write().is_ok(),
                "health must release handle snapshot before waiting for failures"
            );
            assert!(pool.connecting.try_lock().is_ok());
        }
        drop(errors);
    }

    #[tokio::test]
    async fn concurrent_connected_attempts_do_not_reserve_or_replace_transport() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = crate::mcp::client::tests::fake_server_config();
        pool.connect_server("connected", &config).await.unwrap();
        let (first, second) = tokio::join!(
            pool.begin_connect("connected"),
            pool.begin_connect("connected")
        );
        assert!(matches!(first, ConnectAttempt::Connected));
        assert!(matches!(second, ConnectAttempt::Connected));
        assert!(pool.connecting.lock().await.is_empty());
        assert_eq!(pool.clients.lock().await.len(), 1);
        pool.disconnect_all().await;
    }

    #[tokio::test]
    async fn reconnect_waiter_observes_published_failure_after_registration() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let config = test_config("fixture");
        let ConnectAttempt::Leader(leader) = pool.begin_connect("late").await else {
            panic!("leader");
        };
        let ConnectAttempt::Wait(waiter) = pool.begin_connect("late").await else {
            panic!("waiter");
        };
        let waiting = pool.wait_for_connect("late", &waiter);
        tokio::pin!(waiting);
        assert!(futures::poll!(waiting.as_mut()).is_pending());
        pool.finish_connect(
            "late",
            leader,
            Err(anyhow::anyhow!("fixture failure")),
            &config,
        )
        .await;
        tokio::time::timeout(Duration::from_secs(1), waiting)
            .await
            .expect("registered follower must observe completion");
        assert_eq!(
            pool.last_errors.read().await["late"].message,
            "fixture failure"
        );
        assert!(pool.connecting.lock().await.is_empty());
    }

    #[tokio::test]
    async fn begin_connect_deduplicates_concurrent_attempts() {
        let pool = Arc::new(SharedMcpPool::new(McpConfig::default()));

        let handles = pool.handles.write().await;
        let first = pool.begin_connect("demo");
        let second = pool.begin_connect("demo");
        tokio::pin!(first, second);
        assert!(futures::poll!(first.as_mut()).is_pending());
        assert!(futures::poll!(second.as_mut()).is_pending());
        drop(handles);
        let (first, second) = tokio::join!(first, second);

        let first_notify = match first {
            ConnectAttempt::Leader(notify) => notify,
            _ => panic!("first attempt should lead"),
        };
        let second_notify = match second {
            ConnectAttempt::Wait(notify) => notify,
            _ => panic!("second attempt should wait"),
        };

        assert!(Arc::ptr_eq(&first_notify, &second_notify));
    }

    #[tokio::test]
    async fn reconnect_waiter_observes_leader_completion_before_registration() {
        let pool = SharedMcpPool::new(McpConfig::default());
        let ConnectAttempt::Leader(leader) = pool.begin_connect("early").await else {
            panic!("leader");
        };
        let ConnectAttempt::Wait(waiter) = pool.begin_connect("early").await else {
            panic!("waiter");
        };
        pool.finish_connect(
            "early",
            leader,
            Err(anyhow::anyhow!("fixture failure")),
            &test_config("x"),
        )
        .await;
        tokio::time::timeout(
            std::time::Duration::from_millis(100),
            pool.wait_for_connect("early", &waiter),
        )
        .await
        .expect("an already-completed attempt must not wait for a lost notification");
    }

    #[tokio::test]
    async fn connect_all_skips_non_shared_servers() {
        // Issue #557: shared:false servers are owned per-session and must not
        // be spawned in the daemon-global pool. If the pool tried to connect
        // this nonexistent command it would show up as a failure.
        let mut config = McpConfig::default();
        config.servers.insert(
            "owned-only".to_string(),
            crate::mcp::protocol::McpServerConfig {
                source_scope: None,
                command: "/nonexistent/jcode-test-mcp-557".to_string(),
                args: vec![],
                env: Default::default(),
                shared: false,
                transport: None,
                url: None,
                headers: std::collections::HashMap::new(),
                enabled: None,
                disabled: None,
                timeout_secs: None,
                expose: None,
                workspaces: Vec::new(),
                lazy: None,
            },
        );
        let pool = SharedMcpPool::new(config);

        let (successes, failures) = pool.connect_all().await;
        assert_eq!(successes, 0);
        assert!(
            failures.is_empty(),
            "non-shared server must be skipped, got failures: {failures:?}"
        );
    }
}
