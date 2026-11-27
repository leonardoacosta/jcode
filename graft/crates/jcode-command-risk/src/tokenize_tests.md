# crates/jcode-command-risk/src/tokenize_tests.rs

- heredoc_payload_is_data_not_shell_source · function · L7-L14 — fn heredoc_payload_is_data_not_shell_source()
- heredoc_terminator_does_not_hide_following_command · function · L17-L21 — fn heredoc_terminator_does_not_hide_following_command()
- tab_stripping_and_escaped_delimiters_are_supported · function · L24-L33 — fn tab_stripping_and_escaped_delimiters_are_supported()
- heredoc_declared_inside_command_substitution_is_found · function · L36-L49 — fn heredoc_declared_inside_command_substitution_is_found()
- texts · function · L51-L53 — fn texts(command: &str) -> Vec<String>
- quoting_does_not_hide_a_target · function · L56-L61 — fn quoting_does_not_hide_a_target()
- backslash_escapes_are_resolved · function · L64-L69 — fn backslash_escapes_are_resolved()
- chained_commands_split_into_separate_segments · function · L72-L77 — fn chained_commands_split_into_separate_segments()
- piped_commands_are_separate_segments · function · L80-L84 — fn piped_commands_are_separate_segments()
- truncating_redirect_target_is_marked_but_append_is_not · function · L87-L100 — fn truncating_redirect_target_is_marked_but_append_is_not()
- fd_duplication_is_not_a_truncating_redirect · function · L103-L128 — fn fd_duplication_is_not_a_truncating_redirect()
- recursive_flag_detected_in_bundles_and_long_form · function · L131-L145 — fn recursive_flag_detected_in_bundles_and_long_form()
- basename_strips_the_path_so_absolute_invocations_match · function · L148-L151 — fn basename_strips_the_path_so_absolute_invocations_match()
- tokenizing_is_total_and_never_panics · function · L154-L168 — fn tokenizing_is_total_and_never_panics()
- parentheses_split_subshell_and_command_substitution_bodies · function · L171-L181 — fn parentheses_split_subshell_and_command_substitution_bodies()
