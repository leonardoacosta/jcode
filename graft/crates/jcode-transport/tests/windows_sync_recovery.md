# crates/jcode-transport/tests/windows_sync_recovery.rs

- NEXT_PIPE · constant · L14-L14 — static NEXT_PIPE: AtomicU64 = AtomicU64::new(0);
- Peer · struct · L16-L20 — struct Peer
- start · function · L23-L57 — fn start<F, Fut>(script: F) -> Self
- finish · function · L59-L64 — fn finish(self)
- bounded_client · function · L68-L78 — fn bounded_client(script: impl FnOnce() + Send + 'static)
- sync_stream_timeouts_cancel_only_the_read_and_allow_recovery · function · L81-L121 — fn sync_stream_timeouts_cancel_only_the_read_and_allow_recovery()
- sync_stream_shutdown_interrupts_pending_read_and_affects_all_clones · function · L124-L157 — fn sync_stream_shutdown_interrupts_pending_read_and_affects_all_clones()
- sync_stream_partial_reads_then_peer_close_return_eof · function · L160-L183 — fn sync_stream_partial_reads_then_peer_close_return_eof()
