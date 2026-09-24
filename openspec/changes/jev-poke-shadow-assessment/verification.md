# Verification record

Status: proposal awaiting approval. No application behavior has been implemented or runtime-validated.

## Requirement-to-acceptance mapping

| Requirement | Observable acceptance | Tasks |
| --- | --- | --- |
| Explicit session consent | Public commands, disclosure, status, invalid input, default-off network count | 3.1, 4.3 |
| Non-authoritative recommendations | Queue/todo parity for all labels/errors, waits and overnight exclusions | 3.2, 4.2, 4.3 |
| Bounded evidence | Captured provider request omits sentinel/raw fields and respects UTF-8/byte cap | 2.1, 4.2 |
| Strict typed assessment | Both provider routes, cache parity, malformed and uncertain abstentions | 1.2, 2.2, 4.2 |
| Bounded asynchronous lifecycle | Deadline, budget, deduplication, two-client lease, stale results, reconnect | 2.2, 2.3, 3.3, 4.3 |

Model-quality acceptance is intentionally separate from mechanics. A mocked provider validates integration, not judgment quality. The first feature cannot authorize active interventions.

## Authoring checks

- `openspec validate jev-poke-shadow-assessment --strict --no-interactive`: PASS.
- Requirement scenario checks: PASS, five requirements and sixteen WHEN/THEN scenarios.
- Principal source paths: PASS after correcting the moved protocol reference to `crates/jcode-protocol/src/wire.rs`.
- Dependency validator: BLOCKED, reports the prerequisite's missing execution frontmatter. The dependency resolves to an existing active proposal, but graph admission and implementation readiness are not claimed.

Dependency readiness is blocked: the prerequisite proposal exists, has unchecked implementation tasks, and lacks execution frontmatter. Do not amend another change just to make this proposal appear ready.
