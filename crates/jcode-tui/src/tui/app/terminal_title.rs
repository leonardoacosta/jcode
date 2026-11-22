//! Cheap, session-scoped metadata for the terminal's native tabs.
//!
//! Diff counts are cumulative edit-output lines, matching the transcript's diff
//! badges, not a repository-wide `git diff` (other sessions may share the repo).
//! Time is the current/last observed working episode (including continuations),
//! never the age of an open tab or a fabricated lifetime total.
use super::{App, DisplayMessage};
use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct TerminalTitleState {
    pub base: String,
    pub session_id: String,
    pub last_sent: String,
    pub last_refresh: Option<Instant>,
    pub last_work: Option<Duration>,
    /// First `requires_user_auth` reason, re-read at most once a second:
    /// todo state may come from disk.
    pub herdr_user_auth: Option<(Instant, Option<String>)>,
}

/// Swarm statuses in which a worker is actively running, as the dock ranks them.
fn swarm_status_is_running(status: &str) -> bool {
    matches!(status, "running" | "streaming" | "thinking")
}

pub(super) fn edit_line_counts(message: &DisplayMessage) -> (usize, usize) {
    let Some(tool) = message.tool_data.as_ref().filter(|tool| {
        message.role == "tool"
            && crate::tui::ui::tools_ui::is_edit_tool_name(&tool.name)
            && jcode_tui_tool_display::concise_tool_error_summary(&message.content).is_none()
    }) else {
        return (0, 0);
    };
    crate::tui::ui_diff::diff_change_counts_for_tool(tool, &message.content)
}

