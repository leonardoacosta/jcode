# crates/jcode-provider-anthropic/src/repair_cow_tests.rs

- msg · function · L4-L10 — fn msg(role: Role, content: Vec<ContentBlock>) -> Message
- call · function · L11-L18 — fn call(id: &str) -> ContentBlock
- result · function · L19-L29 — fn result(id: &str, placeholder: bool) -> ContentBlock
- text · function · L30-L35 — fn text() -> ContentBlock
- Repair · type · L36-L36 — type Repair = fn(&[Message]) -> Vec<Message>;
- repairs · function · L37-L43 — fn repairs() -> [(&'static str, Repair, Repair); 3]
- parity · function · L44-L52 — fn parity(history: &[Message])
- frozen_oracles_duplicate_attachment_empty_and_metadata_parity · function · L54-L99 — fn frozen_oracles_duplicate_attachment_empty_and_metadata_parity()
- changed_and_unchanged_payload_sharing · function · L101-L128 — fn changed_and_unchanged_payload_sharing()
- dedupe_keeps_empty_messages_and_detaches_only_changed_payloads · function · L131-L165 — fn dedupe_keeps_empty_messages_and_detaches_only_changed_payloads()
- probe · module · L169-L169 — mod probe;
- legacy_hoist · function · L171-L266 — fn legacy_hoist(messages: &[Message]) -> Vec<Message>
- legacy_dedupe_uses · function · L268-L308 — fn legacy_dedupe_uses(messages: &[Message]) -> Vec<Message>
- legacy_dedupe_results · function · L314-L385 — fn legacy_dedupe_results(messages: &[Message]) -> Vec<Message>
