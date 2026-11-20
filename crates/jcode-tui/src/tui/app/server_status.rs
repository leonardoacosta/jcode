//! `/servers`: live server count, connected-client count and per-server uptime.
//!
//! Rows are UI chrome, so each one stays a single visual row: it is truncated
//! at terminal-cell boundaries with an ellipsis, never wrapped.

use crate::registry::ServerStatus;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Cut `text` to at most `max` terminal cells, ending in `…` when shortened.
fn truncate_cells(text: &str, max: usize) -> String {
    if text.width() <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w + 1 > max {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}

/// The shared status lines, each cut to `width` cells. Never wraps.
pub(super) fn server_status_rows(status: &ServerStatus, width: usize) -> Vec<String> {
    crate::registry::server_status_lines(status)
        .iter()
        .map(|line| truncate_cells(line, width))
        .collect()
}

/// Handle `/servers`. Reads the registry, which drops stale entries first.
pub(super) fn handle_servers_command(app: &mut super::App, trimmed: &str) -> bool {
    if trimmed != "/servers" {
        return false;
    }
    let servers = crate::tui::session_picker::load_servers();
    let status = crate::registry::server_status_from(&servers, chrono::Utc::now());
    let width = crossterm::terminal::size()
        .map(|(w, _)| usize::from(w).saturating_sub(4))
        .unwrap_or(76)
        .max(10);
    let text = server_status_rows(&status, width).join("\n");
    app.push_display_message(super::DisplayMessage::system(text).with_title("Servers"));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{ServerStatusRow, server_status_from};

    fn status(rows: Vec<ServerStatusRow>) -> ServerStatus {
        ServerStatus {
            live_servers: rows.len(),
            connected_clients: Some(rows.iter().filter_map(|r| r.clients).sum()),
            servers: rows,
        }
    }

    fn row(name: &str, uptime: Option<u64>, clients: Option<usize>) -> ServerStatusRow {
        ServerStatusRow {
            name: name.into(),
            pid: 4242,
            version: "v1".into(),
            uptime_secs: uptime,
            clients,
        }
    }

    #[test]
    fn rows_show_counts_and_uptime() {
        let rows = server_status_rows(
            &status(vec![row("blazing", Some(3 * 3600 + 120), Some(2))]),
            80,
        );
        assert_eq!(rows[0], "Servers: 1  Clients: 2");
        assert_eq!(rows[1], "blazing  pid 4242  up 3h 2m  clients 2");
    }

    #[test]
    fn invalid_uptime_is_explicitly_unavailable() {
        let rows = server_status_rows(&status(vec![row("odd", None, None)]), 80);
        assert!(rows[1].contains("up n/a"), "{rows:?}");
        assert!(rows[1].contains("clients n/a"), "{rows:?}");
    }

    #[test]
    fn narrow_terminal_rows_never_wrap_and_truncate_with_ellipsis() {
        let wide = row("名前の長いサーバー🔥🔥🔥", Some(90_000), Some(12));
        for width in [1usize, 2, 5, 12, 20, 33] {
            for text in server_status_rows(&status(vec![wide.clone()]), width) {
                assert!(
                    text.width() <= width,
                    "{width}: {text:?} is {} cells",
                    text.width()
                );
                assert!(!text.contains('\n'));
            }
        }
        let rows = server_status_rows(&status(vec![wide]), 20);
        assert!(rows[1].ends_with('…'), "{rows:?}");
    }

    #[test]
    fn status_from_registry_infos_feeds_rows() {
        let now = chrono::Utc::now();
        let info = crate::registry::ServerInfo {
            id: "server_a_1".into(),
            name: "a".into(),
            icon: "x".into(),
            socket: "/tmp/a.sock".into(),
            debug_socket: "/tmp/a-debug.sock".into(),
            git_hash: "h".into(),
            version: "v".into(),
            pid: 1,
            started_at: "garbage".into(),
            sessions: vec![],
            clients: Some(1),
        };
        let rows = server_status_rows(&server_status_from(&[info], now), 60);
        assert_eq!(rows.len(), 2);
        assert!(rows[1].contains("up n/a"));
    }
}
