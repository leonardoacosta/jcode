use super::*;
use crate::storage::lock_test_env;
#[cfg(unix)]
use crate::transport::Listener;
use std::ffi::OsString;

fn test_server_info(name: &str) -> ServerInfo {
    ServerInfo {
        id: format!("server_{}_123", name),
        name: name.to_string(),
        icon: "🔥".to_string(),
        socket: PathBuf::from(format!("/tmp/{}.sock", name)),
        debug_socket: PathBuf::from(format!("/tmp/{}-debug.sock", name)),
        git_hash: "abc1234".to_string(),
        version: "v0.1.123".to_string(),
        pid: std::process::id(),
        started_at: "2025-01-01T00:00:00Z".to_string(),
        sessions: Vec::new(),
        clients: None,
    }
}

#[test]
fn test_server_info_display_name() {
    let info = test_server_info("blazing");
    assert_eq!(info.display_name(), "🔥 blazing");
}

#[test]
fn test_registry_find_by_name() {
    let mut registry = ServerRegistry::default();
    let info = test_server_info("blazing");
    registry.register(info);

    assert!(registry.find_by_name("blazing").is_some());
    assert!(registry.find_by_name("frozen").is_none());
}

#[test]
fn find_server_by_socket_sync_returns_matching_server() {
    let _guard = lock_test_env();
    let temp_home = tempfile::tempdir().expect("temp home");
    let prev_home: Option<OsString> = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let socket = PathBuf::from("/tmp/blazing.sock");
    let mut registry = ServerRegistry::default();
    let mut info = test_server_info("blazing");
    info.socket = socket.clone();
    registry.register(info.clone());
    std::fs::create_dir_all(temp_home.path()).expect("create temp home");
    std::fs::write(
        registry_path().expect("registry path"),
        serde_json::to_string(&registry).expect("serialize registry"),
    )
    .expect("write registry");

    let found = find_server_by_socket_sync(&socket).expect("find server by socket");
    assert_eq!(found.name, info.name);
    assert_eq!(found.icon, info.icon);

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_stale_preserves_live_socket_paths() {
    let _guard = lock_test_env();
    let temp_home = tempfile::tempdir().expect("temp home");
    let prev_home: Option<OsString> = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let temp_runtime = tempfile::tempdir().expect("temp runtime");
    let socket = temp_runtime.path().join("jcode.sock");
    let debug_socket = temp_runtime.path().join("jcode-debug.sock");
    let _listener = Listener::bind(&socket).expect("bind live socket");
    let _debug_listener = Listener::bind(&debug_socket).expect("bind live debug socket");
    let dead_pid = {
        let mut child = std::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .expect("spawn short-lived child");
        let pid = child.id();
        let _ = child.wait().expect("wait for short-lived child");
        pid
    };

    let mut registry = ServerRegistry::default();
    registry.register(ServerInfo {
        id: "server_old_1".to_string(),
        name: "old".to_string(),
        icon: "🪦".to_string(),
        socket: socket.clone(),
        debug_socket: debug_socket.clone(),
        git_hash: "deadbeef".to_string(),
        version: "v0.0.0".to_string(),
        pid: dead_pid,
        started_at: "2026-01-01T00:00:00Z".to_string(),
        sessions: Vec::new(),
        clients: None,
    });

    let removed = registry.cleanup_stale().await.expect("cleanup stale");
    assert_eq!(removed, vec!["old".to_string()]);
    assert!(
        socket.exists(),
        "cleanup_stale must not unlink a live server socket path"
    );
    assert!(
        debug_socket.exists(),
        "cleanup_stale must not unlink a live debug socket path"
    );

    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

fn status_info(name: &str, started_at: &str, clients: Option<usize>) -> ServerInfo {
    ServerInfo {
        started_at: started_at.to_string(),
        clients,
        ..test_server_info(name)
    }
}

#[test]
fn server_status_counts_servers_clients_and_uptime() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T03:12:30Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let servers = [
        status_info("a", "2026-01-01T00:00:00Z", Some(2)),
        status_info("b", "2026-01-01T03:12:00Z", Some(1)),
    ];
    let status = server_status_from(&servers, now);
    assert_eq!(status.live_servers, 2);
    assert_eq!(status.connected_clients, Some(3));
    assert_eq!(status.servers[0].uptime_secs, Some(3 * 3600 + 12 * 60 + 30));
    assert_eq!(status.servers[1].uptime_secs, Some(30));
}

#[test]
fn malformed_or_future_started_at_degrades_to_unavailable_uptime() {
    let now = chrono::Utc::now();
    let future = (now + chrono::Duration::hours(2)).to_rfc3339();
    let servers = [
        status_info("bad", "not a timestamp", None),
        status_info("future", &future, Some(1)),
        status_info("ok", "2020-01-01T00:00:00Z", Some(0)),
    ];
    let status = server_status_from(&servers, now);
    assert_eq!(status.live_servers, 3, "status stays available");
    assert_eq!(status.servers[0].uptime_secs, None);
    assert_eq!(status.servers[1].uptime_secs, None);
    assert!(status.servers[2].uptime_secs.is_some());
    assert_eq!(status.connected_clients, Some(1));
    // Servers that never published a count are unavailable, not zero.
    let none = server_status_from(&[status_info("old", "2020-01-01T00:00:00Z", None)], now);
    assert_eq!(none.connected_clients, None);
}

#[test]
fn uptime_formats_compactly() {
    assert_eq!(format_uptime(45), "45s");
    assert_eq!(format_uptime(7 * 60), "7m");
    assert_eq!(format_uptime(3 * 3600 + 12 * 60), "3h 12m");
    assert_eq!(format_uptime(2 * 86_400 + 4 * 3600), "2d 4h");
}

#[tokio::test]
async fn status_read_excludes_and_cleans_stale_entries() {
    let _guard = lock_test_env();
    let home = tempfile::tempdir().unwrap();
    let prev_home: Option<OsString> = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", home.path());
    let mut registry = ServerRegistry::default();
    registry.register(status_info("live", "2026-01-01T00:00:00Z", Some(1)));
    let dead_pid = {
        let mut child = std::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .expect("spawn short-lived child");
        let pid = child.id();
        child.wait().expect("wait for short-lived child");
        pid
    };
    registry.register(ServerInfo {
        pid: dead_pid,
        ..status_info("dead", "2026-01-01T00:00:00Z", Some(5))
    });
    registry.save().await.unwrap();
    let status = server_status().await.unwrap();
    assert_eq!(status.live_servers, 1);
    assert_eq!(status.connected_clients, Some(1));
    assert!(
        ServerRegistry::load()
            .await
            .unwrap()
            .find_by_name("dead")
            .is_none()
    );
    match prev_home {
        Some(prev) => crate::env::set_var("JCODE_HOME", prev),
        None => crate::env::remove_var("JCODE_HOME"),
    }
}

#[test]
fn status_lines_are_the_cli_output_for_servers_clients_and_unavailable_uptime() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-01-01T03:12:30Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let servers = [
        status_info("a", "2026-01-01T00:00:00Z", Some(2)),
        status_info("odd", "garbage", None),
    ];
    let lines = server_status_lines(&server_status_from(&servers, now));
    assert_eq!(lines[0], "Servers: 2  Clients: 2");
    assert!(lines[1].starts_with("a  pid "));
    assert!(lines[1].ends_with("up 3h 12m  clients 2"), "{}", lines[1]);
    assert!(lines[2].ends_with("up n/a  clients n/a"), "{}", lines[2]);
    let none = server_status_lines(&server_status_from(&[], now));
    assert_eq!(none, vec!["Servers: 0  Clients: n/a".to_string()]);
}
