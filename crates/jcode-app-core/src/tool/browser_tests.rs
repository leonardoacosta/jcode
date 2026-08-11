use super::*;

#[test]
fn press_script_uses_selector_when_present() {
    let script = build_press_script(Some("Enter"), Some("#email")).unwrap();
    assert!(script.contains("document.querySelector"));
    assert!(script.contains("Enter"));
}

#[test]
fn content_formatter_prefers_content_text() {
    let rendered = format_content_result(&json!({"content": "hello", "title": "x"}));
    assert_eq!(rendered, "hello");
}

#[test]
fn snapshot_maps_to_annotated_get_content() {
    let input = BrowserInput {
        action: "snapshot".into(),
        browser: None,
        profile: None,
        provider_action: None,
        params: None,
        url: None,
        tab_id: Some(7),
        tab_ref: None,
        browser_ref: None,
        window_ref: None,
        generation: None,
        window_id: None,
        frame_id: Some(3),
        all_frames: Some(true),
        selector: None,
        text: None,
        contains: None,
        script: None,
        key: None,
        x: None,
        y: None,
        format: None,
        wait: None,
        new_tab: None,
        focus: None,
        clear: None,
        submit: None,
        page_world: None,
        position: None,
        behavior: None,
        timeout_ms: None,
        path: None,
        fields: None,
        scroll_to: None,
    };

    let (action, params, _) = bridge_request("snapshot", &input).unwrap();
    assert_eq!(action, "getContent");
    assert_eq!(params["format"], "annotated");
    assert_eq!(params["tabId"], 7);
    assert_eq!(params["frameId"], 3);
    assert_eq!(params["allFrames"], true);
}

#[test]
fn eval_maps_script_and_page_world() {
    let input = BrowserInput {
        action: "eval".into(),
        browser: None,
        profile: None,
        provider_action: None,
        params: None,
        url: None,
        tab_id: None,
        tab_ref: None,
        browser_ref: None,
        window_ref: None,
        generation: None,
        window_id: None,
        frame_id: None,
        all_frames: None,
        selector: None,
        text: None,
        contains: None,
        script: Some("return document.title".into()),
        key: None,
        x: None,
        y: None,
        format: None,
        wait: None,
        new_tab: None,
        focus: None,
        clear: None,
        submit: None,
        page_world: Some(true),
        position: None,
        behavior: None,
        timeout_ms: None,
        path: None,
        fields: None,
        scroll_to: None,
    };

    let (action, params, _) = bridge_request("eval", &input).unwrap();
    assert_eq!(action, "evaluate");
    assert_eq!(params["script"], "return document.title");
    assert_eq!(params["pageWorld"], true);
}

#[test]
fn interactables_maps_to_bridge_action() {
    let input = BrowserInput {
        action: "interactables".into(),
        browser: None,
        profile: None,
        provider_action: None,
        params: None,
        url: None,
        tab_id: Some(9),
        tab_ref: None,
        browser_ref: None,
        window_ref: None,
        generation: None,
        window_id: None,
        frame_id: None,
        all_frames: None,
        selector: Some("main".into()),
        text: None,
        contains: None,
        script: None,
        key: None,
        x: None,
        y: None,
        format: None,
        wait: None,
        new_tab: None,
        focus: None,
        clear: None,
        submit: None,
        page_world: None,
        position: None,
        behavior: None,
        timeout_ms: None,
        path: None,
        fields: None,
        scroll_to: None,
    };

    let (action, params, _) = bridge_request("interactables", &input).unwrap();
    assert_eq!(action, "getInteractables");
    assert_eq!(params["tabId"], 9);
    assert_eq!(params["selector"], "main");
}

#[test]
fn schema_exposes_advanced_browser_fields() {
    let schema = BrowserTool::new().parameters_schema();
    let props = schema["properties"]
        .as_object()
        .expect("browser schema should have properties");

    assert!(props.contains_key("action"));
    assert!(props.contains_key("browser"));
    assert!(props.contains_key("profile"));
    assert!(props.contains_key("url"));
    assert!(props.contains_key("tab_id"));
    assert!(props.contains_key("tab_ref"));
    assert!(props.contains_key("frame_id"));
    assert!(props.contains_key("selector"));
    assert!(props.contains_key("text"));
    assert!(props.contains_key("contains"));
    assert!(props.contains_key("script"));
    assert!(props.contains_key("key"));
    assert!(props.contains_key("x"));
    assert!(props.contains_key("y"));
    assert!(props.contains_key("format"));
    assert!(props.contains_key("wait"));
    assert!(props.contains_key("new_tab"));
    assert!(props.contains_key("timeout_ms"));
    assert!(props.contains_key("path"));
    assert!(props.contains_key("fields"));
    assert!(props.contains_key("provider_action"));
    assert!(props.contains_key("params"));
    assert!(props.contains_key("all_frames"));
    assert!(props.contains_key("focus"));
    assert!(props.contains_key("clear"));
    assert!(props.contains_key("submit"));
    assert!(props.contains_key("page_world"));
    assert!(props.contains_key("position"));
    assert!(props.contains_key("behavior"));
    assert!(props.contains_key("scroll_to"));
    assert!(props.contains_key("browser_ref"));
    assert!(props.contains_key("window_ref"));
    assert!(props.contains_key("generation"));

    let browser_enum = props["browser"]["enum"]
        .as_array()
        .expect("browser enum should be an array");
    assert!(browser_enum.iter().any(|value| value == "mac"));
}

