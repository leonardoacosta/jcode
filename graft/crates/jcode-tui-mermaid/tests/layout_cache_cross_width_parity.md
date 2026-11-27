# crates/jcode-tui-mermaid/tests/layout_cache_cross_width_parity.rs

- NARROW_CELLS · constant · L70-L70 — const NARROW_CELLS: u16 = 90;
- WIDE_CELLS · constant · L73-L73 — const WIDE_CELLS: u16 = 200;
- FLOWCHART_TEMPLATE · constant · L76-L76 — const FLOWCHART_TEMPLATE: &str = "flowchart LR\n    A{cell}[Ingest] --> B{cell}{Valid?}\n    B{cell} -->|yes| C{cell}[Store]\n    B{cell} -->|no| D{cell}[Reject]\n    C{cell} --> E{cell}[Index]\n    E{cell} --> F{cell}[Serve]";
- SEQUENCE_TEMPLATE · constant · L80-L80 — const SEQUENCE_TEMPLATE: &str = "sequenceDiagram\n    participant U as User{cell}\n    participant S as Server{cell}\n    participant D as Db{cell}\n    U->>S: request {cell}\n    S->>D: query\n    D-->>S: rows\n    S-->>U: response";
- Rendered · struct · L82-L86 — struct Rendered
- render_png · function · L89-L100 — fn render_png(label: &str, content: &str, width_cells: u16) -> Rendered
- evict_pngs_for_hash · function · L106-L123 — fn evict_pngs_for_hash(cache_dir: &Path, hash: u64) -> usize
- decode_rgba · function · L125-L129 — fn decode_rgba(label: &str, bytes: &[u8]) -> image::RgbaImage
- pixel_diff_details · function · L134-L166 — fn pixel_diff_details(a: &image::RgbaImage, b: &image::RgbaImage) -> Option<String>
- CellOutcome · struct · L168-L172 — struct CellOutcome
- run_cell · function · L178-L301 — fn run_cell(label: &str, content: &str, expected_path_marker: Option<&str>) -> CellOutcome
- layout_cache_cross_width_parity · function · L305-L368 — fn layout_cache_cross_width_parity()
