# Continuation evidence delta

## Purpose

Assess whether existing logs can support diagnosis of continuation decisions, then establish a bounded, offline regression baseline for the existing engines. This evidence-only capability does not change continuation behavior.

## ADDED Requirements

### Requirement: Existing-log reuse and coverage assessment
The system SHALL assess existing OmniRoute log coverage and existing Jcode logs before proposing or adding logging. The assessment SHALL state the target diagnosis, which existing sources were examined, the events and fields each source can evidence, and gaps or limits. Current source verifies `AUTO_POKE_DECISION` idle reasons `no_todos`, `unchanged_ownership_assessment`, and `unchanged_todos`, plus `queue_continuation`, at `crates/jcode-tui/src/tui/app/input.rs`; this does not establish coverage of every branch or session correlation. The assessment SHALL distinguish routed inference traffic visible to OmniRoute from local decisions that do not route traffic.

#### Scenario: Existing sources assessed first
- **GIVEN** a target continuation diagnosis and existing OmniRoute and Jcode logs
- **WHEN** coverage is assessed
- **THEN** the assessment documents relevant events and fields from both sources before recommending any logging addition

#### Scenario: Local stop is not inferred from gateway absence
- **GIVEN** OmniRoute can observe routed inference traffic but not a local TUI stop or wait decision
- **WHEN** no gateway request appears for the relevant interval
- **THEN** the assessment does not treat that absence as proof of a local decision or its reason

#### Scenario: Source coverage is bounded
- **GIVEN** the current source identifies the stated idle reasons and `queue_continuation` at the specified file
- **WHEN** the assessment describes source coverage
- **THEN** it does not claim every decision branch or session correlation is covered without additional evidence

### Requirement: Evidence limits and provenance
The system SHALL record deployed capture, retention, correlation, and provenance as `verified`, `unknown`, `unavailable`, or `missing` only when the assessment identifies supporting evidence. Unknown or missing data SHALL NOT support an attributed conclusion. Source-pinned upstream documentation SHALL NOT be represented as evidence of deployed behavior. Agent claims, confidence, tool success, isolated output text including `PASS`, and unsupported reports of test success alone SHALL NOT count as acceptance proof. Acceptance conclusions SHALL require behavior-relevant checks with established scope and provenance.

#### Scenario: Deployment facts lack evidence
- **GIVEN** upstream documentation describes capture or retention but no deployment evidence verifies it
- **WHEN** the assessment records those properties
- **THEN** it marks them `unknown` or `unavailable` and does not present the documentation as deployed evidence

#### Scenario: Correlation cannot be verified
- **GIVEN** logs lack a verified session or event correlation field
- **WHEN** a diagnosis would attribute events to a session
- **THEN** the attribution is recorded as unknown or missing and no causal conclusion is drawn from it

#### Scenario: Success signals are not proof
- **GIVEN** an agent claims success, reports high confidence, a tool returns successfully, or output contains `PASS`
- **WHEN** evidence is assessed for acceptance
- **THEN** those signals alone do not establish acceptance or successful tests

### Requirement: Engine-specific offline regression baseline
The system SHALL define a reviewed synthetic baseline for each existing engine at its real existing tests: `FollowUpPolicy::decide` tests for the pure policy and TUI `schedule_auto_poke_followup_if_needed` app tests. The engines SHALL remain distinct. The baseline SHALL use those implementations and tests directly, not copied rules, a serialized replay framework, or an exact live-session replay claim. It SHALL cover repeated verification and legitimate required checks, changed evidence, stop, wait and queued behavior, stale or absent metadata, tool errors, and limits, recording truthful observed outcomes without promising semantic improvement. Cases SHALL be allocated to the boundary that actually consumes the relevant state or events; unsupported dimensions SHALL be documented, not fabricated as new engine inputs.

#### Scenario: Repeated verification and required checks
- **GIVEN** reviewed synthetic cases for unchanged repeated verification and a genuinely required check
- **WHEN** each engine's existing tests evaluate its cases
- **THEN** the baseline records both outcomes and does not treat reduced activity as improvement if required checks are suppressed

