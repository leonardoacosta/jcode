#![cfg_attr(test, allow(clippy::await_holding_lock))]

//! Server registry for multi-server architecture
//!
//! Tracks running servers in `~/.jcode/servers.json` for discovery by clients.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

use crate::storage::jcode_dir;

/// Information about a running server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Full server ID (e.g., "server_blazing_1705012345678")
    pub id: String,
    /// Short name (e.g., "blazing")
    pub name: String,
    /// Icon for display (e.g., "🔥")
    pub icon: String,
    /// Socket path
    pub socket: PathBuf,
    /// Debug socket path
    pub debug_socket: PathBuf,
    /// Git hash of the binary
    pub git_hash: String,
    /// Version string (e.g., "v0.1.123")
    pub version: String,
    /// Process ID
    pub pid: u32,
    /// When the server started (ISO 8601)
    pub started_at: String,
    /// Session names currently on this server
    #[serde(default)]
    pub sessions: Vec<String>,
    /// Connected client count last published by the server. `None` for
    /// entries written by older builds, shown as unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clients: Option<usize>,
}

impl ServerInfo {
    /// Display name with icon (e.g., "🔥 blazing")
    pub fn display_name(&self) -> String {
        format!("{} {}", self.icon, self.name)
    }
}

/// The server registry file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerRegistry {
    /// Map from server name to server info
    #[serde(flatten)]
    pub servers: HashMap<String, ServerInfo>,
}

impl ServerRegistry {
    /// Load the registry from disk
    pub async fn load() -> Result<Self> {
        let path = registry_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).await?;
        let registry: Self = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Save the registry to disk
    pub async fn save(&self) -> Result<()> {
        let path = registry_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
            if let Err(e) = crate::platform::set_directory_permissions_owner_only(parent) {
                crate::logging::info(&format!(
                    "Registry save: failed to harden directory permissions for {}: {}",
                    parent.display(),
                    e
                ));
            }
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).await?;
        if let Err(e) = crate::platform::set_permissions_owner_only(&path) {
            crate::logging::info(&format!(
                "Registry save: failed to harden file permissions for {}: {}",
                path.display(),
                e
            ));
        }
        Ok(())
    }

    /// Register a server
    pub fn register(&mut self, info: ServerInfo) {
        self.servers.insert(info.name.clone(), info);
    }

    /// Unregister a server by name
    pub fn unregister(&mut self, name: &str) {
        self.servers.remove(name);
    }

    /// Find a server by name
    pub fn find_by_name(&self, name: &str) -> Option<&ServerInfo> {
        self.servers.get(name)
    }

    /// Get all servers sorted by started_at (newest first)
    pub fn servers_by_time(&self) -> Vec<&ServerInfo> {
        let mut servers: Vec<_> = self.servers.values().collect();
        servers.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        servers
    }

    /// Clean up stale entries (servers that are no longer running or have been superseded).
    ///
    /// Socket path ownership is managed by the server process itself. Registry
    /// cleanup must not unlink those paths because a new live server can reuse
    /// the same published socket after a reboot or reload while an older
    /// registry entry still references it.
    pub async fn cleanup_stale(&mut self) -> Result<Vec<String>> {
        let mut removed = Vec::new();

        // First pass: remove entries whose process is dead
        let names: Vec<_> = self.servers.keys().cloned().collect();
        for name in &names {
            if let Some(info) = self.servers.get(name) {
                let pid = info.pid;
                if !is_process_running(pid) {
                    removed.push(name.clone());
                    self.servers.remove(name);
                }
            }
        }

        // Second pass: if multiple entries share the same socket path (happens
        // after server exec/reload), keep only the newest one.
        let remaining: Vec<_> = self.servers.keys().cloned().collect();
        let mut socket_to_newest: std::collections::HashMap<PathBuf, (String, String)> =
            std::collections::HashMap::new();
        for name in &remaining {
            if let Some(info) = self.servers.get(name) {
                let entry = socket_to_newest
                    .entry(info.socket.clone())
                    .or_insert_with(|| (name.clone(), info.started_at.clone()));
                if info.started_at > entry.1 {
                    *entry = (name.clone(), info.started_at.clone());
                }
            }
        }
        for name in &remaining {
            if let Some(info) = self.servers.get(name)
                && let Some((newest_name, _)) = socket_to_newest.get(&info.socket)
                && newest_name != name
            {
                removed.push(name.clone());
                self.servers.remove(name);
            }
        }

        if !removed.is_empty() {
            self.save().await?;
        }

        Ok(removed)
    }

    /// Add a session to a server
    pub fn add_session(&mut self, server_name: &str, session_name: &str) {
        if let Some(info) = self.servers.get_mut(server_name)
            && !info.sessions.contains(&session_name.to_string())
        {
            info.sessions.push(session_name.to_string());
        }
    }

    /// Remove a session from a server
    pub fn remove_session(&mut self, server_name: &str, session_name: &str) {
        if let Some(info) = self.servers.get_mut(server_name) {
            info.sessions.retain(|s| s != session_name);
        }
    }
}

/// Get the path to the registry file
pub fn registry_path() -> Result<PathBuf> {
    Ok(jcode_dir()?.join("servers.json"))
}

/// Get the socket directory path
pub fn socket_dir() -> Result<PathBuf> {
    Ok(crate::storage::runtime_dir().join("jcode"))
}

