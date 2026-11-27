# crates/jcode-provider-openrouter/src/request.rs

- sanitize_tool_parameters_schema · function · L20-L22 — pub fn sanitize_tool_parameters_schema(schema: &Value) -> Value
- orphan_tool_output_to_user_message · function · L24-L42 — fn orphan_tool_output_to_user_message(
- build_chat_messages · function · L49-L575 — pub fn build_chat_messages(
- request_tests · module · L578-L673 — mod request_tests
- tool_call · function · L583-L599 — fn tool_call(id: &str, output: &str) -> [Message; 2]
- repeated_tool_call_ids_get_their_own_outputs_in_order · function · L602-L616 — fn repeated_tool_call_ids_get_their_own_outputs_in_order()
- orphaned_tool_output_is_recovered_as_a_user_message · function · L619-L631 — fn orphaned_tool_output_is_recovered_as_a_user_message()
- multi_turn_replay_preserves_vertex_tool_call_thought_signatures · function · L634-L672 — fn multi_turn_replay_preserves_vertex_tool_call_thought_signatures()
- sanitize_schema_tests · module · L676-L822 — mod sanitize_schema_tests
- top_level_any_of_is_flattened_for_anthropic_family_upstreams · function · L681-L733 — fn top_level_any_of_is_flattened_for_anthropic_family_upstreams()
- top_level_all_of_promotes_required_fields · function · L736-L761 — fn top_level_all_of_promotes_required_fields()
- bare_object_schema_gains_empty_properties · function · L764-L768 — fn bare_object_schema_gains_empty_properties()
- empty_and_non_object_schemas_become_empty_object_schema · function · L771-L775 — fn empty_and_non_object_schemas_become_empty_object_schema()
- existing_properties_and_unrelated_fields_are_preserved · function · L778-L786 — fn existing_properties_and_unrelated_fields_are_preserved()
- nested_object_schemas_are_sanitized_recursively · function · L789-L815 — fn nested_object_schemas_are_sanitized_recursively()
- type_arrays_including_object_are_sanitized · function · L818-L821 — fn type_arrays_including_object_are_sanitized()
