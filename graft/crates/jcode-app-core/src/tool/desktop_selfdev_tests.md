# crates/jcode-app-core/src/tool/desktop_selfdev_tests.rs

- input · function · L3-L5 — fn input(action: &str) -> Input
- context · function · L7-L17 — fn context(working_dir: Option<PathBuf>) -> ToolContext
- checkout · function · L19-L28 — fn checkout() -> tempfile::TempDir
- every_action_rejects_non_desktop_and_absent_context · function · L31-L53 — async fn every_action_rejects_non_desktop_and_absent_context()
- builds_both_desktop_packages_with_matching_profile · function · L56-L69 — fn builds_both_desktop_packages_with_matching_profile()
- schema_advertises_only_one_desktop_rebuild_and_reload_action · function · L72-L84 — fn schema_advertises_only_one_desktop_rebuild_and_reload_action()
- reload_actions_never_route_to_cli_or_subprocess · function · L87-L91 — fn reload_actions_never_route_to_cli_or_subprocess()
- test_and_inspection_commands_are_scoped_and_read_only · function · L94-L110 — fn test_and_inspection_commands_are_scoped_and_read_only()
- screenshot_uses_private_script_with_fresh_build_and_target_output · function · L113-L133 — fn screenshot_uses_private_script_with_fresh_build_and_target_output()
- screenshot_rejects_symlink_destination · function · L137-L141 — fn screenshot_rejects_symlink_destination()
- ambiguous_instances_and_foreign_executables_are_rejected · function · L144-L182 — fn ambiguous_instances_and_foreign_executables_are_rejected()
- custom_test_executes_from_detected_repo_root · function · L185-L200 — async fn custom_test_executes_from_detected_repo_root()
- invalid_timeout_and_action_fail_before_execution · function · L203-L218 — async fn invalid_timeout_and_action_fail_before_execution()
- command_timeout_is_bounded_and_failures_are_reported · function · L221-L242 — async fn command_timeout_is_bounded_and_failures_are_reported()
- process_output_is_bounded · function · L245-L250 — async fn process_output_is_bounded()
- reload_sends_only_r_and_requires_real_acknowledgement · function · L254-L273 — async fn reload_sends_only_r_and_requires_real_acknowledgement()
- unsafe_sockets_are_rejected_before_connecting · function · L277-L290 — async fn unsafe_sockets_are_rejected_before_connecting()
- screenshot_execution_never_discovers_live_instances · function · L293-L320 — async fn screenshot_execution_never_discovers_live_instances()
