# crates/jcode-app-core/src/server/client_comm_channels.rs

- ChannelSubscriptions · type · L11-L11 — type ChannelSubscriptions = Arc<RwLock<HashMap<String, HashMap<String, HashSet<String>>>>>;
- swarm_id_for_session · function · L13-L19 — async fn swarm_id_for_session(
- handle_comm_list_channels · function · L21-L58 — pub(super) async fn handle_comm_list_channels(
- handle_comm_channel_members · function · L60-L114 — pub(super) async fn handle_comm_channel_members(
- handle_comm_subscribe_channel · function · L120-L199 — pub(super) async fn handle_comm_subscribe_channel(
- handle_comm_unsubscribe_channel · function · L205-L284 — pub(super) async fn handle_comm_unsubscribe_channel(
