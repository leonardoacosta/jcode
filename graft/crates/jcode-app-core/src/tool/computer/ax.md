# crates/jcode-app-core/src/tool/computer/ax.rs

- ElementHandle · struct · L21-L28 — pub struct ElementHandle
- resolve_script · function · L34-L40 — fn resolve_script(&self) -> String
- tell · function · L43-L52 — fn tell(app: &str, body: &str) -> String
- ui_tree · function · L55-L147 — pub fn ui_tree(app: Option<&str>, depth: u32) -> Result<ToolOutput>
- find_element · function · L150-L236 — pub fn find_element(
- press · function · L239-L249 — pub fn press(handle: &ElementHandle) -> Result<ToolOutput>
- perform_action · function · L252-L263 — pub fn perform_action(handle: &ElementHandle, ax_action: &str) -> Result<ToolOutput>
- set_value · function · L266-L279 — pub fn set_value(handle: &ElementHandle, value: &str) -> Result<ToolOutput>
- get_value · function · L282-L286 — pub fn get_value(handle: &ElementHandle) -> Result<ToolOutput>
- select_menu · function · L289-L317 — pub fn select_menu(app: &str, path: &[String]) -> Result<ToolOutput>
- element_at · function · L320-L403 — pub fn element_at(app: &str, x: f64, y: f64) -> Result<ToolOutput>
