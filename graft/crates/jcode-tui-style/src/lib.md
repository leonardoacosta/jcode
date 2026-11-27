# crates/jcode-tui-style/src/lib.rs

- color · module · L1-L1 — pub mod color;
- harmony · module · L2-L2 — pub mod harmony;
- palette · module · L3-L3 — pub mod palette;
- theme · module · L4-L4 — pub mod theme;
- theme_mode · module · L5-L5 — pub mod theme_mode;
- STYLE_TEST_LOCK · constant · L20-L20 — pub(crate) static STYLE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
- restore_terminal_quietly · function · L32-L36 — pub fn restore_terminal_quietly()
- catch_recoverable_panic · function · L51-L61 — pub fn catch_recoverable_panic<R>(f: impl FnOnce() -> R) -> std::thread::Result<R>
- Scope · struct · L52-L52 — struct Scope;
- drop · function · L54-L56 — fn drop(&mut self)
- panic_will_be_recovered · function · L64-L66 — pub fn panic_will_be_recovered() -> bool
- install_panic_hook_with · function · L68-L80 — fn install_panic_hook_with(restore: fn())
- init_terminal · function · L88-L97 — pub fn init_terminal() -> std::io::Result<ratatui::DefaultTerminal>
- HOOK · constant · L91-L91 — static HOOK: std::sync::Once = std::sync::Once::new();
- panic_hook_tests · module · L100-L133 — mod panic_hook_tests
- count_restore · function · L107-L109 — fn count_restore()
- recovered_panics_do_not_restore_the_terminal · function · L112-L132 — fn recovered_panics_do_not_restore_the_terminal()
