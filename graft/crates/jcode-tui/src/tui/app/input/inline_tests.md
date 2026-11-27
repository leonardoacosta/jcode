# crates/jcode-tui/src/tui/app/input/inline_tests.rs

- ssh_clipboard_image_bytes_work_without_local_file_or_url_fetch · function · L2-L26 — fn ssh_clipboard_image_bytes_work_without_local_file_or_url_fetch()
- with_local_env · function · L37-L46 — fn with_local_env<T>(f: impl FnOnce() -> T) -> T
- dropped_paths_accept_quotes_shell_escapes_and_file_urls · function · L49-L65 — fn dropped_paths_accept_quotes_shell_escapes_and_file_urls()
- dropped_images_load_all_supported_files_and_reject_mixed_text · function · L68-L88 — fn dropped_images_load_all_supported_files_and_reject_mixed_text()
- dropped_bmp_is_converted_and_fake_image_is_rejected · function · L93-L110 — fn dropped_bmp_is_converted_and_fake_image_is_rejected()
- smart_paste_prefers_normal_text_when_clipboard_has_text · function · L113-L125 — fn smart_paste_prefers_normal_text_when_clipboard_has_text()
- smart_paste_uses_image_only_when_no_text_is_available · function · L128-L146 — fn smart_paste_uses_image_only_when_no_text_is_available()
- smart_paste_empty_clipboard_stays_empty_not_dictation · function · L149-L157 — fn smart_paste_empty_clipboard_stays_empty_not_dictation()
- smart_paste_uses_image_when_text_target_is_blank · function · L160-L180 — fn smart_paste_uses_image_when_text_target_is_blank()
- paste_shortcut_accepts_control_alt_command_and_meta_v · function · L183-L207 — fn paste_shortcut_accepts_control_alt_command_and_meta_v()
- wayland_text_type_prefers_utf8_plain_text · function · L210-L217 — fn wayland_text_type_prefers_utf8_plain_text()
- shifted_printable_fallback_uppercases_ascii_letters · function · L220-L223 — fn shifted_printable_fallback_uppercases_ascii_letters()
- shifted_printable_fallback_preserves_terminal_translated_symbols · function · L226-L231 — fn shifted_printable_fallback_preserves_terminal_translated_symbols()
- shifted_printable_fallback_does_not_synthesize_us_symbol_layout · function · L234-L238 — fn shifted_printable_fallback_does_not_synthesize_us_symbol_layout()
- text_input_for_shifted_symbols_preserves_layout_translated_char · function · L241-L249 — fn text_input_for_shifted_symbols_preserves_layout_translated_char()
- text_input_for_altgr_symbols_preserves_layout_translated_char · function · L252-L262 — fn text_input_for_altgr_symbols_preserves_layout_translated_char()
- text_input_for_control_shortcut_letters_stays_non_text · function · L265-L277 — fn text_input_for_control_shortcut_letters_stays_non_text()