fn work_label(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs >= 3600 {
        format!("{}h{:02}m", secs / 3600, secs / 60 % 60)
    } else if secs >= 60 {
        format!("{}m{:02}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

fn title_with_metrics(
    base: &str,
    counts: (usize, usize),
    work: Option<Duration>,
    active: bool,
) -> String {
    let mut title = base.to_owned();
    if counts != (0, 0) {
        title.push_str(&format!(" · +{} -{}", counts.0, counts.1));
    }
    if let Some(work) = work {
        title.push_str(&format!(
            " · {} ~{}",
            if active { "work" } else { "last" },
            work_label(work)
        ));
    } else if active {
        title.push_str(" · working");
    }
    title
}

impl App {
    /// Tell herdr (when running in a herdr pane) whether this client is
    /// blocked, working or idle and which session to resume. Tick-safe:
    /// todo state is re-read at most once a second, unchanged state is a
    /// cheap comparison, and reports are sent off-thread.
    pub(super) fn sync_herdr_agent_state(&self) {
        if self.suppress_terminal_title_updates || !self.is_remote_client() {
            return;
        }
        if !crate::tui::herdr::active() {
            return;
        }
        let (state, message) = self.herdr_state();
        crate::tui::herdr::sync(state, message.as_deref(), self.active_client_session_id());
    }

    pub(super) fn herdr_state(&self) -> (crate::tui::herdr::AgentState, Option<String>) {
        let session_id = self.active_client_session_id();
        let user_auth_reason = self.herdr_user_auth_reason();
        let processing = self.is_processing();
        // The subtree walk clones members; skip it when its answer cannot
        // change the state or no member runs at all.
        let owned_worker_running = user_auth_reason.is_none()
            && !processing
            && self
                .remote_swarm_members
                .iter()
                .any(|member| swarm_status_is_running(&member.status))
            && session_id.is_some_and(|self_id| {
                super::tui_state::filter_inline_swarm_subtree(&self.remote_swarm_members, self_id)
                    .iter()
                    .any(|member| swarm_status_is_running(&member.status))
            });
        let wake_task_running = self
            .background_task_rows_ref()
            .iter()
            .any(|task| task.wake && task.status == crate::tui::BackgroundTaskRowStatus::Running);
        crate::tui::herdr::derive_state(crate::tui::herdr::StateInputs {
            processing,
            user_auth_reason: user_auth_reason.as_deref(),
            owned_worker_running,
            wake_task_running,
        })
    }

    fn herdr_user_auth_reason(&self) -> Option<String> {
        const REFRESH: Duration = Duration::from_secs(1);
        let mut title = self.terminal_title.borrow_mut();
        if let Some((at, reason)) = &title.herdr_user_auth
            && at.elapsed() < REFRESH
        {
            return reason.clone();
        }
        let state = self.current_todo_state();
        let reason = state
            .0
            .iter()
            .find(|todo| crate::todo::todo_requires_user_auth(todo))
            .and_then(|todo| todo.requires_user_auth.as_ref())
            .map(|auth| auth.reason.clone());
        title.herdr_user_auth = Some((Instant::now(), reason.clone()));
        reason
    }

    pub(super) fn set_terminal_title_base(&self, session_id: &str, base: String) {
        let mut state = self.terminal_title.borrow_mut();
        if state.session_id != session_id {
            *state = TerminalTitleState::default();
            state.session_id = session_id.to_owned();
        }
        state.base = base;
        state.last_refresh = None;
        drop(state);
        self.refresh_terminal_title_metrics();
    }

    /// Tick-safe: no filesystem access, transcript scans, or subprocesses.
    /// At most one OSC title update per second, and only if the text changed.
    pub(super) fn refresh_terminal_title_metrics(&self) {
        if self.suppress_terminal_title_updates {
            return;
        }
        let mut state = self.terminal_title.borrow_mut();
        if state.base.is_empty()
            || state
                .last_refresh
                .is_some_and(|at| at.elapsed() < Duration::from_secs(1))
        {
            return;
        }
        state.last_refresh = Some(Instant::now());
        let active = self.is_processing();
        let work = if active {
            self.display_turn_duration_secs()
                .filter(|secs| secs.is_finite() && *secs >= 0.0)
                .map(Duration::from_secs_f32)
        } else {
            state.last_work
        };
        if active && work.is_some() {
            state.last_work = work;
        }
        let title = title_with_metrics(&state.base, self.display_edit_line_counts, work, active);
        if state.last_sent != title
            && crossterm::execute!(std::io::stdout(), crossterm::terminal::SetTitle(&title)).is_ok()
        {
            state.last_sent = title;
        }
    }

    pub(super) fn remember_terminal_title_work(&self) {
        if let Some(secs) = self
            .display_turn_duration_secs()
            .filter(|secs| secs.is_finite() && *secs >= 0.0)
        {
            let mut state = self.terminal_title.borrow_mut();
            state.last_work = Some(Duration::from_secs_f32(secs));
            state.last_refresh = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ToolCall;

    #[test]
    fn terminal_title_counts_follow_append_remove_and_history_replace() {
        let mut app = super::super::tests::create_test_app();
        app.suppress_terminal_title_updates = true;
        let message = DisplayMessage::tool(
            "@@ -1 +1 @@\n-old\n+new\n+extra",
            ToolCall {
                id: "edit-1".into(),
                name: "edit".into(),
                input: serde_json::json!({"file_path": "example.rs"}),
                intent: None,
                thought_signature: None,
            },
        );
        app.push_display_message(message.clone());
        assert_eq!(app.display_edit_line_counts, (2, 1));
        app.push_display_message(DisplayMessage::user("continue"));
        assert_eq!(app.display_edit_line_counts, (2, 1));
        app.adjust_display_message_stats(&message, false);
        assert_eq!(app.display_edit_line_counts, (0, 0));
        app.replace_display_messages(vec![message.clone(), message]);
        assert_eq!(app.display_edit_line_counts, (4, 2));
        app.replace_display_messages(vec![]);
        assert_eq!(app.display_edit_line_counts, (0, 0));
    }

    #[test]
    fn terminal_title_work_freezes_and_resets_on_session_switch() {
        let mut app = super::super::tests::create_test_app();
        app.suppress_terminal_title_updates = true;
        app.set_terminal_title_base("first", "First task".into());
        app.visible_turn_started = Some(Instant::now() - Duration::from_secs(125));
        app.clear_visible_turn_started();
        let completed = app.terminal_title.borrow().last_work.unwrap();
        assert_eq!(completed.as_secs(), 125);
        app.remember_terminal_title_work();
        assert_eq!(app.terminal_title.borrow().last_work, Some(completed));
        app.set_terminal_title_base("first", "Renamed task".into());
        assert_eq!(app.terminal_title.borrow().last_work, Some(completed));
        app.set_terminal_title_base("second", "Second task".into());
        assert_eq!(app.terminal_title.borrow().last_work, None);
        assert!(app.terminal_title.borrow().last_sent.is_empty());
    }

    #[test]
    fn title_metrics_show_edits_and_current_or_last_turn() {
        let base = "🦊 Improve session tabs";
        assert_eq!(title_with_metrics(base, (0, 0), None, false), base);
        assert_eq!(
            title_with_metrics(base, (12, 3), Some(Duration::from_secs(125)), true),
            "🦊 Improve session tabs · +12 -3 · work ~2m05s"
        );
        assert_eq!(
            title_with_metrics(base, (0, 4), Some(Duration::from_secs(125)), false),
            "🦊 Improve session tabs · +0 -4 · last ~2m05s"
        );
        assert_eq!(
            title_with_metrics(base, (0, 0), None, true),
            "🦊 Improve session tabs · working"
        );
    }

    #[test]
    fn work_duration_boundaries_are_compact() {
        for (secs, expected) in [
            (0, "0s"),
            (59, "59s"),
            (60, "1m00s"),
            (3599, "59m59s"),
            (3600, "1h00m"),
            (90061, "25h01m"),
        ] {
            assert_eq!(work_label(Duration::from_secs(secs)), expected);
        }
    }

    #[test]
    fn edit_counts_ignore_failures_and_non_edit_output() {
        let mut message = DisplayMessage::tool(
            "@@ -1 +1 @@\n-old\n+new\n+extra",
            ToolCall {
                id: "edit-1".into(),
                name: "edit".into(),
                input: serde_json::json!({"file_path": "example.rs"}),
                intent: None,
                thought_signature: None,
            },
        );
        assert_eq!(edit_line_counts(&message), (2, 1));
        message.tool_data.as_mut().unwrap().name = "bash".into();
        assert_eq!(edit_line_counts(&message), (0, 0));
        message.tool_data.as_mut().unwrap().name = "edit".into();
        message.content = "Error: could not apply patch\n-old\n+new".into();
        assert_eq!(edit_line_counts(&message), (0, 0));
    }

    fn swarm_member(
        id: &str,
        status: &str,
        parent: Option<&str>,
    ) -> crate::protocol::SwarmMemberStatus {
        crate::protocol::SwarmMemberStatus {
            session_id: id.to_string(),
            friendly_name: None,
            status: status.to_string(),
            detail: None,
            task_label: None,
            role: None,
            is_headless: Some(true),
            live_attachments: None,
            status_age_secs: None,
            output_tail: None,
            report_back_to_session_id: parent.map(str::to_string),
            todo_progress: None,
            todo_items: Vec::new(),
            runtime: crate::protocol::SwarmMemberRuntime::default(),
        }
    }

    fn set_remote_todos(app: &mut App, todos: serde_json::Value) {
        let session = app.active_client_session_id().unwrap().to_string();
        app.remote_todos_payload = Some((
            session,
            serde_json::json!({"todos": todos, "plan": {}, "goals": []}).to_string(),
        ));
        app.terminal_title.borrow_mut().herdr_user_auth = None;
    }

    fn todo(id: &str, status: &str, extra: serde_json::Value) -> serde_json::Value {
        let mut todo = serde_json::json!({
            "id": id, "content": id, "status": status, "priority": "high"
        });
        todo.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        todo
    }

    #[test]
    fn herdr_state_follows_user_auth_todos_workers_and_wake_tasks() {
        use crate::tui::herdr::AgentState::{Blocked, Idle, Working};
        let mut app = super::super::tests::create_test_app();
        let self_id = app.active_client_session_id().unwrap().to_string();
        set_remote_todos(&mut app, serde_json::json!([]));
        assert_eq!(app.herdr_state(), (Idle, None));

        // Dependency waits do not block the pane; a login todo does.
        set_remote_todos(
            &mut app,
            serde_json::json!([
                todo(
                    "dep",
                    "pending",
                    serde_json::json!({"blocked_by": ["other"]})
                ),
                todo(
                    "wait",
                    "pending",
                    serde_json::json!({"waiting_on": ["bg1"]})
                ),
            ]),
        );
        assert_eq!(app.herdr_state(), (Idle, None));
        let reason = "Type login password into the Unlock Login Keyring prompt";
        set_remote_todos(
            &mut app,
            serde_json::json!([
                todo(
                    "done",
                    "completed",
                    serde_json::json!({"requires_user_auth": {"reason": "stale"}})
                ),
                todo(
                    "login",
                    "in_progress",
                    serde_json::json!({"requires_user_auth": {"reason": reason}})
                ),
            ]),
        );
        assert_eq!(app.herdr_state(), (Blocked, Some(reason.to_string())));
        set_remote_todos(
            &mut app,
            serde_json::json!([todo(
                "login",
                "completed",
                serde_json::json!({"requires_user_auth": {"reason": reason}})
            )]),
        );
        assert_eq!(app.herdr_state(), (Idle, None));

        // Only running workers in this session's own spawn subtree count.
        app.remote_swarm_members = vec![
            swarm_member("stranger", "running", None),
            swarm_member("blocked-child", "blocked", Some(&self_id)),
        ];
        assert_eq!(app.herdr_state(), (Idle, None));
        for status in ["running", "streaming", "thinking"] {
            app.remote_swarm_members = vec![
                swarm_member("child", "completed", Some(&self_id)),
                swarm_member("grandchild", status, Some("child")),
            ];
            assert_eq!(app.herdr_state(), (Working, None), "{status}");
        }
        app.remote_swarm_members.clear();

        // Only running background tasks that wake this session count.
        let row = |wake, status| crate::tui::BackgroundTaskRow {
            task_id: "bg".into(),
            label: "sleep".into(),
            percent: None,
            status,
            completed_at: None,
            wake,
        };
        app.background_task_rows = vec![row(false, crate::tui::BackgroundTaskRowStatus::Running)];
        assert_eq!(app.herdr_state(), (Idle, None));
        app.background_task_rows = vec![row(true, crate::tui::BackgroundTaskRowStatus::Completed)];
        assert_eq!(app.herdr_state(), (Idle, None));
        app.background_task_rows = vec![row(true, crate::tui::BackgroundTaskRowStatus::Running)];
        assert_eq!(app.herdr_state(), (Working, None));
        // Finishing through the shared mutator (every completion path) idles.
        app.finish_background_task(
            "bg".into(),
            "sleep".into(),
            crate::tui::BackgroundTaskRowStatus::Completed,
        );
        assert_eq!(app.herdr_state(), (Idle, None));
        app.background_task_rows = vec![row(true, crate::tui::BackgroundTaskRowStatus::Running)];

        // Blocked wins over working.
        set_remote_todos(
            &mut app,
            serde_json::json!([todo(
                "login",
                "pending",
                serde_json::json!({"requires_user_auth": {"reason": reason}})
            )]),
        );
        assert_eq!(app.herdr_state().0, Blocked);
    }

    #[test]
    fn received_todo_snapshot_updates_herdr_state_inside_throttle_window() {
        use crate::tui::herdr::AgentState::{Blocked, Idle};
        let mut app = super::super::tests::create_test_app();
        let session = app.active_client_session_id().unwrap().to_string();
        let snapshot = |todos: serde_json::Value| {
            serde_json::json!({"todos": todos, "plan": {}, "goals": []}).to_string()
        };
        assert!(app.receive_remote_todos(session.clone(), snapshot(serde_json::json!([]))));
        assert_eq!(app.herdr_state(), (Idle, None));
        let login = todo(
            "login",
            "pending",
            serde_json::json!({"requires_user_auth": {"reason": "Log in"}}),
        );
        assert!(app.receive_remote_todos(session, snapshot(serde_json::json!([login]))));
        assert_eq!(app.herdr_state(), (Blocked, Some("Log in".to_string())));
    }
}
