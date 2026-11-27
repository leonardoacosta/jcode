# crates/jcode-app-core/src/server/request_watchdog.rs

- RequestHandlerWatchdog · struct · L3-L5 — pub(super) struct RequestHandlerWatchdog
- RequestHandlerWatchdogContext · struct · L7-L18 — pub(super) struct RequestHandlerWatchdogContext
- spawn · function · L21-L69 — pub(super) fn spawn(ctx: RequestHandlerWatchdogContext) -> Self
- drop · function · L73-L75 — fn drop(&mut self)
- log_request_lifecycle_handled · function · L78-L93 — pub(super) fn log_request_lifecycle_handled(
- should_start_idle_soft_interrupt · function · L95-L101 — pub(super) fn should_start_idle_soft_interrupt(
- reject_if_agent_busy_for_request · function · L103-L123 — pub(super) fn reject_if_agent_busy_for_request(
- send_agent_busy_error · function · L125-L151 — pub(super) fn send_agent_busy_error(