#### Scenario: Changed evidence and continuation states
- **GIVEN** reviewed synthetic cases with changed evidence and stop, wait, and queued states
- **WHEN** the pure policy and TUI app tests evaluate those cases separately
- **THEN** the baseline records each engine's observed behavior without asserting the engines are equivalent

#### Scenario: Missing metadata, errors, and limits
- **GIVEN** reviewed synthetic cases with stale or absent metadata, tool errors, or reached limits
- **WHEN** the existing tests evaluate them
- **THEN** the baseline records observed outcomes and uncertainty without inventing attribution, replay results, or guaranteed improvements

### Requirement: Behavioral noninterference
This evidence-only slice SHALL preserve the existing decisions, messages, ordering, counters, and read-versus-acknowledgment effects of the pure policy and TUI scheduler as distinct engines. Direct tests SHALL run without OmniRoute, live logs, or provider access. Production logs SHALL NOT become inputs to continuation decisions.

#### Scenario: Pure policy remains unchanged
- **GIVEN** a reviewed synthetic input evaluated by the existing `FollowUpPolicy::decide` tests
- **WHEN** evidence assessment or baseline checks run
- **THEN** the policy's existing decision and message behavior remain unchanged

#### Scenario: TUI scheduling remains unchanged
- **GIVEN** a reviewed synthetic input evaluated by the existing `schedule_auto_poke_followup_if_needed` app tests
- **WHEN** evidence assessment or baseline checks run
- **THEN** the TUI's existing decisions, ordering, counters, and read-versus-acknowledgment effects remain unchanged

#### Scenario: Offline test boundary
- **GIVEN** the direct pure-policy and TUI app tests
- **WHEN** they run
- **THEN** they require no OmniRoute, live production logs, or provider access, and production logs are not read as decision inputs

### Requirement: Demonstrated-gap logging only
Logging additions SHALL be conditional on a specific, reproducible missing evidence field or event that obstructs a target diagnosis after existing-log coverage has been assessed. If such a gap is demonstrated, any addition SHALL use existing native logging and include focused tests. If no specific gap is demonstrated, or evidence is inconclusive, the assessment SHALL record no logging addition or an inconclusive result. The assessment SHALL NOT presume that gateway deployment, capture, or retention changes are necessary.

#### Scenario: Reproducible evidence gap
- **GIVEN** an assessed target diagnosis and a reproducible missing field or event that blocks it
- **WHEN** an addition is justified
- **THEN** the addition is bounded to existing native logging and has focused tests for the needed evidence

#### Scenario: No demonstrated gap
- **GIVEN** existing logs support the target diagnosis, or the evidence does not identify a reproducible missing field or event
- **WHEN** the assessment concludes
- **THEN** it records no logging addition or an inconclusive result and does not presume a gateway change is required

### Requirement: Privacy without a new trace store
This capability SHALL NOT add a trace store, capture flag, raw-body capture, automatic export or import, or private session fixtures. Documentation and tests SHALL use reviewed synthetic data, never production prompts, tool arguments, tool output, credentials, or private transcript content. Synthetic expected messages MAY assert existing behavior without copying user-session content. The assessment SHALL NOT claim that existing gateway logs are metadata-only or private by default; it SHALL state what evidence supports any privacy characterization and leave unverified properties unknown.

#### Scenario: No private transcript material
- **GIVEN** documentation or tests are authored for the baseline or log assessment
- **WHEN** examples and cases are added
- **THEN** they use reviewed synthetic data and contain no private session fixtures, production prompts, tool arguments, tool output, credentials, or transcript content

#### Scenario: Existing gateway privacy is unverified
- **GIVEN** no deployed evidence establishes that gateway logs contain only metadata or are private by default
- **WHEN** the assessment describes those logs
- **THEN** it makes neither claim and records the relevant privacy properties as unknown
