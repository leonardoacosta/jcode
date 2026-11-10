# session-cache-affinity Specification

## Purpose
Preserve stable, opt-in cache identity for named-provider sessions across continuation, resume and forks, and require verified selfdev RC acceptance before shared promotion.

## Requirements

### Requirement: Opt-in named-provider session cache affinity
Named compatible providers SHALL default to no generated cache affinity. An opted-in profile SHALL send an opaque profile/session-scoped prompt_cache_key and matching x-session-id without changing unrelated providers.

#### Scenario: Stable session lifecycle
- **WHEN** an opted-in session continues, reconnects, resumes or rebuilds its runtime
- **THEN** its generated identity remains stable and independent sessions or child session bindings have distinct identities

#### Scenario: Explicit identity override
- **WHEN** explicit request or header identity is configured
- **THEN** the effective body and header identity agree, invalid or conflicting values fail before network, and opted-out request behavior remains unchanged

### Requirement: RC-gated promotion
The changed executable SHALL be tested through selfdev isolated runtime acceptance before shared promotion. Failed or unverifiable required gates SHALL prevent promotion.

#### Scenario: Real session acceptance
- **WHEN** the RC runs tool-enabled append-only turns and session resume against the actual gateway
- **THEN** runtime identity and cache behavior are observed on the exact RC binary before any shared activation
