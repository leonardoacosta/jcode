# crates/jcode-tui/src/tui/app/catchup.rs

- CATCHUP_PAGE_ID · constant · L6-L6 — pub(super) const CATCHUP_PAGE_ID: &str = "catchup";
- CATCHUP_PAGE_TITLE · constant · L7-L7 — const CATCHUP_PAGE_TITLE: &str = "Catch Up";
- queue_catchup_resume · function · L10-L23 — pub(super) fn queue_catchup_resume(
- take_pending_catchup_resume · function · L25-L27 — pub(super) fn take_pending_catchup_resume(&mut self) -> Option<PendingCatchupResume>
- begin_in_flight_catchup_resume · function · L29-L37 — pub(super) fn begin_in_flight_catchup_resume(&mut self, request: PendingCatchupResume)
- clear_in_flight_catchup_resume · function · L39-L41 — pub(super) fn clear_in_flight_catchup_resume(&mut self)
- maybe_show_catchup_after_history · function · L43-L85 — pub(super) fn maybe_show_catchup_after_history(&mut self, session_id: &str)
- snapshot_without_catchup · function · L87-L94 — pub(super) fn snapshot_without_catchup(&self) -> SidePanelSnapshot
- pop_catchup_return_target · function · L96-L98 — pub(super) fn pop_catchup_return_target(&mut self) -> Option<String>
- catchup_page · function · L100-L115 — fn catchup_page(&self, session_id: &str, markdown: String) -> SidePanelPage
