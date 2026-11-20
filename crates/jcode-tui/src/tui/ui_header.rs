use super::box_utils::render_rounded_box;
use super::changelog::get_unseen_changelog_entries;
use super::{TuiState, dim_color, is_running_stable_release, semver};
use crate::tui::color_support::rgb;
use ratatui::prelude::*;

/// UI chrome is single-line. Budget the same trailing cell as the transcript
/// preparer so even an exactly-full header cannot acquire a continuation row.
fn fit_header_lines(lines: Vec<Line<'static>>, width: u16) -> Vec<Line<'static>> {
    let budget = usize::from(width.saturating_sub(1));
    lines
        .into_iter()
        .map(|mut line| {
            for span in &mut line.spans {
                if span.content.chars().any(char::is_control) {
                    span.content = span
                        .content
                        .chars()
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect::<String>()
                        .into();
                }
            }
            let mut fitted = jcode_tui_render::truncate_line_with_ellipsis_to_width(&line, budget);
            fitted.style = line.style;
            fitted.alignment = line.alignment;
            fitted
        })
        .collect()
}

/// `ws: <world> · <repo>: tracker:…` (+ ` !id` / ` ?id`) for the session's working directory. Runs
/// every frame, so it reads a short-TTL cache and never runs git on the render
/// thread: the first frame has no badge and a later one (via the header
/// signature) picks up the background probe.
pub(super) fn workspace_badge(app: &dyn TuiState) -> Option<String> {
    let dir = app.working_dir()?;
    crate::workspace::badge_cached(std::path::Path::new(&dir))
}

fn standalone_version_line() -> Line<'static> {
    let text = if is_running_stable_release() {
        let tag = jcode_build_meta::git_tag();
        if tag.is_empty() || tag.contains('-') {
            format!("{} · release", semver())
        } else {
            format!("{} · release {}", semver(), tag)
        }
    } else {
        semver().to_string()
    };
    Line::from(Span::styled(text, Style::default().fg(dim_color()))).alignment(Alignment::Left)
}

/// The persisted subset of the real header. Binary versions are omitted when
/// the selected session does not own them.
#[cfg(test)]
pub(crate) fn build_preview_header(
    session: &crate::tui::session_picker::SessionInfo,
    width: u16,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match session.server_name.as_deref() {
        Some(server) => {
            let icon = session
                .server_icon
                .as_deref()
                .map(|icon| format!(" {icon}"))
                .unwrap_or_default();
            lines.push(Line::from(Span::styled(
                format!("🗼 {}{icon}", capitalize(server)),
                Style::default().fg(dim_color()),
            )));
        }
        // Resuming a local transcript runs in this client, so its version
        // belongs to the preview too.
        None => lines.push(standalone_version_line()),
    }
    fit_header_lines(lines, width)
}

#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
fn unseen_changelog_entries_override() -> &'static std::sync::Mutex<Option<Vec<String>>> {
    static OVERRIDE: OnceLock<std::sync::Mutex<Option<Vec<String>>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| std::sync::Mutex::new(None))
}

fn with_unseen_changelog_entries<R>(render: impl FnOnce(&[String]) -> R) -> R {
    #[cfg(test)]
    {
        if let Ok(guard) = unseen_changelog_entries_override().lock()
            && let Some(entries) = guard.as_ref()
        {
            return render(entries);
        }
    }
    render(get_unseen_changelog_entries())
}

// Existing override lifecycle tests request an owned snapshot. Production
// rendering uses the borrowed callback and never takes this test-only copy.
#[cfg(test)]
fn unseen_changelog_entries() -> Vec<String> {
    with_unseen_changelog_entries(<[String]>::to_vec)
}

#[cfg(test)]
pub(crate) fn set_unseen_changelog_entries_override_for_tests(entries: Option<Vec<String>>) {
    let mut guard = unseen_changelog_entries_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = entries;
}

/// Keep an override in place only while its owning render test is in scope.
/// Callers must hold the shared render-state test lock before creating this guard.
#[cfg(test)]
pub(crate) struct ChangelogEntriesOverrideGuard;

#[cfg(test)]
pub(crate) fn scoped_unseen_changelog_entries_override_for_tests(
    entries: Vec<String>,
) -> ChangelogEntriesOverrideGuard {
    set_unseen_changelog_entries_override_for_tests(Some(entries));
    ChangelogEntriesOverrideGuard
}

