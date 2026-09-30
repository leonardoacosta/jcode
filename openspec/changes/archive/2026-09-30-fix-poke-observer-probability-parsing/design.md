## Context
`parse_poke_assessment` in `crates/jcode-app-core/src/agent/poke_shadow.rs` validates probability range, total, selected choice, and display thresholds. Its current `v.as_f64().unwrap_or(0.0)` accepts malformed values as zero. Existing parser tests cover valid, uncertain, inconsistent, and bad-sum responses. The repository has unrelated active edits that must remain untouched.

## Goals / Non-Goals
Goals: fail closed on nonnumeric probability entries and prove valid numeric zeros still work.
Non-goals: change observer thresholds, activate shadow recommendations, integrate new overnight observers, or modify scheduling.

## Decisions
Replace zero fallback with explicit numeric decoding failure returning the existing malformed-response result. This preserves the public return type and abstention behavior. A tests-only change would leave the defect intact. A broader distribution-schema rewrite would add compatibility risk without being necessary for this fix.
Use table-driven malformed-value regression cases with numeric entries already totaling one so sum validation cannot mask the defect. Include a valid numeric-zero control.

## Risks / Trade-offs
Previously tolerated malformed provider output will now abstain. This is intentional: observer recommendations must not depend on silently repaired evidence. Missing-answer handling and optional provider confidence remain unchanged.

## Migration Plan
No migration or configuration change. Run targeted package parser tests, then the package test suite where feasible, inspect the scoped diff, and commit only the fix and its approved planning artifacts. Reverting the isolated commit restores prior behavior.

## Open Questions
None for this bounded fix. Overnight run isolation and broader observer integration remain separate work.
