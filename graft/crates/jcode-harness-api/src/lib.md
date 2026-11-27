# crates/jcode-harness-api/src/lib.rs

- client · module · L18-L18 — mod client;
- edit_stats · module · L19-L19 — mod edit_stats;
- events · module · L20-L20 — mod events;
- requests · module · L25-L25 — mod requests;
- runtime_commands · module · L26-L26 — mod runtime_commands;
- sockets · module · L28-L28 — mod sockets;
- swarm_metadata · module · L29-L29 — mod swarm_metadata;
- schema_snapshot_tests · module · L47-L47 — mod schema_snapshot_tests;
- capability_coverage_tests · module · L51-L51 — mod capability_coverage_tests;
- API_VERSION_MAJOR · constant · L54-L54 — pub const API_VERSION_MAJOR: u32 = 1;
- API_VERSION_MINOR · constant · L56-L56 — pub const API_VERSION_MINOR: u32 = 11;
- ClientFrame · struct · L60-L67 — pub struct ClientFrame
- ServerFrame · struct · L71-L80 — pub struct ServerFrame
- new · function · L83-L89 — pub fn new(id: u64, request: ApiRequest) -> Self
- event · function · L93-L99 — pub fn event(event: ApiEvent) -> Self
- reply · function · L101-L107 — pub fn reply(reply_to: u64, event: ApiEvent) -> Self
