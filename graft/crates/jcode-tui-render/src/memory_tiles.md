# crates/jcode-tui-render/src/memory_tiles.rs

- MemoryTilePlan · struct · L5-L10 — pub struct MemoryTilePlan
- MemoryTile · struct · L12-L15 — pub struct MemoryTile
- MemoryTileItem · struct · L18-L21 — pub struct MemoryTileItem
- from · function · L24-L29 — fn from(content: String) -> Self
- from · function · L33-L35 — fn from(content: &str) -> Self
- parse_memory_display_entries · function · L38-L95 — pub fn parse_memory_display_entries(content: &str) -> Vec<(String, MemoryTileItem)>
- group_into_tiles · function · L97-L119 — pub fn group_into_tiles<T>(entries: Vec<(String, T)>) -> Vec<MemoryTile>
- split_by_display_width · function · L123-L145 — pub fn split_by_display_width(s: &str, max_width: usize) -> Vec<String>
- truncate_to_display_width · function · L147-L178 — fn truncate_to_display_width(s: &str, max_width: usize) -> String
- format_memory_updated_age · function · L180-L197 — fn format_memory_updated_age(updated_at: DateTime<Utc>) -> String
- memory_age_text_tint · function · L199-L215 — fn memory_age_text_tint(updated_at: Option<DateTime<Utc>>) -> Color
- memory_tile_content_lines · function · L217-L315 — fn memory_tile_content_lines(
- render_memory_tile_box · function · L317-L352 — fn render_memory_tile_box(
- plan_memory_tile · function · L354-L378 — pub fn plan_memory_tile(
- choose_memory_tile_span · function · L380-L411 — pub fn choose_memory_tile_span(
- render_memory_tiles · function · L413-L587 — pub fn render_memory_tiles(
- Placement · struct · L437-L441 — struct Placement
- PlannedTile · struct · L444-L447 — struct PlannedTile
