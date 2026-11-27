# crates/jcode-app-core/src/tool/inflight.rs

- IN_FLIGHT · constant · L30-L31 — static IN_FLIGHT: LazyLock<Mutex<HashMap<String, usize>>> =
- InFlightToolGuard · struct · L34-L36 — pub struct InFlightToolGuard
- drop · function · L39-L49 — fn drop(&mut self)
- mark_tool_in_flight · function · L54-L63 — pub fn mark_tool_in_flight(tool_call_id: &str) -> Option<InFlightToolGuard>
- is_tool_in_flight · function · L66-L71 — pub fn is_tool_in_flight(tool_call_id: &str) -> bool
- in_flight_tool_count · function · L74-L76 — pub fn in_flight_tool_count() -> usize
- tests · module · L79-L110 — mod tests
- guard_tracks_and_releases · function · L83-L91 — fn guard_tracks_and_releases()
- nested_guards_release_only_after_the_last_one · function · L94-L103 — fn nested_guards_release_only_after_the_last_one()
- empty_ids_are_not_tracked · function · L106-L109 — fn empty_ids_are_not_tracked()
