# src/cli/commands/menubar_tests.rs

- title_no_sessions_is_icon_only · function · L5-L11 — fn title_no_sessions_is_icon_only()
- title_idle_shows_zero_streaming_and_total · function · L14-L20 — fn title_idle_shows_zero_streaming_and_total()
- title_streaming_shows_compact_ratio · function · L23-L29 — fn title_streaming_shows_compact_ratio()
- summary_pluralizes_sessions · function · L32-L47 — fn summary_pluralizes_sessions()
- session_menu_item_title_uses_meaningful_title_instead_of_animal_name · function · L50-L60 — fn session_menu_item_title_uses_meaningful_title_instead_of_animal_name()
- session_menu_item_title_is_icon_only_without_meaningful_title · function · L63-L73 — fn session_menu_item_title_is_icon_only_without_meaningful_title()
- session_menu_item_title_loads_persisted_todo_title · function · L76-L123 — fn session_menu_item_title_loads_persisted_todo_title()
- menu_presence_includes_only_user_root_sessions · function · L126-L175 — fn menu_presence_includes_only_user_root_sessions()
- counts_report_serializes_to_json · function · L178-L185 — fn counts_report_serializes_to_json()
- menubar_sandbox_detection · function · L189-L229 — fn menubar_sandbox_detection()
- menubar_sandbox_detects_consistent_home_override · function · L233-L257 — fn menubar_sandbox_detects_consistent_home_override()
- menubar_sandbox_accepts_symlinked_real_home · function · L261-L274 — fn menubar_sandbox_accepts_symlinked_real_home()
- menubar_sandbox_fails_closed_without_passwd_home · function · L278-L297 — fn menubar_sandbox_fails_closed_without_passwd_home()
- menubar_helper_pids_matches_only_exact_menubar_argv · function · L300-L309 — fn menubar_helper_pids_matches_only_exact_menubar_argv()
- PASSWD_PROBE_ENV · constant · L311-L311 — const PASSWD_PROBE_ENV: &str = "JCODE_MENUBAR_PASSWD_PROBE";
- passwd_home_probe_child · function · L318-L324 — fn passwd_home_probe_child()
- passwd_home_matches_getpwuid_and_ignores_home_env · function · L328-L367 — fn passwd_home_matches_getpwuid_and_ignores_home_env()
