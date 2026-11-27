# crates/jcode-tui/src/tui/cloud_latency.rs

- tests · module · L10-L10 — mod tests;
- MAX_PENDING · constant · L12-L12 — const MAX_PENDING: usize = 64;
- RequestTimings · struct · L20-L22 — pub(super) struct RequestTimings
- PendingRequest · struct · L23-L29 — struct PendingRequest
- Span · struct · L31-L37 — pub(super) struct Span
- new · function · L39-L47 — pub(super) fn new(stage: &'static str) -> Self
- operation · function · L48-L50 — pub(super) fn operation(&mut self, operation: &'static str)
- finish · function · L51-L53 — pub(super) fn finish(&mut self, outcome: &'static str)
- drop · function · L56-L65 — fn drop(&mut self)
- request_key · function · L68-L77 — fn request_key(request: &Request) -> Option<(u64, &'static str)>
- begin_request · function · L79-L86 — pub(super) fn begin_request(request: &Request) -> Option<Span>
- request_sent · function · L89-L124 — pub(super) fn request_sent(&self, request: &Request)
- response_wait · function · L126-L178 — fn response_wait(&self, event: &ServerEvent) -> Option<(&'static str, u64, f64)>
- parse_event · function · L180-L206 — pub(super) fn parse_event(&self, text: &str) -> serde_json::Result<ServerEvent>
- event_key · function · L209-L216 — pub(super) fn event_key(event: &ServerEvent) -> Option<(&'static str, Option<u64>)>
- applied · function · L218-L230 — pub(super) fn applied(kind: &'static str, started: Instant, redraw: bool)
- rendered · function · L232-L241 — pub(super) fn rendered()
- spawn_usage_hydration · function · L243-L256 — pub(super) fn spawn_usage_hydration(
