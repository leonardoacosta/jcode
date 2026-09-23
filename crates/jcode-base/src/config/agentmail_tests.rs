use crate::config::Config;

#[test]
fn agentmail_config_defaults_disabled_and_parses() {
    let defaults = Config::default();
    assert!(!defaults.safety.agentmail_enabled);
    assert!(!defaults.safety.agentmail_reply_enabled);
    assert!(defaults.safety.agentmail_api_key.is_none());

    let parsed: Config = toml::from_str(
        r#"[safety]
agentmail_enabled = true
agentmail_reply_enabled = true
agentmail_inbox_id = "inbox-test"
agentmail_api_key = "must-not-display"
agentmail_allowed_senders = ["owner@example.com"]
"#,
    )
    .unwrap();
    assert!(parsed.safety.agentmail_enabled);
    assert!(parsed.safety.agentmail_reply_enabled);
    assert_eq!(
        parsed.safety.agentmail_inbox_id.as_deref(),
        Some("inbox-test")
    );
    assert_eq!(
        parsed.safety.agentmail_allowed_senders,
        ["owner@example.com"]
    );
    assert!(!parsed.display_string().contains("must-not-display"));
    let mut no_allowlist = parsed.clone();
    no_allowlist.safety.agentmail_allowed_senders.clear();
    assert!(
        no_allowlist
            .display_string()
            .contains("AgentMail replies: disabled")
    );
}
