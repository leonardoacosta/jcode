# crates/jcode-base/src/soft_interrupt_store.rs

- PersistedSoftInterrupt · struct · L7-L15 — struct PersistedSoftInterrupt
- PersistedSoftInterruptSource · enum · L19-L24 — enum PersistedSoftInterruptSource
- from · function · L27-L34 — fn from(value: SoftInterruptSource) -> Self
- from · function · L38-L45 — fn from(value: PersistedSoftInterruptSource) -> Self
- from · function · L49-L57 — fn from(value: SoftInterruptMessage) -> Self
- from · function · L61-L69 — fn from(value: PersistedSoftInterrupt) -> Self
- dir_path · function · L72-L74 — fn dir_path() -> Result<PathBuf>
- path_for_session · function · L76-L78 — fn path_for_session(session_id: &str) -> Result<PathBuf>
- load · function · L80-L91 — pub fn load(session_id: &str) -> Result<Vec<SoftInterruptMessage>>
- take · function · L93-L100 — pub fn take(session_id: &str) -> Result<Vec<SoftInterruptMessage>>
- overwrite · function · L102-L118 — pub fn overwrite(session_id: &str, interrupts: &[SoftInterruptMessage]) -> Result<()>
- append · function · L120-L124 — pub fn append(session_id: &str, interrupt: SoftInterruptMessage) -> Result<()>
- clear · function · L126-L128 — pub fn clear(session_id: &str) -> Result<()>
- remove_by_client_ids · function · L132-L146 — pub fn remove_by_client_ids(session_id: &str, client_ids: &[String]) -> Result<Vec<String>>
- soft_interrupt_store_tests · module · L150-L150 — mod soft_interrupt_store_tests;
