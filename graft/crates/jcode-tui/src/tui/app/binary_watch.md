# crates/jcode-tui/src/tui/app/binary_watch.rs

- PENDING · constant · L12-L12 — static PENDING: AtomicBool = AtomicBool::new(false);
- STARTED · constant · L13-L13 — static STARTED: AtomicBool = AtomicBool::new(false);
- take_pending · function · L16-L18 — pub(super) fn take_pending() -> bool
- rearm · function · L21-L23 — pub(super) fn rearm()
- ensure_started · function · L27-L59 — pub(super) fn ensure_started()
- imp · module · L62-L125 — mod imp
- DEBOUNCE_MS · constant · L69-L69 — const DEBOUNCE_MS: i32 = 750;
- run · function · L71-L124 — pub(super) fn run(dirs: Vec<PathBuf>)
