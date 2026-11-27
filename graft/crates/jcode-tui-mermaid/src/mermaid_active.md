# crates/jcode-tui-mermaid/src/mermaid_active.rs

- ACTIVE_DIAGRAMS · constant · L7-L8 — static ACTIVE_DIAGRAMS: LazyLock<Mutex<Vec<ActiveDiagram>>> =
- STREAMING_PREVIEW_DIAGRAM · constant · L12-L13 — static STREAMING_PREVIEW_DIAGRAM: LazyLock<Mutex<Option<ActiveDiagram>>> =
- ActiveDiagram · struct · L17-L22 — struct ActiveDiagram
- to_diagram_info · function · L24-L31 — fn to_diagram_info(diagram: ActiveDiagram) -> DiagramInfo
- to_active_diagram · function · L33-L40 — fn to_active_diagram(diagram: DiagramInfo) -> ActiveDiagram
- register_active_diagram · function · L42-L64 — pub fn register_active_diagram(hash: u64, width: u32, height: u32, label: Option<String>)
- set_streaming_preview_diagram · function · L67-L76 — pub fn set_streaming_preview_diagram(hash: u64, width: u32, height: u32, label: Option<String>)
- clear_streaming_preview_diagram · function · L79-L83 — pub fn clear_streaming_preview_diagram()
- get_active_diagrams · function · L86-L110 — pub fn get_active_diagrams() -> Vec<DiagramInfo>
- snapshot_active_diagrams · function · L113-L119 — pub fn snapshot_active_diagrams() -> Vec<DiagramInfo>
- restore_active_diagrams · function · L122-L130 — pub fn restore_active_diagrams(snapshot: Vec<DiagramInfo>)
- active_diagram_count · function · L132-L138 — pub fn active_diagram_count() -> usize
- clear_active_diagrams · function · L141-L146 — pub fn clear_active_diagrams()
