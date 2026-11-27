# src/cli/hibernate_wait.rs

- WAIT_ENV · constant · L11-L11 — pub const WAIT_ENV: &str = "JCODE_HIBERNATE_WAIT";
- is_invocation · function · L13-L15 — pub fn is_invocation() -> bool
- waiter_command · function · L18-L30 — pub fn waiter_command(session_id: &str) -> std::io::Result<std::process::Command>
- run · function · L32-L44 — pub fn run() -> anyhow::Result<()>
- draw_notice · function · L46-L58 — fn draw_notice()
- wait_for_input · function · L60-L90 — fn wait_for_input()
