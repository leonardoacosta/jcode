use super::*;
use crate::message::Message;
use crate::provider::{EventStream, Provider};
use crate::tool::Registry;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::OnceLock;

#[test]
fn header_rows_clip_unicode_and_controls_without_losing_style() {
    let style = Style::default().fg(Color::Magenta);
    let source = vec![
        Line::from(vec![
            Span::styled("账号 👩‍💻 e\u{301} ", style),
            Span::raw("/very/long/path\nwith\ttabs and a long account email@example.com"),
        ])
        .alignment(Alignment::Left),
    ];
    for width in [0u16, 1, 2, 3, 8, 18, 40, 80] {
        let fitted = fit_header_lines(source.clone(), width);
        assert_eq!(fitted.len(), 1);
        assert!(fitted[0].width() <= usize::from(width.saturating_sub(1)));
        assert_eq!(fitted[0].alignment, Some(Alignment::Left));
        assert!(
            fitted[0]
                .spans
                .iter()
                .all(|span| !span.content.chars().any(char::is_control))
        );
        if width >= 8 {
            assert_eq!(fitted[0].spans[0].style, style);
        }
    }
}

#[test]
fn startup_header_never_adds_wraparound_rows_at_narrow_widths() {
    let app = create_test_app();
    for width in [2u16, 8, 18, 32, 40, 60, 80, 120] {
        let (mut lines, secondary) = build_header_sections(&app, width);
        lines.extend(secondary);
        for line in lines {
            let budget = usize::from(width.saturating_sub(1));
            assert!(line.width() <= budget, "width={width}: {line:?}");
            let wrapped = jcode_tui_markdown::wrap_line(line, budget);
            assert_eq!(
                wrapped.len(),
                1,
                "header must stay one visual row at width={width}"
            );
        }
    }
}

#[test]
fn changelog_override_is_cleared_after_a_render_test_panics() {
    let _lock = crate::tui::ui::render_state_test_lock();
    let panic = std::panic::catch_unwind(|| {
        let _fixture = scoped_unseen_changelog_entries_override_for_tests(vec![
            "temporary changelog entry".to_owned(),
        ]);
        assert_eq!(unseen_changelog_entries(), ["temporary changelog entry"]);
        panic!("injected render failure");
    });
    assert!(panic.is_err());
    assert!(
        unseen_changelog_entries_override()
            .lock()
            .unwrap()
            .is_none(),
        "a failed render test must not leak its changelog fixture"
    );
}

struct MockProvider;

#[async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        _messages: &[Message],
        _tools: &[crate::message::ToolDefinition],
        _system: &str,
        _resume_session_id: Option<&str>,
    ) -> Result<EventStream> {
        Err(anyhow::anyhow!(
            "Mock provider should not be used for streaming completions in ui header tests"
        ))
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(MockProvider)
    }
}

fn ensure_test_jcode_home_if_unset() {
    static TEST_HOME: OnceLock<std::path::PathBuf> = OnceLock::new();

    if std::env::var_os("JCODE_HOME").is_some() {
        return;
    }

    let path = TEST_HOME.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("jcode-test-home-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&path);
        path
    });
    crate::env::set_var("JCODE_HOME", path);
}

fn create_test_app() -> crate::tui::app::App {
    ensure_test_jcode_home_if_unset();

    let provider: Arc<dyn Provider> = Arc::new(MockProvider);
    let rt = tokio::runtime::Runtime::new().expect("test runtime");
    let registry = rt.block_on(Registry::new(provider.clone()));
    crate::tui::app::App::new_for_test_harness(provider, registry)
}

#[test]
fn combined_header_sections_match_individual_builders() {
    let app = create_test_app();
    let (persistent, secondary) = build_header_sections(&app, 80);

    assert_eq!(persistent, build_persistent_header(&app, 80));
    assert_eq!(secondary, build_header_lines(&app, 80));
}

fn rendered_header_lines(app: &crate::tui::app::App, width: u16) -> Vec<String> {
    build_persistent_header(app, width)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect()
}

#[test]
fn remote_header_is_one_server_client_row_without_session_name() {
    let mut app = create_test_app();
    let client = compact_version_label(jcode_build_meta::version());
    app.set_remote_server_identity_for_tests(
        Some("blazing"),
        Some("🔥"),
        Some(jcode_build_meta::version()),
        Some("session_fox_1705012345678"),
    );
    let rows = rendered_header_lines(&app, 120);
    assert_eq!(rows, vec![format!("🗼 {client} · 💻 {client}")], "{rows:?}");
    let all = {
        let (p, s) = build_header_sections(&app, 120);
        p.iter()
            .chain(s.iter())
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    };
    for gone in [
        "server:",
        "client:",
        "Fox",
        "🦊",
        "Blazing",
        "/model to switch",
        "Account router",
        "No auto-routing",
        "mcp ",
        "skills:",
    ] {
        assert!(!all.contains(gone), "{gone:?} must not render: {all}");
    }
}

