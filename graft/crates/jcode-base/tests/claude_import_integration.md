# crates/jcode-base/tests/claude_import_integration.rs

- ENV_LOCK · constant · L32-L32 — static ENV_LOCK: Mutex<()> = Mutex::new(());
- EnvVarGuard · struct · L35-L38 — struct EnvVarGuard
- set_path · function · L41-L45 — fn set_path(key: &'static str, value: &Path) -> Self
- drop · function · L49-L55 — fn drop(&mut self)
- make_project_dir · function · L59-L63 — fn make_project_dir(home: &Path) -> PathBuf
- all_text · function · L65-L78 — fn all_text(session: &Session) -> String
- import_array_tool_result_with_image_uses_placeholder_not_base64 · function · L85-L132 — fn import_array_tool_result_with_image_uses_placeholder_not_base64()
- jcode_session_file · function · L137-L140 — fn jcode_session_file(home: &Path, claude_id: &str) -> PathBuf
- import_tool_result_null_and_missing_content_does_not_drop_lines · function · L147-L173 — fn import_tool_result_null_and_missing_content_does_not_drop_lines()
- import_excludes_sidechain_messages · function · L180-L208 — fn import_excludes_sidechain_messages()
- import_skips_only_malformed_lines · function · L215-L243 — fn import_skips_only_malformed_lines()
- list_filters_empty_and_meta_only_transcripts · function · L250-L295 — fn list_filters_empty_and_meta_only_transcripts()
- index_with_missing_full_path_falls_back_to_sibling_jsonl · function · L303-L351 — fn index_with_missing_full_path_falls_back_to_sibling_jsonl()
- lazy_lister_handles_array_tool_result_session · function · L358-L379 — fn lazy_lister_handles_array_tool_result_session()
