# crates/jcode-provider-anthropic/src/repair_efficiency_tests.rs

- cpu_probe · module · L6-L6 — mod cpu_probe;
- native · function · L8-L13 — fn native(item: Value) -> ContentBlock
- message · function · L14-L21 — fn message(role: Role, blocks: Vec<ContentBlock>) -> Message
- text · function · L22-L27 — fn text() -> ContentBlock
- parity · function · L28-L33 — fn parity(messages: &[Message])
- frozen_oracle_exhaustive_native_shapes_and_final_pause · function · L36-L77 — fn frozen_oracle_exhaustive_native_shapes_and_final_pause()
- shares_only_unchanged_nonempty_messages · function · L80-L118 — fn shares_only_unchanged_nonempty_messages()
- legacy_repair · function · L120-L207 — fn legacy_repair(messages: &[Message]) -> Vec<Message>