/// Get the socket path for a named server
pub fn server_socket_path(name: &str) -> PathBuf {
    socket_dir()
        .map(|d| d.join(format!("{}.sock", name)))
        .unwrap_or_else(|_| std::env::temp_dir().join(format!("jcode-{}.sock", name)))
}

/// Get the debug socket path for a named server
pub fn server_debug_socket_path(name: &str) -> PathBuf {
    socket_dir()
        .map(|d| d.join(format!("{}-debug.sock", name)))
        .unwrap_or_else(|_| std::env::temp_dir().join(format!("jcode-{}-debug.sock", name)))
}

/// Check if a process is still running
fn is_process_running(pid: u32) -> bool {
    crate::platform::is_process_running(pid)
}

/// Unregister a server from the registry
pub async fn unregister_server(name: &str) -> Result<()> {
    let mut registry = ServerRegistry::load().await?;
    registry.unregister(name);
    registry.save().await?;
    Ok(())
}

/// List all running servers
pub async fn list_servers() -> Result<Vec<ServerInfo>> {
    let mut registry = ServerRegistry::load().await?;
    registry.cleanup_stale().await?;
    Ok(registry.servers_by_time().into_iter().cloned().collect())
}

/// Publish the connected-client count for `name`. A missing entry is left
/// alone so a server never re-registers itself by accident.
pub async fn update_client_count(name: &str, clients: usize) -> Result<()> {
    let mut registry = ServerRegistry::load().await?;
    match registry.servers.get_mut(name) {
        Some(info) if info.clients != Some(clients) => info.clients = Some(clients),
        _ => return Ok(()),
    }
    registry.save().await
}

/// One live server as shown by the CLI and TUI status views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerStatusRow {
    pub name: String,
    pub pid: u32,
    pub version: String,
    /// `None` when `started_at` is invalid or in the future.
    pub uptime_secs: Option<u64>,
    /// `None` when the server has not published a count.
    pub clients: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerStatus {
    pub live_servers: usize,
    /// Sum over servers that published a count; `None` if none did.
    pub connected_clients: Option<usize>,
    pub servers: Vec<ServerStatusRow>,
}

/// Uptime from an RFC 3339 `started_at`, or `None` if it is unparsable or
/// later than `now`.
pub fn uptime_secs(started_at: &str, now: chrono::DateTime<chrono::Utc>) -> Option<u64> {
    let started = chrono::DateTime::parse_from_rfc3339(started_at).ok()?;
    u64::try_from(now.signed_duration_since(started).num_seconds()).ok()
}

/// Compact uptime such as `45s`, `7m`, `3h 12m` or `2d 4h`.
pub fn format_uptime(secs: u64) -> String {
    let (days, hours, mins) = (secs / 86_400, secs % 86_400 / 3_600, secs % 3_600 / 60);
    match (days, hours, mins) {
        (0, 0, 0) => format!("{secs}s"),
        (0, 0, m) => format!("{m}m"),
        (0, h, m) => format!("{h}h {m}m"),
        (d, h, _) => format!("{d}d {h}h"),
    }
}

pub fn server_status_from(
    servers: &[ServerInfo],
    now: chrono::DateTime<chrono::Utc>,
) -> ServerStatus {
    let rows: Vec<ServerStatusRow> = servers
        .iter()
        .map(|info| ServerStatusRow {
            name: info.name.clone(),
            pid: info.pid,
            version: info.version.clone(),
            uptime_secs: uptime_secs(&info.started_at, now),
            clients: info.clients,
        })
        .collect();
    let published: Vec<usize> = rows.iter().filter_map(|row| row.clients).collect();
    ServerStatus {
        live_servers: rows.len(),
        connected_clients: (!published.is_empty()).then(|| published.iter().sum()),
        servers: rows,
    }
}

/// Display lines shared by the CLI and TUI: a summary, then one per server.
pub fn server_status_lines(status: &ServerStatus) -> Vec<String> {
    let label = |count: Option<usize>| count.map_or_else(|| "n/a".to_string(), |n| n.to_string());
    let mut lines = vec![format!(
        "Servers: {}  Clients: {}",
        status.live_servers,
        label(status.connected_clients)
    )];
    for server in &status.servers {
        lines.push(format!(
            "{}  pid {}  up {}  clients {}",
            server.name,
            server.pid,
            server
                .uptime_secs
                .map_or_else(|| "n/a".to_string(), format_uptime),
            label(server.clients)
        ));
    }
    lines
}

/// Live servers only: goes through `list_servers`, which drops dead entries.
pub async fn server_status() -> Result<ServerStatus> {
    Ok(server_status_from(
        &list_servers().await?,
        chrono::Utc::now(),
    ))
}

/// Best-effort sync lookup for a server by socket path.
///
/// This is used by client-side window title code before the async runtime is fully
/// established or in synchronous spawn helpers.
pub fn find_server_by_socket_sync(socket: &std::path::Path) -> Option<ServerInfo> {
    let path = registry_path().ok()?;
    let content = std::fs::read_to_string(path).ok()?;
    let registry: ServerRegistry = serde_json::from_str(&content).ok()?;
    registry
        .servers
        .values()
        .find(|info| info.socket == socket)
        .cloned()
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;
