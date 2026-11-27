# crates/jcode-tui/src/tui/app/terminal_liveness.rs

- INITIAL_TTY_NR · constant · L26-L26 — static INITIAL_TTY_NR: OnceLock<u64> = OnceLock::new();
- capture_initial_tty · function · L31-L33 — pub(crate) fn capture_initial_tty()
- terminal_abandoned · function · L37-L49 — pub(crate) fn terminal_abandoned() -> bool
- current_tty_nr · function · L52-L55 — fn current_tty_nr() -> Option<u64>
- current_tty_nr · function · L58-L70 — fn current_tty_nr() -> Option<u64>
- current_tty_nr · function · L73-L75 — fn current_tty_nr() -> Option<u64>
- parse_tty_nr · function · L81-L88 — fn parse_tty_nr(stat: &str) -> Option<u64>
- tests · module · L91-L110 — mod tests
- parses_tty_nr_from_stat_line · function · L95-L98 — fn parses_tty_nr_from_stat_line()
- parses_tty_nr_with_hostile_comm · function · L101-L104 — fn parses_tty_nr_with_hostile_comm()
- missing_paren_yields_none · function · L107-L109 — fn missing_paren_yields_none()
