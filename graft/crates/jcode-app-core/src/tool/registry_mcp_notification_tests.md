# crates/jcode-app-core/src/tool/registry_mcp_notification_tests.rs

- SERVER · constant · L7-L36 — const SERVER: &str = r#"
- wait_for · function · L38-L49 — async fn wait_for(registry: &Registry, name: &str, present: bool)
- mcp_list · function · L51-L66 — async fn mcp_list(registry: &Registry) -> String
- wait_for_list · function · L68-L76 — async fn wait_for_list(registry: &Registry, needle: &str)
- run · function · L78-L118 — async fn run(opted_in: bool)
- opted_in_tools_list_changed_refreshes_the_model_facing_registry · function · L121-L123 — async fn opted_in_tools_list_changed_refreshes_the_model_facing_registry()
- opted_out_server_keeps_its_first_catalog · function · L126-L128 — async fn opted_out_server_keeps_its_first_catalog()