#[cfg(test)]
impl Drop for ChangelogEntriesOverrideGuard {
    fn drop(&mut self) {
        set_unseen_changelog_entries_override_for_tests(None);
    }
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Compact form of a full build version string: `v0.25.19-dev (abc1234, dirty)`
/// becomes `v0.25.19-dev`. Used for the per-line server/client version labels.
fn compact_version_label(version: &str) -> String {
    let trimmed = version.trim();
    match trimmed.split_once(" (") {
        Some((head, _)) => head.trim().to_string(),
        None => trimmed.to_string(),
    }
}

/// Version label for a `server:`/`client:` header line. Normally compact
/// (semver only); keeps the git-hash suffix when the two sides share a semver
/// but differ by build, so the mismatch is still visible at a glance.
fn header_version_label(version: &str, include_hash: bool) -> String {
    if include_hash {
        version.trim().to_string()
    } else {
        compact_version_label(version)
    }
}

#[cfg(test)]
pub(super) fn build_persistent_header(app: &dyn TuiState, width: u16) -> Vec<Line<'static>> {
    fit_header_lines(persistent_rows(app), width)
}

#[cfg(test)]
pub(crate) fn build_header_lines(app: &dyn TuiState, width: u16) -> Vec<Line<'static>> {
    fit_header_lines(secondary_rows(app), width)
}

fn persistent_rows(app: &dyn TuiState) -> Vec<Line<'static>> {
    let dim = Style::default().fg(dim_color());
    let server_version = app.server_display_version();
    let client_version = app
        .server_display_name()
        .map(|_| jcode_build_meta::version().to_string());
    let Some(server) = server_version.as_deref() else {
        // Standalone: no server/client split, just the running version.
        return vec![standalone_version_line()];
    };
    let client = client_version.as_deref().unwrap_or(server);
    let mismatch = server.trim() != client.trim();
    // Same semver but different builds: keep the hash so the mismatch reads.
    let include_hash = mismatch && compact_version_label(server) == compact_version_label(client);
    let version_style = if mismatch {
        Style::default().fg(rgb(255, 200, 100))
    } else {
        dim
    };
    let mut spans = vec![
        Span::styled("🗼 ", dim),
        Span::styled(header_version_label(server, include_hash), version_style),
        Span::styled(" · 💻 ", dim),
        Span::styled(header_version_label(client, include_hash), version_style),
    ];
    let mut badges = Vec::new();
    if app.server_update_available() == Some(true) {
        badges.push("srv↑");
    }
    if app.client_update_available() {
        badges.push("cli↑");
    }
    if !badges.is_empty() {
        spans.push(Span::styled(format!("  {}", badges.join(" ")), dim));
    }
    vec![Line::from(spans).alignment(Alignment::Left)]
}

fn secondary_rows(app: &dyn TuiState) -> Vec<Line<'static>> {
    let dim = Style::default().fg(dim_color());
    let mut rows: Vec<String> = Vec::new();
    if let Some(mut ws) = workspace_badge(app) {
        if let Some(branch) = app.git_branch() {
            ws.push_str(&format!(" branch:{branch}"));
        }
        rows.push(ws);
    }
    if let Some(dir) = app.working_dir() {
        let dir = std::path::Path::new(&dir);
        rows.extend(crate::tui::header_facts::linear_row(dir));
        rows.extend(crate::tui::header_facts::openspec_row(dir));
    }
    let mut lines: Vec<Line> = rows
        .into_iter()
        .map(|row| Line::from(Span::styled(row, dim)).alignment(Alignment::Left))
        .collect();
    lines.push(Line::from(""));
    lines
}

/// Build the "Updates" rounded box (unseen release notes) so it can be
/// rendered inside the top padding above the header. `max_lines` bounds the
/// total height including the box borders; entries beyond the budget are
/// collapsed into a "…N more" line. Returns an empty vec when there are no
/// unseen entries or the budget/width is too small for a box.
pub(super) fn build_updates_box_lines(width: u16, max_lines: usize) -> Vec<Line<'static>> {
    let w = width as usize;
    if w <= 20 || max_lines < 3 {
        return Vec::new();
    }
    with_unseen_changelog_entries(|entries| updates_box_lines_from_entries(w, max_lines, entries))
}

