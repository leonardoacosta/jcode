# crates/jcode-tui/src/tui/app/copy_badge_state.rs

- pulse_active · function · L4-L6 — fn pulse_active(expires_at: Option<Instant>, now: Instant) -> bool
- alt_is_active · function · L8-L10 — pub(crate) fn alt_is_active(&self, now: Instant) -> bool
- shift_is_active · function · L12-L15 — pub(crate) fn shift_is_active(&self, now: Instant) -> bool
- key_is_active · function · L17-L26 — pub(crate) fn key_is_active(&self, key: char, now: Instant) -> bool
- feedback_for_key · function · L28-L36 — pub(crate) fn feedback_for_key(&self, key: char, now: Instant) -> Option<bool>
- expand_feedback_is_active · function · L38-L41 — pub(crate) fn expand_feedback_is_active(&self, now: Instant) -> bool
