# crates/jcode-terminal-prompt/src/secret_pty_tests.rs

- drain · function · L18-L33 — fn drain(master: &mut std::fs::File, timeout: Duration) -> Vec<u8>
- read_secret_over_pty · function · L37-L112 — fn read_secret_over_pty(typed: &str) -> (String, String)
- typing_a_secret_on_a_tty_echoes_a_mask_and_never_the_secret · function · L118-L134 — fn typing_a_secret_on_a_tty_echoes_a_mask_and_never_the_secret()
- backspace_erases_one_mask_and_one_character · function · L139-L146 — fn backspace_erases_one_mask_and_one_character()
