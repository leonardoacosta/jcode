//! `/mcp` status table and `/mcp login <server>`.

use super::{App, DisplayMessage};
use crate::mcp::McpConfig;
use std::collections::HashSet;
use std::path::Path;

/// Config paths `/mcp` reports when nothing is configured.
const CONFIG_PATHS: &str = "~/.jcode/mcp.json, ~/.claude.json, ~/.claude/mcp.json, \
    <project>/.mcp.json, <project>/.jcode/mcp.json, <project>/.claude/mcp.json";

fn server_state(
    cfg: &crate::mcp::McpServerConfig,
    tools: Option<usize>,
    cwd: Option<&Path>,
) -> String {
    if !cfg.is_enabled() {
        "disabled".into()
    } else if !cfg.in_workspace(cwd) {
        "out of scope".into()
    } else {
        match tools {
            Some(0) => "connecting".into(),
            Some(_) => "connected".into(),
            None if cfg.is_lazy() => "lazy".into(),
            None => "not connected".into(),
        }
    }
}

/// Markdown table of configured servers. `global_names` holds servers that
/// also resolve without a project dir; others come from project config.
pub(super) fn render_mcp_status(
    config: &McpConfig,
    global_names: &HashSet<String>,
    connected: &[(String, usize)],
    cwd: Option<&Path>,
) -> String {
    if config.servers.is_empty() {
        return format!("**MCP**: no servers configured.\n\nChecked: {CONFIG_PATHS}");
    }
    let mut names: Vec<_> = config.servers.keys().collect();
    names.sort();
    let mut out = String::from(
        "**MCP servers**\n\n| Server | Scope | State | Tools | Exposure |\n|---|---|---|---|---|\n",
    );
    for name in names {
        let cfg = &config.servers[name];
        let tools = connected.iter().find(|(n, _)| n == name).map(|(_, c)| *c);
        let scope = if global_names.contains(name) {
            "global"
        } else {
            "project"
        };
        let exposure = if cfg.is_direct() { "direct" } else { "search" };
        out.push_str(&format!(
            "| {name} | {scope} | {} | {} | {exposure} |\n",
            server_state(cfg, tools, cwd),
            tools.filter(|c| *c > 0).map_or("-".into(), |c| c.to_string()),
        ));
    }
    out.push_str("\nFailure details: `mcp status` tool or `~/.jcode/logs`. Sign in: `/mcp login <server>`.");
    out
}

pub(super) fn handle_mcp_command(app: &mut App, trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("/mcp") else {
        return false;
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return false;
    }
    let rest = rest.trim();
    let cwd = app.session.working_dir.clone().map(std::path::PathBuf::from);
    let config = McpConfig::load_for_dir(cwd.as_deref());

    if let Some(server) = rest.strip_prefix("login") {
        let server = server.trim();
        if server.is_empty() {
            app.push_display_message(DisplayMessage::error("Usage: /mcp login <server>"));
            return true;
        }
        start_login(app, &config, server);
        return true;
    }
    if !rest.is_empty() {
        app.push_display_message(DisplayMessage::error(
            "Usage: /mcp | /mcp login <server>",
        ));
        return true;
    }

    let global: HashSet<String> = McpConfig::load_for_dir(None).servers.into_keys().collect();
    let message = render_mcp_status(&config, &global, &app.mcp_server_names, cwd.as_deref());
    app.push_display_message(DisplayMessage::system(message));
    true
}

fn start_login(app: &mut App, config: &McpConfig, name: &str) {
    let Some(server) = config.servers.get(name) else {
        app.push_display_message(DisplayMessage::error(format!(
            "MCP server '{name}' is not configured."
        )));
        return;
    };
    let Some(url) = server.url.clone().filter(|_| server.is_remote()) else {
        app.push_display_message(DisplayMessage::error(format!(
            "MCP server '{name}' uses stdio; OAuth login applies to remote servers only."
        )));
        return;
    };
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        app.push_display_message(DisplayMessage::error("No async runtime for MCP login."));
        return;
    };
    app.push_display_message(DisplayMessage::system(format!(
        "Starting OAuth login for MCP server '{name}' (waits up to five minutes for the browser callback)..."
    )));
    let session_id = Some(app.session.id.clone());
    let name = name.to_string();
    handle.spawn(async move {
        let publish = |message: String| {
            crate::bus::Bus::global().publish(crate::bus::BusEvent::UiActivity(
                crate::bus::UiActivity::auth(session_id.clone(), message, None::<String>),
            ));
        };
        let result = crate::mcp::oauth::login(&url, crate::mcp::oauth::BrowserMode::Open, |auth| {
            publish(format!("Open this URL to authorize MCP server '{name}':\n{auth}"));
        })
        .await;
        publish(match result {
            Ok(()) => format!("Logged in to MCP server '{name}'. Reconnect it to use the credentials."),
            Err(error) => format!("MCP login failed for '{name}': {error:#}"),
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> McpConfig {
        serde_json::from_value(serde_json::json!({"servers": {
            "live": {"command": "a", "expose": "direct"},
            "off": {"command": "b", "enabled": false},
            "brown": {"command": "c", "workspaces": ["/dev/brown/**"]},
            "later": {"command": "d", "lazy": true},
            "proj": {"command": "e"}
        }}))
        .unwrap()
    }

    #[test]
    fn status_table_shows_scope_state_and_exposure() {
        let global: HashSet<String> = ["live", "off", "brown", "later"]
            .into_iter()
            .map(String::from)
            .collect();
        let out = render_mcp_status(
            &config(),
            &global,
            &[("live".into(), 3), ("proj".into(), 0)],
            Some(Path::new("/dev/personal/x")),
        );
        assert!(out.contains("| live | global | connected | 3 | direct |"), "{out}");
        assert!(out.contains("| off | global | disabled | - | search |"), "{out}");
        assert!(out.contains("| brown | global | out of scope |"), "{out}");
        assert!(out.contains("| later | global | lazy |"), "{out}");
        assert!(out.contains("| proj | project | connecting |"), "{out}");
    }

    #[test]
    fn empty_state_names_config_paths() {
        let out = render_mcp_status(&McpConfig::default(), &HashSet::new(), &[], None);
        assert!(out.contains("no servers configured"));
        assert!(out.contains("~/.jcode/mcp.json") && out.contains(".mcp.json"));
    }

    #[test]
    fn login_without_server_shows_usage() {
        let mut app = super::super::tests::create_test_app();
        assert!(handle_mcp_command(&mut app, "/mcp login"));
        assert!(app.display_messages().last().unwrap().content.contains("Usage"));
        assert!(!handle_mcp_command(&mut app, "/mcpfoo"));
    }
}
