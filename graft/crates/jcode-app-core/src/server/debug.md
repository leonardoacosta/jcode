# crates/jcode-app-core/src/server/debug.rs

- ChannelSubscriptions · type · L36-L36 — type ChannelSubscriptions = Arc<RwLock<HashMap<String, HashMap<String, HashSet<String>>>>>;
- debug_command_authorized · function · L38-L42 — fn debug_command_authorized(command: &str, general_debug_control: bool) -> bool
- ClientDebugState · struct · L45-L48 — pub(super) struct ClientDebugState
- ClientConnectionInfo · struct · L51-L65 — pub(super) struct ClientConnectionInfo
- register · function · L68-L71 — pub(super) fn register(&mut self, client_id: String, tx: mpsc::UnboundedSender<(u64, String)>)
- unregister · function · L73-L78 — pub(super) fn unregister(&mut self, client_id: &str)
- active_sender · function · L80-L94 — pub(super) fn active_sender(
- sender_for_id · function · L96-L101 — pub(super) fn sender_for_id(
- resolve_client_debug_sender · function · L104-L156 — async fn resolve_client_debug_sender(
- resolve_transcript_target_session · function · L158-L214 — async fn resolve_transcript_target_session(
- inject_transcript · function · L216-L265 — pub(super) async fn inject_transcript(
- forward_voice_activity · function · L269-L282 — pub(super) async fn forward_voice_activity(
- is_event_subscription_command · function · L287-L291 — fn is_event_subscription_command(command: &str) -> bool
- event_subscription_tests · module · L294-L310 — mod event_subscription_tests
- passive_event_streams_do_not_reserve_activity · function · L298-L309 — fn passive_event_streams_do_not_reserve_activity()
- handle_debug_client · function · L316-L716 — pub(super) async fn handle_debug_client(
- debug_tests · module · L720-L720 — mod debug_tests;
