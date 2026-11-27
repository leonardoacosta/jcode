# crates/jcode-app-core/src/server/client_comm_message.rs

- ChannelSubscriptions · type · L15-L15 — type ChannelSubscriptions = Arc<RwLock<HashMap<String, HashMap<String, HashSet<String>>>>>;
- swarm_id_for_session · function · L17-L23 — async fn swarm_id_for_session(
- friendly_name_for_session · function · L25-L33 — async fn friendly_name_for_session(
- resolve_dm_target_session · function · L35-L80 — async fn resolve_dm_target_session(
- resolve_comm_delivery_mode · function · L82-L97 — fn resolve_comm_delivery_mode(
- handle_comm_message · function · L103-L601 — pub(super) async fn handle_comm_message(
