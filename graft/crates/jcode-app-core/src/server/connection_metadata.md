# crates/jcode-app-core/src/server/connection_metadata.rs

- ConnectionMetadataUpdate · enum · L7-L12 — pub(super) enum ConnectionMetadataUpdate<'a>
- release_large_event_buffer · function · L14-L20 — pub(super) fn release_large_event_buffer(buffer: &mut Vec<u8>)
- from_event · function · L23-L39 — pub(super) fn from_event(event: &'a ServerEvent) -> Option<Self>
- apply · function · L41-L57 — pub(super) fn apply(self, is_processing: &mut bool, current_tool_name: &mut Option<String>)
