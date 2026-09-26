use super::*;

#[test]
fn disabled_without_configuration() {
    let dir = tempfile::tempdir().unwrap();
    assert!(load(dir.path()).unwrap().is_none());
}

#[test]
fn configuration_rejects_public_or_invalid_origins() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::new(
        dir.path().to_path_buf(),
        dir.path().join("daemon.sock"),
        8123,
    );
    assert!(config.validate().is_ok());
    for origin in [
        "http://example.ts.net",
        "https://example.com",
        "https://node.tail.ts.net/path",
        "https://user@node.tail.ts.net",
        "https://node.tail.ts.net?token=x",
    ] {
        config.tailnet_origin = Some(origin.into());
        assert!(config.validate().is_err(), "accepted {origin}");
    }
    config.tailnet_origin = Some("https://node.tail.ts.net:8443".into());
    assert!(config.validate().is_ok());
}

#[test]
fn configuration_persists_secret_and_socket_scope() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config::new(
        dir.path().to_path_buf(),
        dir.path().join("daemon.sock"),
        8123,
    );
    save(dir.path(), &config).unwrap();
    let restored = load(dir.path()).unwrap().unwrap();
    assert_eq!(restored.control_token, config.control_token);
    assert_eq!(restored.socket_path, config.socket_path);
    assert_eq!(restored.local_origin(), "http://127.0.0.1:8123");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.path().join("config.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o077,
            0
        );
    }
}

#[test]
fn corrupt_configuration_is_not_replaced() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), "broken").unwrap();
    assert!(load(dir.path()).is_err());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("config.json")).unwrap(),
        "broken"
    );
}
