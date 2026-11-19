#![cfg_attr(test, allow(clippy::await_holding_lock))]
use super::*;

/// Point the process at a temp jcode home and return it with a restore guard.
fn temp_jcode_home() -> (tempfile::TempDir, Option<std::ffi::OsString>) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let prev = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", dir.path());
    crate::config::Config::invalidate_cache();
    (dir, prev)
}

fn restore_jcode_home(prev: Option<std::ffi::OsString>) {
    if let Some(prev) = prev {
        crate::env::set_var("JCODE_HOME", prev);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
    crate::config::Config::invalidate_cache();
}

#[test]
fn writing_the_active_config_reports_what_changed() {
    let _guard = crate::storage::lock_test_env();
    let (_dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    std::fs::write(&path, "[keybindings]\nscroll_up = \"ctrl+y\"\n").expect("write");

    let notice = config_edit_notice(
        &path,
        "[keybindings]\nscroll_up = \"ctrl+k\"\n",
        "[keybindings]\nscroll_up = \"ctrl+y\"\n",
    )
    .expect("an active-config edit should be reported");

    assert!(notice.contains("keybindings.scroll_up"), "{notice}");
    assert!(notice.contains("live now"), "{notice}");

    restore_jcode_home(prev);
}

#[test]
fn writing_an_unrelated_file_reports_nothing() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let other = dir.path().join("notes.toml");
    std::fs::write(&other, "[display]\ncentered = true\n").expect("write");

    assert!(
        config_edit_notice(&other, "", "[display]\ncentered = true\n").is_none(),
        "only the active config file should get a change report"
    );

    restore_jcode_home(prev);
}

#[test]
fn comment_only_config_edit_reports_nothing() {
    let _guard = crate::storage::lock_test_env();
    let (_dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    let after = "# note\n[display]\ncentered = true\n";
    std::fs::write(&path, after).expect("write");

    assert!(
        config_edit_notice(&path, "[display]\ncentered = true\n", after).is_none(),
        "a comment-only edit must not claim a settings change"
    );

    restore_jcode_home(prev);
}

#[test]
fn restart_required_sections_say_so() {
    let _guard = crate::storage::lock_test_env();
    let (_dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    let after = "[gateway]\nport = 8888\n";
    std::fs::write(&path, after).expect("write");

    let notice =
        config_edit_notice(&path, "[gateway]\nport = 7777\n", after).expect("report expected");
    assert!(
        notice.contains("Restart required for: gateway.port"),
        "{notice}"
    );

    restore_jcode_home(prev);
}

#[test]
fn the_notice_leaves_the_config_cache_current() {
    let _guard = crate::storage::lock_test_env();
    let (_dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    std::fs::write(&path, "[display]\ncentered = false\n").expect("write");
    assert!(!crate::config::config().display.centered);

    // Rewrite immediately: without the notice's explicit invalidation this can
    // land inside the config cache's staleness throttle.
    let after = "[display]\ncentered = true\n";
    std::fs::write(&path, after).expect("rewrite");
    let notice =
        config_edit_notice(&path, "[display]\ncentered = false\n", after).expect("report expected");

    assert!(notice.contains("live now"), "{notice}");
    assert!(
        crate::config::config().display.centered,
        "claiming 'live now' requires the config cache to already reflect the edit"
    );

    restore_jcode_home(prev);
}

#[test]
fn a_config_write_that_breaks_toml_syntax_is_reported_loudly() {
    let _guard = crate::storage::lock_test_env();
    let (_dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    let broken = "[display\ncentered = true\n";
    std::fs::write(&path, broken).expect("write");

    let notice = config_edit_notice(&path, "[display]\ncentered = true\n", broken)
        .expect("a config file that stopped parsing must never be silent");
    assert!(notice.contains("WARNING"), "{notice}");
    assert!(notice.contains("no longer load"), "{notice}");
    assert!(notice.contains("falling back to default"), "{notice}");

    restore_jcode_home(prev);
}

/// End-to-end through the real `write` tool: the path an agent actually takes
/// when a user says "change this setting".
#[tokio::test]
async fn the_write_tool_reports_config_changes_end_to_end() {
    use crate::tool::{Tool, ToolContext};

    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    std::fs::write(
        &path,
        "[display]\ncentered = false\n\n[gateway]\nport = 7777\n",
    )
    .expect("seed config");
    assert!(!crate::config::config().display.centered);

    let ctx = ToolContext {
        session_id: "test".to_string(),
        message_id: "test".to_string(),
        tool_call_id: "test".to_string(),
        working_dir: Some(dir.path().to_path_buf()),
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };

    let output = crate::tool::write::WriteTool
        .execute(
            serde_json::json!({
                "file_path": path.to_string_lossy(),
                "content": "[display]\ncentered = true\n\n[gateway]\nport = 8888\n",
            }),
            ctx,
        )
        .await
        .expect("write should succeed");

    let body = output.output;
    assert!(body.contains("display.centered"), "{body}");
    assert!(body.contains("live now"), "{body}");
    assert!(
        body.contains("Restart required for: gateway.port"),
        "{body}"
    );
    assert!(
        crate::config::config().display.centered,
        "the display change should be live in-process immediately after the write"
    );

    restore_jcode_home(prev);
}

/// `apply_patch` reaches config.toml through its own write paths, so it gets
/// the same report as write/edit.
#[tokio::test]
async fn apply_patch_reports_config_changes() {
    use crate::tool::{Tool, ToolContext};

    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let path = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    std::fs::write(&path, "[display]\ncentered = false\n").expect("seed config");
    assert!(!crate::config::config().display.centered);

    let ctx = ToolContext {
        session_id: "test".to_string(),
        message_id: "test".to_string(),
        tool_call_id: "test".to_string(),
        working_dir: Some(dir.path().to_path_buf()),
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: crate::tool::ToolExecutionMode::Direct,
    };

    let patch_text = format!(
        "*** Begin Patch\n*** Update File: {}\n@@\n-centered = false\n+centered = true\n*** End Patch\n",
        path.display()
    );
    let output = crate::tool::apply_patch::ApplyPatchTool
        .execute(serde_json::json!({ "patch_text": patch_text }), ctx)
        .await
        .expect("patch should apply");

    let body = output.output;
    assert!(body.contains("display.centered"), "{body}");
    assert!(body.contains("live now"), "{body}");
    assert!(
        crate::config::config().display.centered,
        "the patched setting should be live immediately"
    );

    restore_jcode_home(prev);
}

#[test]
fn fragment_edit_is_reported_and_broken_fragment_warns() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    std::fs::write(&root, "include = [\"config/ui.toml\"]\n").expect("root");
    let fragment = dir.path().join("config/ui.toml");
    std::fs::write(&fragment, "[display]\ncentered = true\n").expect("fragment");

    let notice = config_edit_notice(
        &fragment,
        "[display]\ncentered = false\n",
        "[display]\ncentered = true\n",
    )
    .expect("a listed fragment edit should be reported");
    assert!(notice.contains("display.centered"), "{notice}");
    assert!(notice.contains("live now"), "{notice}");

    // A fragment the manifest does not list is just another file.
    let unlisted = dir.path().join("config/other.toml");
    std::fs::write(&unlisted, "[display]\ncentered = true\n").expect("unlisted");
    assert!(config_edit_notice(&unlisted, "", "[display]\ncentered = true\n").is_none());

    // Breaking the fragment warns, names the fragment, and never echoes values.
    std::fs::write(&fragment, "[display\ncentered = \"s3cret\"\n").expect("break");
    let notice = config_edit_notice(&fragment, "", "[display\n")
        .expect("a fragment that stopped parsing must never be silent");
    assert!(notice.contains("WARNING"), "{notice}");
    assert!(notice.contains("ui.toml"), "{notice}");
    assert!(!notice.contains("s3cret"), "{notice}");

    restore_jcode_home(prev);
}

#[test]
fn config_edit_watch_covers_fragments_and_new_includes() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    std::fs::write(&root, "include = [\"config/ui.toml\"]\n").expect("root");
    let ui = dir.path().join("config/ui.toml");
    std::fs::write(&ui, "[display]\ncentered = false\n").expect("ui");

    let watch = ConfigEditWatch::begin();
    std::fs::write(&ui, "[display]\ncentered = true\n").expect("edit fragment");
    let mut body = String::new();
    watch.finish(&mut body);
    assert!(body.contains("display.centered"), "{body}");

    // Root edit that lists a fragment created during the same tool call.
    let watch = ConfigEditWatch::begin();
    std::fs::write(
        dir.path().join("config/extra.toml"),
        "[display]\nshow_thinking = true\n",
    )
    .expect("extra");
    std::fs::write(
        &root,
        "include = [\"config/ui.toml\", \"config/extra.toml\"]\n",
    )
    .expect("root");
    let mut body = String::new();
    watch.finish(&mut body);
    assert!(body.contains("include"), "{body}");
    assert!(body.contains("display.show_thinking"), "{body}");

    restore_jcode_home(prev);
}

#[test]
fn masked_fragment_edit_does_not_claim_live_effect() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    std::fs::write(
        &root,
        "include = [\"config/a.toml\", \"config/b.toml\"]\n[display]\ncentered = false\n",
    )
    .expect("root");
    let a = dir.path().join("config/a.toml");
    let b = dir.path().join("config/b.toml");
    std::fs::write(
        &a,
        "[display]\ncentered = true\n[keybindings]\nscroll_up = \"ctrl+y\"\n",
    )
    .expect("a");
    std::fs::write(&b, "[gateway]\nport = 1\n").expect("b");

    // Root overrides display.centered: editing it in `a` changes nothing.
    let notice = config_edit_notice(
        &a,
        "[display]\ncentered = false\n",
        "[display]\ncentered = true\n",
    )
    .expect("masked edit still gets a note");
    assert!(!notice.contains("live now"), "{notice}");
    assert!(notice.contains("overridden"), "{notice}");

    // A key nothing later sets is reported as live.
    let notice = config_edit_notice(
        &a,
        "[keybindings]\nscroll_up = \"ctrl+k\"\n",
        "[keybindings]\nscroll_up = \"ctrl+y\"\n",
    )
    .expect("unmasked edit reported");
    assert!(notice.contains("keybindings.scroll_up"), "{notice}");
    assert!(notice.contains("live now"), "{notice}");

    // A later fragment masks an earlier one.
    std::fs::write(&b, "[keybindings]\nscroll_up = \"ctrl+u\"\n").expect("b");
    let notice = config_edit_notice(
        &a,
        "[keybindings]\nscroll_up = \"ctrl+k\"\n",
        "[keybindings]\nscroll_up = \"ctrl+y\"\n",
    )
    .expect("note");
    assert!(notice.contains("overridden"), "{notice}");

    restore_jcode_home(prev);
}

#[test]
fn masking_compares_the_exact_key_not_a_prefix_overlap() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    // The root sets a sibling and a longer key sharing a prefix with the
    // edited key, but never the edited key itself.
    std::fs::write(
        &root,
        "include = [\"config/a.toml\"]\n[display]\ncentered_extra = true\n[agents]\nswarm_model_x = \"x\"\n",
    )
    .expect("root");
    let a = dir.path().join("config/a.toml");
    std::fs::write(
        &a,
        "[display]\ncentered = true\n[agents]\nswarm_model = \"m2\"\n",
    )
    .expect("a");

    let notice = config_edit_notice(
        &a,
        "[display]\ncentered = false\n[agents]\nswarm_model = \"m1\"\n",
        "[display]\ncentered = true\n[agents]\nswarm_model = \"m2\"\n",
    )
    .expect("notice");
    assert!(!notice.contains("overridden"), "{notice}");
    assert!(notice.contains("`display.centered`"), "{notice}");
    assert!(notice.contains("`agents.swarm_model`"), "{notice}");

    // The winning source for that exact key does mask it.
    std::fs::write(
        &root,
        "include = [\"config/a.toml\"]\n[display]\ncentered = false\n",
    )
    .expect("root");
    let notice = config_edit_notice(
        &a,
        "[display]\ncentered = false\n",
        "[display]\ncentered = true\n",
    )
    .expect("notice");
    assert!(notice.contains("overridden"), "{notice}");

    restore_jcode_home(prev);
}

#[test]
fn removing_a_fragment_from_the_manifest_reports_and_deactivates_it() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    std::fs::write(
        &root,
        "include = [\"config/ui.toml\", \"config/more.toml\"]\n",
    )
    .expect("root");
    let ui = dir.path().join("config/ui.toml");
    let more = dir.path().join("config/more.toml");
    std::fs::write(&ui, "[display]\ncentered = true\n").expect("ui");
    std::fs::write(&more, "[display]\nshow_thinking = true\n").expect("more");

    let watch = ConfigEditWatch::begin();
    std::fs::write(&root, "include = [\"config/ui.toml\"]\n").expect("drop more.toml");
    let mut body = String::new();
    watch.finish(&mut body);
    assert!(
        body.contains("`include`"),
        "manifest edit is reported: {body}"
    );
    assert!(body.contains("live now"), "{body}");

    // The dropped fragment is an ordinary file again: no claims about it.
    assert!(
        config_edit_notice(
            &more,
            "[display]\nshow_thinking = true\n",
            "[display]\nshow_thinking = false\n"
        )
        .is_none(),
        "an unlisted fragment must not get a config notice"
    );
    // Edits to the remaining fragment still report.
    assert!(
        config_edit_notice(
            &ui,
            "[display]\ncentered = false\n",
            "[display]\ncentered = true\n"
        )
        .is_some()
    );

    restore_jcode_home(prev);
}

