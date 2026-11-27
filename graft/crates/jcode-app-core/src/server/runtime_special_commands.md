# crates/jcode-app-core/src/server/runtime_special_commands.rs

- status_args · function · L8-L14 — fn status_args(args: &[String]) -> Result<()>
- premium_arg · function · L16-L42 — fn premium_arg(
- session_manifest · function · L45-L75 — fn session_manifest(session_id: &str) -> Result<Option<crate::overnight::OvernightManifest>>
- execute · function · L77-L156 — pub(super) async fn execute(
- logout · function · L158-L269 — fn logout(args: &[String]) -> Result<(Value, String)>
- remote · function · L271-L309 — fn remote(args: &[String]) -> Result<(Value, String)>
- overnight · function · L311-L432 — async fn overnight(agent: &mut Agent, args: &[String]) -> Result<(Value, String)>
- tests · module · L435-L496 — mod tests
- premium_modes_are_exact_and_invalid_input_never_defaults · function · L438-L466 — fn premium_modes_are_exact_and_invalid_input_never_defaults()
- logout_requires_explicit_supported_provider_and_confirmation · function · L468-L477 — fn logout_requires_explicit_supported_provider_and_confirmation()
- remote_help_and_cloud_are_side_effect_free · function · L480-L487 — fn remote_help_and_cloud_are_side_effect_free()
- read_only_commands_reject_mutation_arguments · function · L490-L495 — fn read_only_commands_reject_mutation_arguments()