#[test]
fn resolve_provider_accepts_auto_firefox_chrome_and_mac() {
    assert!(resolve_provider(Some("auto")).is_ok());
    assert!(resolve_provider(Some("firefox")).is_ok());
    assert!(resolve_provider(Some("chrome")).is_ok());
    assert!(resolve_provider(Some("mac")).is_ok());
}

#[test]
fn resolve_provider_rejects_unsupported_browser() {
    let err = resolve_provider(Some("safari"))
        .err()
        .expect("safari should not resolve yet");
    assert!(
        err.to_string()
            .contains("not wired into the built-in browser tool")
    );
}

#[test]
fn profile_selection_requires_explicit_chrome() {
    assert!(validate_profile_route(Some("social"), "chrome").is_ok());
    assert!(validate_profile_route(Some("social"), "auto").is_err());
    assert!(validate_profile_route(Some("social"), "firefox").is_err());
    assert!(validate_profile_route(Some("social"), "mac").is_err());
    assert!(validate_profile_route(None, "auto").is_ok());
}

#[cfg(unix)]
#[test]
fn mac_fleet_config_uses_safe_env_and_default_paths() {
    let _guard = jcode_base::storage::lock_test_env();
    let temp = tempfile::TempDir::new().expect("create temp dir");
    jcode_base::env::set_var("JCODE_HOME", temp.path());
    jcode_base::env::set_var("JCODE_MAC_BROWSER_FLEET_SECRET", "env-secret");
    jcode_base::env::set_var(
        "JCODE_MAC_BROWSER_FLEET_SOCKET",
        temp.path().join("custom.sock"),
    );

    let config = mac_fleet::MacFleetConfig::from_env().expect("config should load");
    assert_eq!(config.secret, "env-secret");
    assert_eq!(config.socket_path, temp.path().join("custom.sock"));

    jcode_base::env::remove_var("JCODE_MAC_BROWSER_FLEET_SECRET");
    jcode_base::env::remove_var("JCODE_MAC_BROWSER_FLEET_SOCKET");
    std::fs::create_dir_all(temp.path().join("browser")).expect("create browser config dir");
    std::fs::write(
        temp.path().join("browser/mac-fleet.secret"),
        "file-secret\n",
    )
    .expect("write peer secret");
    let config = mac_fleet::MacFleetConfig::from_env().expect("default config should load");
    assert_eq!(
        config.socket_path,
        temp.path().join("browser/mac-fleet.sock")
    );
    assert_eq!(config.secret, "file-secret");
}

#[cfg(unix)]
#[test]
fn mac_fleet_maps_tool_actions_to_bounded_wire_requests() {
    let input: BrowserInput = serde_json::from_value(json!({
        "action": "open",
        "browser": "mac",
        "browser_ref": "chrome:stable",
        "window_ref": "win-1",
        "tab_ref": "tab-1",
        "generation": 7,
        "url": "https://example.com",
        "timeout_ms": 2500
    }))
    .unwrap();

    let request = mac_fleet::build_request("req-1", "secret".into(), "open", &input).unwrap();
    assert_eq!(
        request.action,
        jcode_mac_browser_fleet::WireAction::Navigate
    );
    assert_eq!(request.target_generation, 7);
    assert_eq!(request.payload["target"]["browser_id"], "chrome:stable");
    assert_eq!(request.payload["target"]["window_id"], "win-1");
    assert_eq!(request.payload["target"]["tab_id"], "tab-1");
    assert_eq!(request.payload["url"], "https://example.com");

    let encoded = mac_fleet::encode_request_line(&request).unwrap();
    assert!(encoded.ends_with(b"\n"));
    assert!(encoded.len() <= mac_fleet::MAX_REQUEST_BYTES + 1);
}

#[cfg(unix)]
#[test]
fn mac_fleet_errors_preserve_policy_meaning() {
    let approval = mac_fleet::tool_error_from_wire(json!({
        "ok": false,
        "error": {"kind": "approvalRequired", "message": "local approval is required"}
    }))
    .unwrap_err()
    .to_string();
    assert!(approval.contains("approval required"));

    let stale = mac_fleet::tool_error_from_wire(json!({
        "ok": false,
        "error": {"kind": "staleGeneration", "message": "target generation is stale"}
    }))
    .unwrap_err()
    .to_string();
    assert!(stale.contains("stale generation"));
}

#[test]
fn prepend_setup_message_preserves_images_and_metadata() {
    let output = ToolOutput::new("done")
        .with_title("browser screenshot")
        .with_metadata(json!({"backend": "firefox_agent_bridge"}))
        .with_labeled_image("image/png", "abc", "shot");

    let output = prepend_setup_message(output, "setup log");
    assert!(output.output.starts_with("setup log\n\ndone"));
    assert_eq!(output.images.len(), 1);
    assert_eq!(output.title.as_deref(), Some("browser screenshot"));
    assert_eq!(output.metadata.as_ref().unwrap()["setup_ran"], true);
    assert_eq!(
        output.metadata.as_ref().unwrap()["backend"],
        "firefox_agent_bridge"
    );
}