#[test]
fn valid_edit_notices_never_echo_secret_values() {
    let _guard = crate::storage::lock_test_env();
    let (dir, prev) = temp_jcode_home();

    let root = crate::config::Config::path().expect("config path");
    std::fs::create_dir_all(dir.path().join("config")).expect("mkdir");
    std::fs::write(&root, "include = [\"config/p.toml\"]\n").expect("root");
    let fragment = dir.path().join("config/p.toml");
    let before = "[providers.acme]\napi_key = \"sentinel-old-secret\"\nbase_url = \"https://acme.example/v1\"\n";
    let after = "[providers.acme]\napi_key = \"sentinel-new-secret\"\nbase_url = \"https://acme.example/v2\"\n";
    std::fs::write(&fragment, after).expect("fragment");

    let notice = config_edit_notice(&fragment, before, after).expect("fragment notice");
    assert!(!notice.contains("sentinel"), "{notice}");
    assert!(notice.contains("providers.acme.api_key"), "{notice}");
    assert!(notice.contains("https://acme.example/v2"), "{notice}");

    // Same through the root file and the multi-file watch.
    std::fs::write(
        &root,
        "include = [\"config/p.toml\"]\n[auth]\nsome_token = \"sentinel-root-token\"\n",
    )
    .expect("root");
    let notice = config_edit_notice(
        &root,
        "include = [\"config/p.toml\"]\n",
        "include = [\"config/p.toml\"]\n[auth]\nsome_token = \"sentinel-root-token\"\n",
    )
    .expect("root notice");
    assert!(!notice.contains("sentinel"), "{notice}");
    assert!(notice.contains("auth.some_token"), "{notice}");

    // A broken edit carrying literal and numeric values stays value-free too.
    std::fs::write(
        &fragment,
        "[display]\ncentered = 'sentinel-lit'\nshow_thinking = 55555\n",
    )
    .expect("bad");
    let notice = config_edit_notice(&fragment, "", "x").expect("warning");
    assert!(notice.contains("WARNING"), "{notice}");
    assert!(
        !notice.contains("sentinel") && !notice.contains("55555"),
        "{notice}"
    );

    restore_jcode_home(prev);
}
