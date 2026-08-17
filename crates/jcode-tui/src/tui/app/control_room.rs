use super::App;
use crate::tui::control_room::{ControlRoomAction, ControlRoomOverlay};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use jcode_session_types::{
    ContextConfidence, ContextProvenance, ContextRow, ContextSection, ContextSnapshot,
};
use std::cell::RefCell;
use std::path::Path;

impl App {
    pub(in crate::tui::app) fn toggle_control_room(&mut self) {
        if self.control_room_overlay.is_some() {
            self.control_room_overlay = None;
            self.set_status_notice("Context Control Room closed");
        } else {
            self.open_control_room();
        }
    }

    pub(in crate::tui::app) fn open_control_room(&mut self) {
        let snapshot = self.build_control_room_snapshot();
        self.control_room_overlay = Some(RefCell::new(ControlRoomOverlay::new(snapshot)));
        self.set_status_notice("Context Control Room opened · Alt+O toggles, Esc closes");
    }

    pub(in crate::tui::app) fn handle_control_room_key(
        &mut self,
        code: KeyCode,
        _modifiers: KeyModifiers,
    ) -> Result<()> {
        let Some(overlay_cell) = self.control_room_overlay.as_ref() else {
            return Ok(());
        };
        let action = overlay_cell.borrow_mut().handle_key(code);
        match action {
            ControlRoomAction::Close => {
                self.control_room_overlay = None;
                self.set_status_notice("Context Control Room closed");
            }
            ControlRoomAction::Copy => {
                self.copy_selected_control_room_value_with(super::helpers::copy_to_clipboard);
            }
            ControlRoomAction::Focus => {
                self.focus_selected_control_room_surface();
            }
            ControlRoomAction::None => {}
        }
        Ok(())
    }

    pub(in crate::tui::app) fn build_control_room_snapshot(&self) -> ContextSnapshot {
        let org_row = ContextRow::unavailable("Organization", "no organization selected");
        let project_row = self.project_context_row();
        let workspace_row = self.workspace_context_row();
        let initiative_row = ContextRow::unavailable("Initiative", "no initiative selected");
        let task_row = ContextRow::unavailable("Task/run", "no task/run selected");

        let mut session_row = ContextRow::current(
            "Jcode session",
            self.session
                .custom_title
                .clone()
                .or_else(|| self.session.title.clone())
                .or_else(|| self.session.short_name.clone())
                .unwrap_or_else(|| self.session.id.clone()),
        );
        session_row.stable_id = Some(self.session.id.clone());
        session_row.focusable = true;

        let provider_model = self.provider.model();
        let provider_label = self
            .session
            .provider_key
            .as_deref()
            .unwrap_or_else(|| self.provider.name())
            .to_string();
        let model_label = self.session.model.clone().unwrap_or(provider_model);
        let mut provider_row = ContextRow::current(
            "Provider/model",
            format!("{} / {}", provider_label, model_label),
        );
        provider_row.copyable = false;

        let herdr_row = herdr_context_row();

        ContextSnapshot {
            semantic: ContextSection {
                title: "Semantic context".to_string(),
                rows: vec![
                    org_row,
                    project_row,
                    workspace_row,
                    initiative_row,
                    task_row,
                ],
            },
            execution: ContextSection {
                title: "Execution substrate".to_string(),
                rows: vec![session_row, provider_row, herdr_row],
            },
        }
    }

    fn project_context_row(&self) -> ContextRow {
        let Some(working_dir) = self.session.working_dir.as_deref() else {
            return ContextRow::unavailable("Project", "no project context");
        };
        let project_name = Path::new(working_dir)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(working_dir);
        ContextRow {
            label: "Project".to_string(),
            value: Some(project_name.to_string()),
            stable_id: Some(working_dir.to_string()),
            provenance: ContextProvenance::InferredFromPath,
            confidence: ContextConfidence::Inferred,
            copyable: true,
            focusable: false,
            unavailable_reason: None,
        }
    }