#[test]
fn description_tells_models_to_check_status_before_setup() {
    let tool = BrowserTool::new();
    let description = tool.description();
    assert!(description.contains("action='status'"));
    assert!(description.contains("setup only if not ready"));
}

#[cfg(unix)]
#[tokio::test]
async fn readiness_does_not_trust_a_stale_setup_marker() {
    use std::os::unix::fs::PermissionsExt;

    let _guard = jcode_base::storage::lock_test_env();
    let prev_home = std::env::var_os("JCODE_HOME");
    let temp = tempfile::TempDir::new().expect("create temp dir");
    jcode_base::env::set_var("JCODE_HOME", temp.path());

    let browser_dir = temp.path().join("browser");
    std::fs::create_dir_all(&browser_dir).expect("create browser dir");
    let browser = browser_dir.join("browser");
    std::fs::write(&browser, "#!/bin/sh\nexit 1\n").expect("write fake browser");
    let mut perms = std::fs::metadata(&browser)
        .expect("stat fake browser")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&browser, perms).expect("chmod fake browser");
    std::fs::write(browser_dir.join("firefox-agent-bridge-host"), "host").expect("write fake host");
    std::fs::write(browser_dir.join(".setup-complete"), "complete").expect("write setup marker");

    let error = ensure_firefox_ready()
        .await
        .expect_err("stale setup marker must not bypass live readiness");
    let message = error.to_string();
    assert!(message.contains("not responding"), "{message}");
    assert!(
        message.contains("Do not retry browser actions"),
        "{message}"
    );
    assert!(message.contains("capability discovery"), "{message}");

    if let Some(prev_home) = prev_home {
        jcode_base::env::set_var("JCODE_HOME", prev_home);
    } else {
        jcode_base::env::remove_var("JCODE_HOME");
    }
}

#[tokio::test]
#[ignore = "requires installed agent-browser and Chrome"]
async fn agent_browser_provider_live_smoke() {
    if std::env::var("JCODE_AGENT_BROWSER_LIVE").as_deref() != Ok("1") {
        eprintln!("set JCODE_AGENT_BROWSER_LIVE=1 to run the live Chrome provider smoke test");
        return;
    }

    let tool = BrowserTool::new();
    let session_id = "live/chrome smoke".to_string();
    let profile = std::env::var("JCODE_AGENT_BROWSER_LIVE_PROFILE").ok();
    let ctx = ToolContext {
        session_id: session_id.clone(),
        message_id: "message".to_string(),
        tool_call_id: "tool".to_string(),
        working_dir: None,
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: jcode_tool_core::ToolExecutionMode::Direct,
    };

    let status = tool
        .execute(
            json!({"action": "status", "browser": "chrome"}),
            ctx.clone(),
        )
        .await
        .expect("chrome status should execute");
    assert_eq!(status.metadata.as_ref().unwrap()["browser"], "chrome");

    let mut open_input = json!({"action": "open", "browser": "chrome", "url": "about:blank"});
    if let Some(profile) = profile.as_deref() {
        open_input["profile"] = json!(profile);
    }
    let opened = tool
        .execute(open_input, ctx.clone())
        .await
        .expect("chrome open should execute");
    assert_eq!(
        opened.metadata.as_ref().unwrap()["backend"],
        "agent_browser"
    );
    if let Some(profile) = profile.as_deref() {
        assert_eq!(opened.metadata.as_ref().unwrap()["profile"], profile);
        assert_eq!(
            opened.metadata.as_ref().unwrap()["credential_bearing_profile"],
            true
        );
    }

    let mut title_input = json!({"action": "get_content", "browser": "chrome", "format": "title"});
    if let Some(profile) = profile.as_deref() {
        title_input["profile"] = json!(profile);
    }
    let title = tool
        .execute(title_input, ctx)
        .await
        .expect("chrome get title should execute");
    assert_eq!(title.metadata.as_ref().unwrap()["browser"], "chrome");

    let mut screenshot_input = json!({"action": "screenshot", "browser": "chrome"});
    if let Some(profile) = profile.as_deref() {
        screenshot_input["profile"] = json!(profile);
    }
    let screenshot = tool
        .execute(
            screenshot_input,
            ToolContext {
                session_id: session_id.clone(),
                message_id: "message".to_string(),
                tool_call_id: "screenshot".to_string(),
                working_dir: None,
                stdin_request_tx: None,
                graceful_shutdown_signal: None,
                execution_mode: jcode_tool_core::ToolExecutionMode::Direct,
            },
        )
        .await
        .expect("chrome screenshot should execute");
    assert_eq!(screenshot.images.len(), 1);

    agent_browser::close_live_session(&session_id, profile.as_deref())
        .await
        .expect("live Chrome session should close");
}
