# src/cli/debug.rs

- run_terminal_command · function · L6-L12 — pub fn run_terminal_command() -> Result<()>
- terminal_capability_report · function · L14-L51 — fn terminal_capability_report() -> serde_json::Value
- run_instructions_command · function · L54-L64 — pub fn run_instructions_command(dir: Option<&std::path::Path>) -> Result<()>
- instructions_report · function · L66-L74 — fn instructions_report(dir: &std::path::Path) -> String
- run_debug_command · function · L76-L174 — pub async fn run_debug_command(
- debug_list_servers · function · L176-L254 — async fn debug_list_servers() -> Result<()>
- get_server_info · function · L256-L290 — async fn get_server_info(debug_socket: &std::path::Path) -> Result<String>
- debug_start_server · function · L292-L364 — async fn debug_start_server(arg: &str, socket_path: Option<String>) -> Result<()>
- instructions_tests · module · L367-L463 — mod instructions_tests
- terminal_report_names_detected_capabilities_and_fallbacks · function · L371-L381 — fn terminal_report_names_detected_capabilities_and_fallbacks()
- terminal_report_no_color_environment · function · L384-L412 — fn terminal_report_no_color_environment()
- terminal_report_explicit_fallback_environment · function · L415-L445 — fn terminal_report_explicit_fallback_environment()
- instructions_lists_project_file · function · L448-L455 — fn instructions_lists_project_file()
- instructions_rejects_invalid_dir · function · L458-L462 — fn instructions_rejects_invalid_dir()
