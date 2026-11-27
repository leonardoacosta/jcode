# crates/jcode-app-core/src/tool/selfdev/launch.rs

- enter_selfdev_session · function · L3-L73 — pub fn enter_selfdev_session(
- schedule_selfdev_prompt_delivery · function · L75-L97 — pub fn schedule_selfdev_prompt_delivery(session_id: String, prompt: String)
- send_prompt_to_session · function · L100-L119 — async fn send_prompt_to_session(session_id: &str, prompt: &str) -> Result<()>
- try_send_prompt_once · function · L121-L137 — async fn try_send_prompt_once(session_id: &str, prompt: &str) -> Result<()>
- do_enter · function · L139-L224 — pub(super) async fn do_enter(
