# crates/jcode-provider-doctor/src/live_provider_probes/tests.rs

- gemini_openai_compat_strips_models_prefix_from_catalog_ids · function · L6-L17 — fn gemini_openai_compat_strips_models_prefix_from_catalog_ids()
- non_gemini_openai_compat_leaves_model_ids_untouched · function · L20-L31 — fn non_gemini_openai_compat_leaves_model_ids_untouched()
- tool_call_with_signature · function · L33-L40 — fn tool_call_with_signature(signature: Option<&str>) -> NativeClaudeToolCall
- reasoning_capability_classifies_streamed_when_reasoning_text_present · function · L43-L50 — fn reasoning_capability_classifies_streamed_when_reasoning_text_present()
- reasoning_capability_classifies_opaque_from_thinking_signature · function · L53-L61 — fn reasoning_capability_classifies_opaque_from_thinking_signature()
- reasoning_capability_classifies_opaque_from_tool_thought_signature · function · L64-L73 — fn reasoning_capability_classifies_opaque_from_tool_thought_signature()
- reasoning_capability_classifies_none_without_any_signal · function · L76-L84 — fn reasoning_capability_classifies_none_without_any_signal()
- reasoning_capability_prefers_streamed_over_opaque · function · L87-L97 — fn reasoning_capability_prefers_streamed_over_opaque()
- parallel_tool_use_replays_every_signature_in_one_assistant_message · function · L100-L148 — fn parallel_tool_use_replays_every_signature_in_one_assistant_message()
