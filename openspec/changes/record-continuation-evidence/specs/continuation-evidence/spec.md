# Continuation evidence delta

## Purpose

Observe native continuation metadata and replay complete sanitized fixtures without changing existing controller behavior or re-logging private text.

## ADDED Requirements

### Requirement: Versioned provenance
The system SHALL identify production metadata receipts as `ContinuationReceiptV1` version 1 and separate typed AGENT CLAIM metadata, NATIVE OBSERVATIONS and RUNTIME IDENTITY. Claims SHALL remain untrusted. Available safe native session/call/turn references and observed controller/daemon identity SHALL remain distinct; unavailable or stale provenance SHALL remain unknown. Untrusted identifiers SHALL use capture-local opaque tokens. Git HEAD or an on-disk binary SHALL NOT establish running-daemon identity. Tool success, output text or confidence SHALL NOT establish acceptance or satisfied tests.

#### Scenario: Available native provenance
- **WHEN** capture observes a native call and controller identity but no daemon identity
- **THEN** the metadata receipt retains available safe references, labels daemon identity unknown and separates claim metadata from observations

#### Scenario: Stale or legacy identity
- **WHEN** usable event identity is missing, stale or supplied only by Git HEAD or a disk binary
- **THEN** affected provenance remains unknown and no running-daemon attribution is invented

#### Scenario: Success is not acceptance
- **WHEN** a native tool returns no error, its output says `PASS`, or an agent reports high confidence
- **THEN** only typed return/claim metadata is recorded, with no copied payload or assertion that acceptance or tests passed

### Requirement: Exact engine inputs
Complete offline replay SHALL use a distinct `ContinuationReplayFixtureV1` containing reviewed synthetic input, the selected engine's exact decision-relevant before-state and fixed eligibility/clock context. The shared Rust policy and existing TUI scheduler SHALL be distinct engines, without serializing the whole TUI application or assuming equivalence. Production receipts SHALL contain metadata only, excluding raw todo/plan/goal text, serialized fingerprints and generated messages by construction. They SHALL NOT be automatically exported as complete fixtures; their exact replay coverage SHALL be incomplete with reason `redacted_input`.

#### Scenario: Stateful policy capture
- **WHEN** a complete sanitized fixture supplies the actual shared-policy input and before-state
- **THEN** replay can evaluate that engine's decision while its state/input remain separate from the production metadata type

#### Scenario: Scheduler no-op capture
- **WHEN** the actual TUI scheduler encounters disabled or pending/queued work
- **THEN** capture records typed no-follow-up/eligibility metadata without storing its private text or substituting the SDK controller

#### Scenario: Missing context
- **WHEN** replay receives production metadata, omitted input or missing decision-relevant context
- **THEN** it reports incomplete coverage rather than supplying guessed defaults or a successful replay

### Requirement: Noninterfering capture
Capture SHALL preserve existing decisions, in-memory messages, ordering, state transitions, caps and requested effects with tracing enabled or disabled. It SHALL NOT make a second decision, reset/re-arm a controller, send messages, acknowledge effects, clear observations or mark reviews delivered. Requested effects SHALL remain separate from observed application, defaulting to `not_observed` without confirmation from the existing native path. Capture failure SHALL NOT cause an agent turn, retry or changed decision.

#### Scenario: Capture parity
- **WHEN** matched native controller states are evaluated with capture off and on
- **THEN** selected follow-ups, in-memory messages, requested effects and acknowledgement counts are identical

#### Scenario: Effects without follow-up
- **WHEN** a controller requests effects but no follow-up message
- **THEN** the receipt preserves that distinction and capture itself does not apply or acknowledge those effects

#### Scenario: Capture failure isolation
- **WHEN** storage, serialization or privacy checks prevent capture
- **THEN** the original controller outcome remains unchanged and only a bounded redacted diagnostic is permitted

### Requirement: Private opt-in storage
`features.continuation_trace` SHALL default to false and disabled capture SHALL perform no trace I/O. Enabled production capture SHALL accept only metadata receipts, never complete fixtures or arbitrary JSON. It SHALL store no raw user-authored task/plan/goal text, generated messages, serialized fingerprints, tool outputs, command arguments, arbitrary error prose, credential payloads or full transcript. It SHALL use the safe-session path `<JCODE_HOME>/todos/continuation-traces/<safe-session-id>.json`, creating the new Unix directory/file with 0700/0600 without modifying existing directory permissions or widening ACLs. Unsafe paths, symlinks or unverified privacy SHALL make capture unavailable. Non-Unix capture SHALL remain explicitly unavailable until equivalent privacy is verified. No automatic export, upload, provider call or migration is permitted.

#### Scenario: Disabled capture
- **WHEN** the setting is missing or false
- **THEN** no trace directory/file is read or written and existing configuration behavior remains unchanged

