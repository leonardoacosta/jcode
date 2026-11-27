# crates/jcode-app-core/src/server/swarm_channels.rs

- ChannelSubscriptions · type · L6-L6 — type ChannelSubscriptions = Arc<RwLock<HashMap<String, HashMap<String, HashSet<String>>>>>;
- with_channel_index_mut · function · L8-L22 — async fn with_channel_index_mut(
- remove_session_channel_subscriptions · function · L24-L35 — pub(super) async fn remove_session_channel_subscriptions(
- subscribe_session_to_channel · function · L37-L50 — pub(super) async fn subscribe_session_to_channel(
- unsubscribe_session_from_channel · function · L52-L65 — pub(super) async fn unsubscribe_session_from_channel(
- list_channels_for_swarm · function · L67-L88 — pub(super) async fn list_channels_for_swarm(
