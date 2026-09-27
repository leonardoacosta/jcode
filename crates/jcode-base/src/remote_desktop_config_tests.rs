use super::{Config, RemoteDesktopConfig, RemoteDesktopTarget};

fn target(id: &str) -> RemoteDesktopTarget {
    RemoteDesktopTarget {
        id: id.into(),
        ssh_destination: "mac".into(),
        executable_path: "/opt/agent-desktop".into(),
        timeout_secs: 30,
        max_output_bytes: 1_048_576,
        enabled: true,
    }
}

#[test]
fn remote_desktop_targets_default_empty_and_lookup_exact_enabled_id() {
    let config = Config::default();
    assert!(config.remote_desktop.targets.is_empty());
    assert!(config.remote_desktop.validate().is_ok());

    let config = RemoteDesktopConfig {
        targets: vec![target("mac-one"), target("mac-two")],
    };
    assert_eq!(config.target("mac-one").unwrap().ssh_destination, "mac");
    assert!(config.target("MAC-ONE").is_err());
    assert!(config.target("unconfigured").is_err());
}

#[test]
fn remote_desktop_rejects_duplicate_or_shell_unsafe_target_values() {
    let duplicate = RemoteDesktopConfig {
        targets: vec![target("same"), target("same")],
    };
    assert!(duplicate.validate().is_err());

    let mut bad_destination = target("ok");
    bad_destination.ssh_destination = "host;touch".into();
    assert!(
        RemoteDesktopConfig {
            targets: vec![bad_destination]
        }
        .validate()
        .is_err()
    );

    let mut bad_path = target("ok");
    bad_path.executable_path = "relative/path".into();
    assert!(
        RemoteDesktopConfig {
            targets: vec![bad_path]
        }
        .validate()
        .is_err()
    );
}

#[test]
fn remote_desktop_target_config_round_trips_with_unrelated_settings() {
    let source = r#"
[tools]
profile = "minimal"
[[remote_desktop.targets]]
id = "mac-one"
ssh_destination = "mac"
executable_path = "/opt/agent-desktop"
timeout_secs = 45
max_output_bytes = 2097152
enabled = true
"#;
    let parsed: Config = toml::from_str(source).unwrap();
    parsed.remote_desktop.validate().unwrap();
    let round_trip: Config = toml::from_str(&toml::to_string(&parsed).unwrap()).unwrap();
    assert_eq!(round_trip.tools.profile, "minimal");
    assert_eq!(
        round_trip.remote_desktop.targets,
        parsed.remote_desktop.targets
    );
}
