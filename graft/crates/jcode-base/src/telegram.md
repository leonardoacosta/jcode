# crates/jcode-base/src/telegram.rs

- API_BASE · constant · L4-L4 — const API_BASE: &str = "https://api.telegram.org/bot";
- TelegramResponse · struct · L7-L12 — struct TelegramResponse<T>
- Update · struct · L15-L18 — pub struct Update
- TelegramMessage · struct · L21-L26 — pub struct TelegramMessage
- Chat · struct · L29-L31 — pub struct Chat
- MessageRequest · struct · L35-L40 — struct MessageRequest<'a>
- UpdatesRequest · struct · L43-L48 — struct UpdatesRequest
- message_request · function · L50-L62 — fn message_request(
- updates_request · function · L64-L78 — fn updates_request(
- send_message · function · L80-L102 — pub async fn send_message(
- get_updates · function · L104-L125 — pub async fn get_updates(
- tests · module · L128-L208 — mod tests
- borrowed_message_request_preserves_legacy_bytes_after_inputs_drop · function · L132-L157 — fn borrowed_message_request_preserves_legacy_bytes_after_inputs_drop()
- borrowed_updates_request_preserves_optional_offset_and_timeout · function · L160-L184 — fn borrowed_updates_request_preserves_optional_offset_and_timeout()
- test_parse_update · function · L187-L199 — fn test_parse_update()
- test_parse_response · function · L202-L207 — fn test_parse_response()
