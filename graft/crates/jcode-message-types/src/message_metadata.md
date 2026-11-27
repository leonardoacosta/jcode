# crates/jcode-message-types/src/message_metadata.rs

- Metadata · struct · L6-L13 — pub(super) struct Metadata<'a>
- new · function · L16-L23 — pub(super) fn new(message: &'a Message) -> Self
- json_bytes · function · L26-L40 — pub(super) fn json_bytes(value: &impl serde::Serialize) -> serde_json::Result<usize>
- Counter · struct · L28-L28 — struct Counter(usize);
- write · function · L30-L33 — fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize>
- flush · function · L34-L36 — fn flush(&mut self) -> std::io::Result<()>
- tests · module · L44-L44 — mod tests;