#### Scenario: Private enabled trace
- **WHEN** Unix capture is explicitly enabled in an isolated private test home
- **THEN** only the metadata type is written under the new private subdirectory within its declared limits, with no raw sentinel payload text

#### Scenario: Unsafe storage boundary
- **WHEN** a session path is unsafe, a destination is a symlink, ownership/privacy is unavailable or equivalent platform privacy is unverified
- **THEN** capture writes nothing, changes no existing permissions and preserves the native decision

### Requirement: Bounded trace integrity
The whole serialized trace SHALL contain at most 64 records and 512 KiB. Reads and writes SHALL be bounded and replacement atomic. Retention SHALL record dropped/truncated coverage when evicting oldest complete records. Oversized records SHALL become bounded omission metadata or be skipped without implying complete history. Malformed existing data SHALL remain untouched; unsupported versions SHALL fail explicitly rather than being silently repaired or treated as current.

#### Scenario: Retention boundary
- **WHEN** a 65th record or byte-cap pressure is encountered
- **THEN** output stays within both caps and coverage explicitly records loss of earlier observations

#### Scenario: Oversized input
- **WHEN** even one record cannot fit safely
- **THEN** bounded omission metadata or an explicit skipped capture replaces a full entry, never a successful complete replay

#### Scenario: Corrupt or unknown trace
- **WHEN** an existing journal is malformed or its version is unsupported
- **THEN** the existing file is not repaired or overwritten, the problem is explicit and the native controller continues unchanged

### Requirement: Isolated engine replay
Complete fixture replay SHALL use the actual `FollowUpPolicy::decide` or the actual TUI scheduler under existing isolated test helpers, not copied rules. The pure SDK replayer SHALL return `unsupported_engine` for TUI records; a TUI test-only adapter SHALL verify that engine separately. Replay SHALL distinguish matched, mismatched, incomplete, unsupported version/engine and invalid results. It SHALL NOT execute tools, commands, providers or follow-up messages; apply recorded effects; open paths named by fixture content; import state into a live controller; or mutate original session data.

#### Scenario: Shared policy match
- **WHEN** a complete valid synthetic SDK-policy fixture is replayed
- **THEN** a fresh native policy is evaluated once and its actual decision/messages/requested effects are compared to the fixture expectation

#### Scenario: TUI adapter boundary
- **WHEN** a complete TUI fixture is presented to the pure public SDK and separately to the TUI test adapter
- **THEN** the SDK returns unsupported engine and the isolated TUI adapter calls the real scheduler without introducing a pure-crate dependency on TUI

#### Scenario: Untrusted replay input
- **WHEN** input is malformed, unsupported, incomplete, metadata-only or includes command/path-like text
- **THEN** replay reports its explicit non-success category without executing that text or touching original session state

### Requirement: Sanitized baseline coverage
The baseline SHALL include reviewed synthetic cases derived from observed failure patterns, not raw private transcripts or automatic snapshot exports. It SHALL cover repeated unchanged/metadata-only validation alongside genuinely required checks and changed state, queued/blocked work, cancellation, stale/unknown identity, interleaved tool failures, missing context, budget boundaries and malformed/version-mismatched inputs. Each engine SHALL be evaluated against its own baseline, with differences and limitations retained.

#### Scenario: Repetition and necessary check
- **WHEN** both unchanged repetition and a newly required check are present in matched fixtures
- **THEN** both native outcomes are recorded without equating less activity with improvement or suppressing the required-check control

#### Scenario: Waiting and cancellation
- **WHEN** work is pending, blocked, queued or cancelled
- **THEN** the baseline exercises actual engine eligibility and stop behavior with capture parity preserved

#### Scenario: Error and identity controls
- **WHEN** same/different-session native calls interleave or identity/error evidence is missing or stale
- **THEN** the fixture verifies explicit unknowns and correct attribution without promoting claims or tool success to acceptance

#### Scenario: Incomplete baseline controls
- **WHEN** fixture state/version is incomplete or a receipt contains only redacted metadata
- **THEN** the baseline expects an explicit incomplete/invalid/unsupported result rather than a fabricated match

### Requirement: Observation-only claims
This capability SHALL provide observations and a baseline only. It SHALL NOT claim to fix repeated pokes, merge the controllers, reduce turns/cost/latency, satisfy a user's goal or authorize P03/overnight implementation. Future improvement claims require matched behavioral evidence including required-check and authorization/stop negative controls, not a count or quieter output alone.

#### Scenario: Baseline without measurement
- **WHEN** receipts and fixtures have been implemented but no later comparison was run
- **THEN** reporting describes baseline coverage and limitations, not measured improvement or completion of P01's other outcomes

#### Scenario: Unmatched comparison
- **WHEN** future results use different inputs, omit required-check controls or provide no serving/provenance evidence
- **THEN** this baseline does not validate the improvement claim and the comparison remains explicitly unverified