    fn workspace_context_row(&self) -> ContextRow {
        if let Some(path) = self.session.working_dir.clone() {
            ContextRow::current("Workspace", path)
        } else {
            ContextRow::unavailable("Workspace", "no working directory")
        }
    }

    pub(in crate::tui::app) fn copy_selected_control_room_value_with(
        &mut self,
        copy: impl FnOnce(&str) -> bool,
    ) -> bool {
        let Some(overlay_cell) = self.control_room_overlay.as_ref() else {
            return false;
        };
        let value = {
            let overlay = overlay_cell.borrow();
            overlay.selected_copy_value().map(str::to_string)
        };
        let Some(value) = value else {
            if let Some(overlay_cell) = self.control_room_overlay.as_ref() {
                overlay_cell
                    .borrow_mut()
                    .set_feedback("Selected context row is not copyable");
            }
            self.set_status_notice("Selected context row is not copyable");
            return false;
        };

        let copied = copy(&value);
        let feedback = if copied {
            "Copied selected context value"
        } else {
            "Clipboard unavailable for selected context value"
        };
        if let Some(overlay_cell) = self.control_room_overlay.as_ref() {
            overlay_cell.borrow_mut().set_feedback(feedback);
        }
        self.set_status_notice(feedback);
        copied
    }

    fn focus_selected_control_room_surface(&mut self) {
        let Some(overlay_cell) = self.control_room_overlay.as_ref() else {
            return;
        };
        let label = {
            let overlay = overlay_cell.borrow();
            overlay.selected_focus_label().map(str::to_string)
        };
        let Some(label) = label else {
            if let Some(overlay_cell) = self.control_room_overlay.as_ref() {
                overlay_cell
                    .borrow_mut()
                    .set_feedback("Selected context row has no existing surface to focus");
            }
            self.set_status_notice("Selected context row has no existing surface to focus");
            return;
        };
        let feedback = format!("Focused existing surface: {label}");
        if let Some(overlay_cell) = self.control_room_overlay.as_ref() {
            overlay_cell.borrow_mut().set_feedback(feedback.clone());
        }
        self.set_status_notice(feedback);
    }
}

pub(in crate::tui::app) fn is_control_room_toggle(code: KeyCode, modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::ALT) && matches!(code, KeyCode::Char('o') | KeyCode::Char('O'))
}

fn herdr_context_row() -> ContextRow {
    let pane = std::env::var("HERDR_PANE_ID")
        .or_else(|_| std::env::var("HERDR_PANE"))
        .ok()
        .filter(|value| !value.trim().is_empty());
    let workspace = std::env::var("HERDR_WORKSPACE").ok();
    let state = std::env::var("HERDR_HARNESS_STATE")
        .or_else(|_| std::env::var("HERDR_STATE"))
        .ok();

    if let Some(pane) = pane {
        let mut value = pane.clone();
        if let Some(workspace) = workspace.filter(|value| !value.trim().is_empty()) {
            value.push_str(" · ");
            value.push_str(&workspace);
        }
        if let Some(state) = state.filter(|value| !value.trim().is_empty()) {
            value.push_str(" · ");
            value.push_str(&state);
        }
        ContextRow {
            label: "Herdr pane".to_string(),
            value: Some(value),
            stable_id: Some(pane),
            provenance: ContextProvenance::Herdr,
            confidence: ContextConfidence::Inferred,
            copyable: true,
            focusable: true,
            unavailable_reason: None,
        }
    } else {
        ContextRow::unavailable("Herdr pane", "Herdr not detected")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_o_matcher_accepts_only_alt_o() {
        assert!(is_control_room_toggle(
            KeyCode::Char('o'),
            KeyModifiers::ALT
        ));
        assert!(is_control_room_toggle(
            KeyCode::Char('O'),
            KeyModifiers::ALT
        ));
        assert!(!is_control_room_toggle(
            KeyCode::Char('o'),
            KeyModifiers::NONE
        ));
        assert!(!is_control_room_toggle(
            KeyCode::Char('p'),
            KeyModifiers::ALT
        ));
    }
}