#[test]
fn version_mismatch_highlights_both_labels_and_keeps_hash_on_same_semver() {
    let mut app = create_test_app();
    let client_semver = compact_version_label(jcode_build_meta::version());
    app.set_remote_server_identity_for_tests(
        Some("blazing"),
        None,
        Some(&format!("{client_semver} (0000000)")),
        Some("session_fox_1705012345678"),
    );
    let lines = build_persistent_header(&app, 200);
    let row = lines[0].to_string();
    assert!(row.contains("(0000000)"), "{row}");
    assert!(row.contains(jcode_build_meta::version().trim()), "{row}");
    let highlighted = lines[0]
        .spans
        .iter()
        .filter(|span| span.style.fg == Some(rgb(255, 200, 100)))
        .count();
    assert_eq!(highlighted, 2, "{:?}", lines[0]);
}

#[test]
fn local_header_shows_only_the_running_version() {
    let app = create_test_app();
    let rows = rendered_header_lines(&app, 120);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(rows[0].starts_with(semver()), "{rows:?}");
    assert!(!rows[0].contains('🗼'), "{rows:?}");
}

#[test]
fn compact_version_label_strips_hash_suffix() {
    assert_eq!(
        compact_version_label("v0.25.19-dev (7e261bcc, dirty)"),
        "v0.25.19-dev"
    );
    assert_eq!(compact_version_label("v0.25.19 (abc1234)"), "v0.25.19");
    assert_eq!(compact_version_label(" v0.25.19 "), "v0.25.19");
}

fn git_in(dir: &std::path::Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("git")
        .success();
    assert!(ok, "git {args:?}");
}

#[test]
fn workspace_badge_renders_on_one_row_without_wrapping_at_narrow_widths() {
    let _env = crate::storage::lock_test_env();
    let home = tempfile::TempDir::new().unwrap();
    let previous = [
        ("JCODE_HOME", std::env::var_os("JCODE_HOME")),
        ("GIT_CONFIG_GLOBAL", std::env::var_os("GIT_CONFIG_GLOBAL")),
    ];
    crate::env::set_var("JCODE_HOME", home.path());
    std::fs::write(home.path().join("gitconfig"), "").unwrap();
    crate::env::set_var("GIT_CONFIG_GLOBAL", home.path().join("gitconfig"));
    std::fs::write(
        home.path().join("config.toml"),
        "[[workspaces.rules]]\nname = \"brown\"\norigin = \"dev.azure.com/brown/**\"\nexpected_email = \"me@brown.example\"\n",
    )
    .unwrap();
    crate::config::Config::invalidate_cache();
    let repo = home.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git_in(&repo, &["init", "-q"]);
    git_in(
        &repo,
        &[
            "remote",
            "add",
            "origin",
            "https://tok@dev.azure.com/brown/p/_git/r",
        ],
    );

    let mut app = create_test_app();
    app.set_centered(false);
    app.set_working_dir_for_tests(&repo.display().to_string());

    // The first frames never block on git; the probe lands asynchronously.
    let secondary = |app: &crate::tui::app::App, width: u16| -> Vec<String> {
        build_header_lines(app, width)
            .iter()
            .map(|l| l.to_string())
            .collect()
    };
    let mut rows = secondary(&app, 80);
    for _ in 0..200 {
        if rows.iter().any(|row| row.contains("ws: brown")) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
        rows = secondary(&app, 80);
    }
    assert!(
        rows[0].starts_with("ws: brown · repo:"),
        "ws row first: {rows:?}"
    );
    assert!(rows[0].contains("?id"), "unset email is flagged: {rows:?}");
    assert!(!rows.iter().any(|row| row.contains("tok")), "{rows:?}");

    // One logical row stays one visual row at every width, truncated by cells.
    let baseline = secondary(&app, 80).len();
    for width in [80u16, 40, 24, 16, 10, 6, 3] {
        let lines = build_header_lines(&app, width);
        assert_eq!(lines.len(), baseline, "row count at width {width}");
        for line in &lines {
            assert!(line.width() <= usize::from(width), "{width}: {line:?}");
        }
    }
    assert!(unicode_width::UnicodeWidthStr::width(secondary(&app, 12)[0].as_str()) <= 12);

    // A badge change invalidates the prepared-header cache signature.
    let before = crate::tui::ui::prepare::header_prep_signature(&app, 80);
    git_in(&repo, &["config", "user.email", "me@brown.example"]);
    for _ in 0..200 {
        if workspace_badge(&app).is_some_and(|b| !b.contains("?id")) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert_eq!(workspace_badge(&app).as_deref(), Some("ws: brown · repo:"));
    assert_ne!(
        before,
        crate::tui::ui::prepare::header_prep_signature(&app, 80),
        "identity change must feed the header signature"
    );
    assert!(!secondary(&app, 80)[0].contains("?id"));

    for (key, value) in previous {
        match value {
            Some(value) => crate::env::set_var(key, value),
            None => crate::env::remove_var(key),
        }
    }
    crate::config::Config::invalidate_cache();
}