// Borrow the cached entries only while building owned output. Do not introduce a
// header-side cache: the changelog getter remains responsible for invalidation.
fn updates_box_lines_from_entries(
    w: usize,
    max_lines: usize,
    new_entries: &[String],
) -> Vec<Line<'static>> {
    if new_entries.is_empty() {
        return Vec::new();
    }

    // Budget for content lines inside the box (borders take 2 lines).
    let content_budget = (max_lines - 2).min(8);
    let has_more = new_entries.len() > content_budget;
    let display_count = if has_more {
        content_budget.saturating_sub(1)
    } else {
        new_entries.len()
    };

    let mut content: Vec<Line> = Vec::new();
    for entry in new_entries.iter().take(display_count) {
        content.push(Line::from(Span::styled(
            format!("• {}", entry),
            Style::default().fg(dim_color()),
        )));
    }
    if has_more {
        content.push(Line::from(Span::styled(
            format!(
                "  …{} more · /changelog to see all",
                new_entries.len() - display_count
            ),
            Style::default().fg(dim_color()),
        )));
    }
    if content.is_empty() {
        return Vec::new();
    }

    render_rounded_box(
        "Updates",
        content,
        w.saturating_sub(2),
        Style::default().fg(dim_color()),
    )
    .into_iter()
    .map(|line| line.alignment(Alignment::Left))
    .collect()
}

/// Both header sections. Every input is in memory or a cached background
/// probe, so this never touches credentials, git or the network.
pub(in crate::tui) fn build_header_sections(
    app: &dyn TuiState,
    width: u16,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    // No historical startup snapshot was recorded. Do not manufacture one
    // using today's version and update state.
    if app.is_replay() {
        return (Vec::new(), Vec::new());
    }
    (
        fit_header_lines(persistent_rows(app), width),
        fit_header_lines(secondary_rows(app), width),
    )
}

#[cfg(test)]
#[path = "ui_header/header_tests.rs"]
mod header_tests;

#[cfg(test)]
mod changelog_borrow_tests {
    use super::*;

    #[test]
    fn changelog_borrow_matches_owned_snapshot_and_refreshes_override() {
        let _lock = crate::tui::ui::render_state_test_lock();
        for count in [0, 1, 2, 7, 8, 9, 100] {
            let entries: Vec<String> = (0..count)
                .map(|i| format!("Update {i}: 界 👩‍💻 {}", "long subject ".repeat(12)))
                .collect();
            let snapshot = entries.clone();
            let _override = scoped_unseen_changelog_entries_override_for_tests(entries);
            for width in [0, 20, 21, 40, 80, 240] {
                for height in [0, 2, 3, 4, 9, 10, 20] {
                    let expected = if width <= 20 || height < 3 {
                        Vec::new()
                    } else {
                        updates_box_lines_from_entries(width as usize, height, &snapshot)
                    };
                    assert_eq!(build_updates_box_lines(width, height), expected);
                }
            }
        }
        // A previously rendered result owns its text and survives replacement.
        let _override = scoped_unseen_changelog_entries_override_for_tests(vec!["first".into()]);
        let first = build_updates_box_lines(80, 10);
        set_unseen_changelog_entries_override_for_tests(Some(vec!["second".into()]));
        let second = build_updates_box_lines(80, 10);
        assert_ne!(first, second);
        assert!(first.iter().any(|line| line.to_string().contains("first")));
        assert!(
            second
                .iter()
                .any(|line| line.to_string().contains("second"))
        );
    }

    #[test]
    fn changelog_borrow_eliminates_snapshot_allocations() {
        let _lock = crate::tui::ui::render_state_test_lock();
        let entries: Vec<String> = (0..100).map(|i| format!("entry {i}")).collect();
        let vector_pointer = entries.as_ptr();
        let string_pointers: Vec<_> = entries.iter().map(|entry| entry.as_ptr()).collect();
        let _override = scoped_unseen_changelog_entries_override_for_tests(entries);
        with_unseen_changelog_entries(|borrowed| {
            assert_eq!(borrowed.as_ptr(), vector_pointer);
            for (entry, pointer) in borrowed.iter().zip(&string_pointers) {
                assert_eq!(entry.as_ptr(), *pointer);
            }
            // Count the allocations specifically removed, not the unchanged
            // output allocations: Vec::clone allocates one buffer plus one for
            // each nonempty String. Pointer identity proves none is copied now.
            let old_snapshot = borrowed.to_vec();
            let old_allocations = usize::from(!old_snapshot.is_empty())
                + old_snapshot
                    .iter()
                    .filter(|entry| !entry.is_empty())
                    .count();
            assert_ne!(old_snapshot.as_ptr(), vector_pointer);
            assert_eq!(old_allocations, 101);
            assert_eq!(old_snapshot, borrowed);
            eprintln!(
                "changelog snapshot allocations: old={old_allocations}, borrowed=0 (100 entries)"
            );
        });
    }
}
