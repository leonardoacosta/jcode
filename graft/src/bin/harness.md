# src/bin/harness.rs

- Args · struct · L14-L22 — struct Args
- NoopProvider · struct · L24-L24 — struct NoopProvider;
- complete · function · L28-L36 — async fn complete(
- name · function · L38-L40 — fn name(&self) -> &str
- fork · function · L42-L44 — fn fork(&self) -> Arc<dyn Provider>
- available_models_display · function · L46-L48 — fn available_models_display(&self) -> Vec<String>
- prefetch_models · function · L50-L52 — async fn prefetch_models(&self) -> Result<()>
- ToolCase · struct · L55-L59 — struct ToolCase
- main · function · L62-L195 — async fn main() -> Result<()>
- create_temp_workspace · function · L197-L201 — fn create_temp_workspace() -> Result<PathBuf>
