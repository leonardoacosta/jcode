# crates/jcode-config-types/src/task_policy.rs

- TasksConfig · struct · L5-L11 — pub struct TasksConfig
- default · function · L14-L22 — fn default() -> Self
- validate · function · L26-L38 — pub fn validate(&self) -> Result<(), String>
- admit · function · L42-L147 — pub fn admit(&self, task: &TaskAdmissionSnapshot) -> AdmissionDecision
- WorktreesConfig · struct · L152-L158 — pub struct WorktreesConfig
- default · function · L161-L168 — fn default() -> Self
- validate · function · L172-L192 — pub fn validate(&self) -> Result<(), String>
- WorktreeCleanup · enum · L197-L200 — pub enum WorktreeCleanup
- TaskAuthorization · struct · L204-L209 — pub struct TaskAuthorization
- TaskAdmissionSnapshot · struct · L213-L227 — pub struct TaskAdmissionSnapshot
- AdmissionState · enum · L231-L236 — pub enum AdmissionState
- AdmissionDecision · struct · L239-L245 — pub struct AdmissionDecision
- tests · module · L248-L378 — mod tests
- admission_resource_deadline_setup_and_ownership_edges · function · L252-L305 — fn admission_resource_deadline_setup_and_ownership_edges()
- task_policy_defaults_fail_closed_and_limits_validate · function · L308-L320 — fn task_policy_defaults_fail_closed_and_limits_validate()
- ready · function · L322-L341 — fn ready() -> (TasksConfig, TaskAdmissionSnapshot)
- admission_has_no_clock_window_and_never_widens_authorization · function · L344-L357 — fn admission_has_no_clock_window_and_never_widens_authorization()
- admission_explains_disabled_unknown_cancelled_dependencies_and_duplicate · function · L360-L377 — fn admission_explains_disabled_unknown_cancelled_dependencies_and_duplicate()
