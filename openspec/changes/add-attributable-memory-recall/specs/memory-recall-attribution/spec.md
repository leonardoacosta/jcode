## ADDED Requirements

### Requirement: Injected memory is attributable

Every memory item injected into model context SHALL carry its stored memory id, source, trust level and last-updated time in the `MemoryInjected` event, and the model-visible block SHALL include the memory id.

#### Scenario: Recalled item carries provenance
- **WHEN** a relevant active memory is injected
- **THEN** the `MemoryInjected` event item includes its `id`, `source`, `trust` and `updated_at`, and the injected text includes the id

#### Scenario: Old event log without new fields
- **WHEN** a memory event log written before this change is read
- **THEN** parsing succeeds with the new fields absent instead of failing

### Requirement: Injected memory is bounded and current

Recall SHALL NOT inject inactive or superseded memories and SHALL cap the number of items and total characters, recording any truncation.

#### Scenario: Superseded memory excluded
- **WHEN** a candidate memory has `active = false` or a `superseded_by` id
- **THEN** it is not injected

#### Scenario: Over-budget recall truncated
- **WHEN** relevant candidates exceed the item or char cap
- **THEN** only items within the cap are injected and the event records the dropped count
