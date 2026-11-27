# crates/jcode-app-core/src/turn_cancel_registry.rs

- NEXT_TOKEN · constant · L31-L31 — static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);
- SessionTurnSignals · type · L33-L33 — type SessionTurnSignals = Vec<(u64, InterruptSignal)>;
- ACTIVE_TURNS · constant · L34-L35 — static ACTIVE_TURNS: LazyLock<Mutex<HashMap<String, SessionTurnSignals>>> =
- ActiveTurnGuard · struct · L45-L49 — pub struct ActiveTurnGuard
- register_active_turn · function · L53-L72 — pub fn register_active_turn(session_id: &str, signal: InterruptSignal) -> ActiveTurnGuard
- active_turn_signals · function · L75-L84 — pub fn active_turn_signals(session_id: &str) -> Vec<InterruptSignal>
- active_session_ids · function · L88-L97 — pub fn active_session_ids() -> Vec<String>
- rename_active_turns · function · L110-L133 — pub fn rename_active_turns(old_session_id: &str, new_session_id: &str)
- drop · function · L136-L166 — fn drop(&mut self)
- has_active_turn · function · L178-L183 — pub fn has_active_turn(session_id: &str) -> bool
- tests · module · L186-L331 — mod tests
- active_session_snapshot_tracks_unique_ids_rename_and_drop · function · L190-L210 — fn active_session_snapshot_tracks_unique_ids_rename_and_drop()
- register_and_drop_tracks_active_signals · function · L213-L228 — fn register_and_drop_tracks_active_signals()
- has_active_turn_tracks_registration_lifetime · function · L236-L248 — fn has_active_turn_tracks_registration_lifetime()
- multiple_turns_for_one_session_are_all_listed · function · L251-L271 — fn multiple_turns_for_one_session_are_all_listed()
- rename_moves_active_turns_to_the_new_session_id · function · L277-L304 — fn rename_moves_active_turns_to_the_new_session_id()
- dropping_a_renamed_guard_leaves_no_orphan_entry · function · L310-L330 — fn dropping_a_renamed_guard_leaves_no_orphan_entry()
