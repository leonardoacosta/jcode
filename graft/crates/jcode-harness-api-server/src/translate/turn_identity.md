# crates/jcode-harness-api-server/src/translate/turn_identity.rs

- TurnFence · struct · L6-L13 — pub(super) struct TurnFence
- scoped · function · L15-L17 — pub(super) fn scoped(&self) -> bool
- active · function · L18-L20 — pub(super) fn active(&self) -> bool
- state · function · L21-L59 — pub(super) fn state(&mut self, event: &Value) -> bool
- start · function · L60-L83 — pub(super) fn start(&mut self, identity: &Value) -> bool
- accepts · function · L84-L88 — pub(super) fn accepts(&self, identity: &Value) -> bool
- is_latest · function · L89-L92 — pub(super) fn is_latest(&self, identity: &Value) -> bool
- complete · function · L93-L97 — pub(super) fn complete(&mut self)
