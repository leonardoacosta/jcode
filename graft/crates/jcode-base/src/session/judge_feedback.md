# crates/jcode-base/src/session/judge_feedback.rs

- FeedbackAdmission · enum · L9-L12 — pub enum FeedbackAdmission
- message · function · L15-L19 — pub fn message(&self) -> &str
- AdmissionState · struct · L23-L28 — struct AdmissionState
- validate_id · function · L30-L40 — fn validate_id(id: &str) -> Result<()>
- update_state · function · L42-L107 — fn update_state<T>(parent: &str, update: impl FnOnce(&mut AdmissionState) -> T) -> Result<T>
- begin_user_turn · function · L110-L112 — pub fn begin_user_turn(parent: &str) -> Result<()>
- begin_user_turn_once · function · L116-L123 — pub fn begin_user_turn_once(parent: &str, token: &str) -> Result<()>
- admit · function · L127-L167 — pub fn admit(
