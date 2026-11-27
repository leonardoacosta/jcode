# crates/jcode-tui/src/tui/backend_cloud.rs

- bind_cloud_authority · function · L5-L7 — pub(crate) fn bind_cloud_authority(&mut self, live: &Arc<std::sync::atomic::AtomicBool>)
- revoke_cloud_authority · function · L9-L17 — pub(super) fn revoke_cloud_authority(&mut self)
- track_cloud_callback · function · L18-L21 — pub(crate) fn track_cloud_callback(&mut self, task: &tokio::task::JoinHandle<()>)
- configure_cloud_tools · function · L22-L38 — pub(crate) async fn configure_cloud_tools(
- clear_local_tools · function · L40-L52 — pub(super) async fn clear_local_tools(&mut self) -> Result<()>
- finish_local_tools_request · function · L54-L61 — pub(crate) fn finish_local_tools_request(&mut self, id: u64, accepted: bool) -> bool
- cloud_callback_sender · function · L63-L91 — pub(crate) fn cloud_callback_sender(
- cloud_expiry_revoke · function · L93-L103 — pub(crate) fn cloud_expiry_revoke(
