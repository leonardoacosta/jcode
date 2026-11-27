# crates/jcode-base/src/mobile_sync/wire_payload.rs

- thumbnail_identity · module · L4-L4 — mod thumbnail_identity;
- thumbnail_half · module · L7-L7 — mod thumbnail_half;
- pipeline_tests · module · L11-L11 — mod pipeline_tests;
- tests · module · L15-L15 — mod tests;
- wire_messages · function · L20-L115 — pub fn wire_messages(session: &Session, start: usize) -> Vec<WireMessage>
- message_chunks · function · L117-L136 — pub(super) fn message_chunks(messages: &[WireMessage]) -> Vec<&[WireMessage]>
- generated_images · function · L140-L194 — pub(super) fn generated_images(text: &str) -> Vec<WireImage>
- attached_images · function · L198-L219 — pub(super) fn attached_images(blocks: &[ContentBlock]) -> Vec<WireImage>
- image_thumbnail · function · L221-L258 — pub(super) fn image_thumbnail(reader: impl std::io::BufRead + std::io::Seek) -> Option<WireImage>
