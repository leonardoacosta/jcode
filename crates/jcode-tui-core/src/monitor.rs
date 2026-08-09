//! Small, renderer-agnostic monitor state for live indicators.
//!
//! A monitor owns the indicators a surface wants to expose, tracks the focused
//! indicator, and gives the UI one key-bound action for expanding its detail.
//! Rendering stays in the caller so this primitive works for inline widgets,
//! side panels, and future remote surfaces alike.

use crossterm::event::{KeyCode, KeyModifiers};

use crate::keybind::KeyBinding;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorIndicator {
    pub id: String,
    pub label: String,
    pub summary: String,
    pub detail: String,
    pub expanded: bool,
}

impl MonitorIndicator {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        summary: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            summary: summary.into(),
            detail: detail.into(),
            expanded: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Monitor {
    indicators: Vec<MonitorIndicator>,
    selected: usize,
    expand_binding: KeyBinding,
}

impl Monitor {
    pub fn new(expand_binding: KeyBinding) -> Self {
        Self {
            indicators: Vec::new(),
            selected: 0,
            expand_binding,
        }
    }

    pub fn indicators(&self) -> &[MonitorIndicator] {
        &self.indicators
    }
    pub fn selected(&self) -> Option<&MonitorIndicator> {
        self.indicators.get(self.selected)
    }
    pub fn selected_index(&self) -> Option<usize> {
        (!self.indicators.is_empty()).then_some(self.selected)
    }

    /// Replace the live snapshot while retaining selection and expansion state by id.
    pub fn watch(&mut self, next: Vec<MonitorIndicator>) {
        let selected_id = self.selected().map(|indicator| indicator.id.clone());
        let expanded_ids = self
            .indicators
            .iter()
            .filter(|indicator| indicator.expanded)
            .map(|indicator| indicator.id.clone())
            .collect::<std::collections::HashSet<_>>();

        self.indicators = next;
        for indicator in &mut self.indicators {
            indicator.expanded |= expanded_ids.contains(&indicator.id);
        }
        self.selected = selected_id
            .and_then(|id| {
                self.indicators
                    .iter()
                    .position(|indicator| indicator.id == id)
            })
            .unwrap_or_else(|| self.selected.min(self.indicators.len().saturating_sub(1)));
    }

    pub fn select_next(&mut self) {
        if !self.indicators.is_empty() {
            self.selected = (self.selected + 1) % self.indicators.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.indicators.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.indicators.len() - 1);
        }
    }

    /// Handle the configured expansion key. Returns true when the key was consumed.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if !self.expand_binding.matches(code, modifiers) {
            return false;
        }
        if let Some(indicator) = self.indicators.get_mut(self.selected) {
            indicator.expanded = !indicator.expanded;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor() -> Monitor {
        Monitor::new(KeyBinding {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
        })
    }

    fn indicator(id: &str) -> MonitorIndicator {
        MonitorIndicator::new(id, id, format!("{id} summary"), format!("{id} detail"))
    }

    #[test]
    fn expansion_key_toggles_selected_indicator_only() {
        let mut monitor = monitor();
        monitor.watch(vec![indicator("one"), indicator("two")]);

        assert!(monitor.handle_key(KeyCode::Char('e'), KeyModifiers::NONE));
        assert!(monitor.indicators()[0].expanded);
        assert!(!monitor.indicators()[1].expanded);
        assert!(!monitor.handle_key(KeyCode::Char('x'), KeyModifiers::NONE));
    }

    #[test]
    fn watch_preserves_selection_and_expansion_by_id() {
        let mut monitor = monitor();
        monitor.watch(vec![indicator("one"), indicator("two")]);
        monitor.select_next();
        monitor.handle_key(KeyCode::Char('e'), KeyModifiers::NONE);

        monitor.watch(vec![indicator("new"), indicator("two"), indicator("one")]);

        assert_eq!(monitor.selected().map(|item| item.id.as_str()), Some("two"));
        assert!(monitor.selected().is_some_and(|item| item.expanded));
    }

    #[test]
    fn selection_wraps_in_both_directions() {
        let mut monitor = monitor();
        monitor.watch(vec![indicator("one"), indicator("two")]);
        monitor.select_previous();
        assert_eq!(monitor.selected().map(|item| item.id.as_str()), Some("two"));
        monitor.select_next();
        assert_eq!(monitor.selected().map(|item| item.id.as_str()), Some("one"));
    }
}
