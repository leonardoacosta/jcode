# crates/jcode-app-core/src/tool/todo_names.rs

- NULL_CLEARS · constant · L6-L11 — const NULL_CLEARS: &[&str] = &[
- normalize_containers · function · L13-L62 — pub(super) fn normalize_containers(input: &mut Value)
- uses_names · function · L64-L70 — pub(super) fn uses_names(value: &Value) -> bool
- task_name · function · L72-L74 — fn task_name(todo: &Value) -> Option<&str>
- resolve_name · function · L76-L88 — fn resolve_name(todos: &[Value], name: &str) -> Result<String>
- normalize_item · function · L90-L117 — fn normalize_item(item: &mut Value, previous: &[Value]) -> Result<bool>
- resolve_dependencies · function · L119-L135 — fn resolve_dependencies(value: &mut Value, catalog: &[Value]) -> Result<()>
- normalize_public_input · function · L139-L301 — pub(super) fn normalize_public_input(input: &mut Value, previous: &[Value]) -> Result<()>
- rename_field · function · L142-L152 — fn rename_field(value: &mut Value, public: &str, internal: &str) -> Result<()>
- validate_unique_names · function · L303-L314 — fn validate_unique_names(todos: &[Value]) -> Result<()>
- public_todos · function · L316-L346 — pub(super) fn public_todos(todos: &[TodoItem]) -> Result<Value>
