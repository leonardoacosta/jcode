# crates/jcode-tui/src/tui/app/todos_remote_cache.rs

- TodoState · type · L5-L9 — type TodoState = (
- RemoteTodoCache · struct · L11-L14 — pub(crate) struct RemoteTodoCache
- load · function · L16-L57 — pub(super) fn load(cache: &RefCell<Option<RemoteTodoCache>>, raw: Option<&str>) -> Arc<TodoState>
- tests · module · L60-L122 — mod tests
- payload · function · L64-L66 — fn payload(intent: &str) -> String
- unchanged_snapshots_share_decoding_and_equal_length_edits_invalidate · function · L69-L83 — fn unchanged_snapshots_share_decoding_and_equal_length_edits_invalidate()
- malformed_fields_still_default_independently_and_cache_is_bounded · function · L86-L98 — fn malformed_fields_still_default_independently_and_cache_is_bounded()
- benchmark_unchanged_remote_todo_decoding · function · L102-L121 — fn benchmark_unchanged_remote_todo_decoding()
