# crates/jcode-harness-api-server/src/session_presence_tests.rs

- SocketDir · struct · L6-L6 — struct SocketDir(std::path::PathBuf);
- new · function · L8-L17 — fn new() -> Self
- NEXT · constant · L9-L9 — static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
- path · function · L18-L20 — fn path(&self) -> &std::path::Path
- drop · function · L23-L25 — fn drop(&mut self)
- snapshot · function · L28-L46 — async fn snapshot(reply: Value) -> std::collections::HashSet<String>
- presence_probe_is_read_only_correlated_and_deduplicated · function · L49-L67 — async fn presence_probe_is_read_only_correlated_and_deduplicated()
- old_malformed_and_disconnected_sources_never_claim_connected_users · function · L70-L83 — async fn old_malformed_and_disconnected_sources_never_claim_connected_users()
- stalled_presence_probe_is_bounded · function · L86-L94 — async fn stalled_presence_probe_is_bounded()
