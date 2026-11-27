# crates/jcode-app-core/src/mission.rs

- MissionStatus · enum · L10-L18 — pub enum MissionStatus
- as_str · function · L21-L31 — pub fn as_str(&self) -> &'static str
- MissionCheckpoint · struct · L35-L38 — pub struct MissionCheckpoint
- Mission · struct · L41-L56 — pub struct Mission
- load · function · L58-L64 — pub fn load(session_id: &str) -> Result<Option<Mission>>
- set · function · L66-L90 — pub fn set(session_id: &str, objective: &str) -> Result<Mission>
- update_status · function · L92-L100 — pub fn update_status(session_id: &str, status: MissionStatus) -> Result<Option<Mission>>
- checkpoint · function · L102-L117 — pub fn checkpoint(session_id: &str, summary: &str) -> Result<Option<Mission>>
- clear · function · L119-L127 — pub fn clear(session_id: &str) -> Result<bool>
- render_status · function · L129-L141 — pub fn render_status(mission: &Mission) -> String
- active_system_reminder · function · L143-L151 — pub fn active_system_reminder(session_id: &str) -> Result<Option<String>>
- render_mission_continuation_prompt · function · L153-L160 — pub fn render_mission_continuation_prompt(mission: &Mission) -> String
- escape_xml_text · function · L162-L167 — fn escape_xml_text(input: &str) -> String
- save · function · L169-L171 — fn save(mission: &Mission) -> Result<()>
- mission_path · function · L173-L177 — fn mission_path(session_id: &str) -> Result<PathBuf>
- sanitize_session_id · function · L179-L190 — fn sanitize_session_id(session_id: &str) -> String
- default_long_horizon_intent · function · L192-L197 — fn default_long_horizon_intent(objective: &str) -> String
