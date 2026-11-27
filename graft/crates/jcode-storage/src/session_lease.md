# crates/jcode-storage/src/session_lease.rs

- SessionLease · struct · L23-L42 — pub struct SessionLease
- is_here · function · L45-L47 — pub fn is_here(&self) -> bool
- SessionLeaseBlock · enum · L52-L55 — pub enum SessionLeaseBlock
- fmt · function · L58-L72 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- session_leases_dir · function · L75-L77 — pub fn session_leases_dir() -> Option<PathBuf>
- lease_path · function · L79-L89 — fn lease_path(session_id: &str) -> Option<PathBuf>
- read_session_lease · function · L91-L95 — pub fn read_session_lease(session_id: &str) -> Option<SessionLease>
- write_session_lease · function · L97-L107 — pub fn write_session_lease(lease: &SessionLease) -> anyhow::Result<()>
- remove_session_lease · function · L109-L113 — pub fn remove_session_lease(session_id: &str)
- session_lease_block · function · L117-L129 — pub fn session_lease_block(session_id: &str, copy_epoch: u64) -> Option<SessionLeaseBlock>
- tests · module · L132-L200 — mod tests
- with_home · function · L135-L142 — fn with_home<T>(f: impl FnOnce() -> T) -> T
- lease · function · L144-L154 — fn lease(epoch: u64, away: Option<&str>) -> SessionLease
- no_lease_never_blocks · function · L157-L159 — fn no_lease_never_blocks()
- moved_away_blocks_every_copy · function · L162-L170 — fn moved_away_blocks_every_copy()
- returned_session_blocks_only_stale_copies · function · L173-L185 — fn returned_session_blocks_only_stale_copies()
- rejects_path_like_ids · function · L188-L199 — fn rejects_path_like_ids()
