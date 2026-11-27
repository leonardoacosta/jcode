# crates/jcode-app-core/src/ambient/manager.rs

- MISSED_GRACE_MINUTES · constant · L17-L17 — const MISSED_GRACE_MINUTES: i64 = 5;
- AmbientManager · struct · L19-L22 — pub struct AmbientManager
- new · function · L25-L34 — pub fn new() -> Result<Self>
- is_enabled · function · L36-L38 — pub fn is_enabled() -> bool
- should_run · function · L41-L52 — pub fn should_run(&self) -> bool
- record_cycle_result · function · L54-L64 — pub fn record_cycle_result(&mut self, result: AmbientCycleResult) -> Result<()>
- take_ready_items · function · L68-L70 — pub fn take_ready_items(&mut self) -> Vec<ScheduledItem>
- take_ready_direct_items · function · L74-L76 — pub fn take_ready_direct_items(&mut self) -> Vec<ScheduledItem>
- settle · function · L79-L81 — pub fn settle(&mut self, id: &str, outcome: Settlement) -> Result<bool>
- recover_scheduled_runs · function · L85-L88 — pub fn recover_scheduled_runs(&mut self) -> Result<RecoveryReport>
- retry_schedule · function · L91-L93 — pub fn retry_schedule(&mut self, id: &str) -> Result<Option<ScheduledItem>>
- schedule · function · L96-L101 — pub fn schedule(&mut self, request: ScheduleRequest) -> Result<String>
- scheduled_item · function · L103-L130 — pub(super) fn scheduled_item(request: ScheduleRequest) -> ScheduledItem
- cancel_schedule · function · L133-L135 — pub fn cancel_schedule(&mut self, id: &str) -> Result<Option<ScheduledItem>>
- state · function · L137-L139 — pub fn state(&self) -> &AmbientState
- queue · function · L141-L143 — pub fn queue(&self) -> &ScheduledQueue
