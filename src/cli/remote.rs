//! Mac-to-homelab transport setup for the normal `jcode` client.
//!
//! The public command remains `jcode`. This module owns only the private
//! LocalForward recovery path and the read-only presence query used by the
//! menu bar. It never turns a failed homelab connection into a local fallback.

use anyhow::Result;
#[cfg(target_os = "macos")]
use anyhow::{Context, anyhow};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

use super::args::Args;
#[cfg(target_os = "macos")]
use super::args::Command as CliCommand;

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const DEFAULT_SSH_HOST: &str = "jcode-homelab";
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const DEFAULT_REMOTE_SOCKET: &str = "/run/user/1000/jcode.sock";
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) const DEFAULT_REMOTE_WORKING_DIR: &str = "/home/nyaptor/dev/jcode";

#[cfg_attr(not(any(test, target_os = "macos")), allow(dead_code))]
pub(crate) fn homelab_ssh_args(
    host: &str,
    local_socket: &Path,
    remote_socket: &str,
) -> Vec<String> {
    vec![
        "-fNT".into(),
        "-o".into(),
        "ExitOnForwardFailure=yes".into(),
        "-o".into(),
        "ServerAliveInterval=30".into(),
        "-o".into(),
        "ServerAliveCountMax=3".into(),
        "-o".into(),
        "StreamLocalBindUnlink=yes".into(),
        "-L".into(),
        format!("{}:{remote_socket}", local_socket.display()),
        host.into(),
    ]
}

#[cfg(target_os = "macos")]
fn local_socket_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine the Mac home directory")?;
    Ok(home.join(".jcode/homelab.sock"))
}

#[cfg(target_os = "macos")]
fn socket_is_live(path: &Path) -> bool {
    #[cfg(unix)]
    {
        return std::os::unix::net::UnixStream::connect(path).is_ok();
    }
    #[allow(unreachable_code)]
    false
}

#[cfg(target_os = "macos")]
fn wait_for_socket(path: &Path) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
    while std::time::Instant::now() < deadline {
        if socket_is_live(path) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}

#[cfg(target_os = "macos")]
fn ensure_socket(path: &Path) -> Result<()> {
    if socket_is_live(path) {
        return Ok(());
    }

    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("remove stale homelab socket {}", path.display()))?;
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let host =
        std::env::var("JCODE_HOMELAB_SSH_HOST").unwrap_or_else(|_| DEFAULT_SSH_HOST.to_string());
    let remote_socket = std::env::var("JCODE_HOMELAB_REMOTE_SOCKET")
        .unwrap_or_else(|_| DEFAULT_REMOTE_SOCKET.to_string());
    let args = homelab_ssh_args(&host, path, &remote_socket);
    let status = Command::new("ssh")
        .args(&args)
        .status()
        .context("start the homelab SSH LocalForward")?;
    if !status.success() || !wait_for_socket(path) {
        anyhow::bail!(
            "homelab connection unavailable: SSH LocalForward to {host} did not create {}",
            path.display()
        );
    }
    Ok(())
}

/// Prepare the default Mac client only. Explicit sockets and subcommands are
/// left alone so server/admin commands remain usable during recovery.
#[cfg(not(target_os = "macos"))]
pub(crate) fn prepare_default_args(_args: &mut Args) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_default_args(args: &mut Args) -> Result<()> {
    let remote_client_command = matches!(
        &args.command,
        None | Some(CliCommand::Connect) | Some(CliCommand::Menubar { .. })
    );
    if !remote_client_command
        || args.socket.is_some()
        || std::env::var_os("JCODE_SOCKET").is_some()
        || std::env::var_os("JCODE_LOCAL_ONLY").is_some()
    {
        return Ok(());
    }

    let path = local_socket_path()?;
    if let Err(error) = ensure_socket(&path) {
        eprintln!("warning: {error:#}");
        eprintln!("warning: set JCODE_LOCAL_ONLY=1 for an intentional local recovery session");
        return Err(error);
    }
    args.socket = Some(path.display().to_string());
    if args.remote_working_dir.is_none() {
        args.remote_working_dir = Some(
            std::env::var("JCODE_HOMELAB_REMOTE_WORKING_DIR")
                .unwrap_or_else(|_| DEFAULT_REMOTE_WORKING_DIR.to_string()),
        );
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub(crate) fn fetch_remote_presence_blocking() -> Result<Vec<crate::session::SessionPresence>> {
    let path = crate::server::socket_path();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let mut client = crate::server::Client::connect_with_path(path).await?;
        match client.get_presence().await? {
            crate::protocol::ServerEvent::Presence { sessions, .. } => Ok(sessions
                .into_iter()
                .map(|session| crate::session::SessionPresence {
                    session_id: session.session_id,
                    pid: 0,
                    streaming: session.streaming,
                    streaming_since: None,
                    internal: false,
                })
                .collect()),
            other => Err(anyhow!("unexpected homelab presence response: {other:?}")),
        }
    })
}
