# crates/jcode-tui/src/tui/app/remote/turn_identity.rs

- TurnFence · struct · L6-L13 — pub(in crate::tui::app) struct TurnFence
- diagnostic_identity · function · L16-L18 — pub(in crate::tui::app) fn diagnostic_identity(&self) -> Option<&TurnIdentity>
- scoped · function · L20-L22 — pub(super) fn scoped(&self) -> bool
- bootstrap · function · L24-L64 — pub(super) fn bootstrap(
- start · function · L66-L83 — pub(super) fn start(&mut self, identity: &TurnIdentity) -> bool
- accepts · function · L85-L87 — pub(super) fn accepts(&self, identity: &TurnIdentity) -> bool
- is_latest · function · L89-L92 — pub(super) fn is_latest(&self, identity: &TurnIdentity) -> bool
- complete · function · L94-L99 — pub(super) fn complete(&mut self, identity: &TurnIdentity)
