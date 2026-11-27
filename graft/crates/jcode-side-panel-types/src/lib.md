# crates/jcode-side-panel-types/src/lib.rs

- SidePanelPageFormat · enum · L5-L9 — pub enum SidePanelPageFormat
- as_str · function · L12-L17 — pub fn as_str(&self) -> &'static str
- SidePanelPageSource · enum · L22-L27 — pub enum SidePanelPageSource
- as_str · function · L30-L36 — pub fn as_str(&self) -> &'static str
- PersistedSidePanelState · struct · L40-L47 — pub struct PersistedSidePanelState
- PersistedSidePanelPage · struct · L50-L59 — pub struct PersistedSidePanelPage
- SidePanelPage · struct · L62-L77 — pub struct SidePanelPage
- SidePanelSnapshot · struct · L80-L87 — pub struct SidePanelSnapshot
- has_pages · function · L90-L92 — pub fn has_pages(&self) -> bool
- focused_page · function · L94-L97 — pub fn focused_page(&self) -> Option<&SidePanelPage>
- snapshot_is_empty · function · L100-L102 — pub fn snapshot_is_empty(snapshot: &SidePanelSnapshot) -> bool
