## ADDED Requirements

### Requirement: Explicit session consent

The TUI SHALL provide `/poke shadow on`, `/poke shadow off`, and `/poke shadow status`, with shadow mode disabled by default. Help SHALL disclose the selected-provider data flow, bounded task text, and possible cost. Enabling SHALL show that disclosure and SHALL NOT modify credentials or persisted configuration.

#### Scenario: Default remains private
- **WHEN** a session starts with credentials and auto-poke enabled but no shadow opt-in
- **THEN** no shadow assessment request is sent

#### Scenario: User enables and inspects shadow mode
- **WHEN** the user executes `/poke shadow on` and `/poke shadow status`
- **THEN** the TUI displays the disclosure, enabled state, selected provider/model, and remaining request budget without revealing credentials

#### Scenario: Invalid command
- **WHEN** the user enters `/poke shadow invalid`
- **THEN** the TUI displays usage and does not enable shadow mode or alter existing poke settings

### Requirement: Non-authoritative recommendations

Shadow assessments SHALL NOT enqueue, suppress, delay, or modify existing poke continuations, todo fields, confidence, refusal handling, or overnight behavior. Ineligible turns SHALL skip inference with a fixed reason.

#### Scenario: Recommendation disagrees with poke
- **WHEN** shadow mode returns any recommendation for a turn
- **THEN** the continuation queue and todo state match the same turn with shadow mode disabled

#### Scenario: Known wait or excluded turn
- **WHEN** a turn has no todos, active background work, pending user input, a permission wait, a guardrail stop, disabled auto-poke, or overnight execution
- **THEN** shadow mode sends no request and reports the applicable skip reason

### Requirement: Bounded evidence

Assessment input SHALL use only the design's allowlisted fields, pass all permitted text through secret redaction, and fit within an 8 KiB UTF-8-safe serialized payload. The system SHALL NOT expose raw payloads, credentials, or provider response bodies through status or diagnostics.

#### Scenario: Sanitized boundary
- **WHEN** request text or tool summaries contain a known credential sentinel
- **THEN** the provider request and displayed diagnostics omit the sentinel and omit raw arguments, output, environment values, and file bodies

#### Scenario: Evidence cannot fit safely
- **WHEN** required evidence exceeds the payload cap after safe minimization
- **THEN** the system skips inference with an evidence-limit reason instead of sending an oversized or malformed payload

### Requirement: Strict typed assessment

The service SHALL reuse the prerequisite Jev resolver and return the same typed result on cache hits and misses. It SHALL accept only the five design labels and valid distributions. It SHALL abstain on invalid data, provider errors, missing credentials, or uncertain predictions under the documented 0.80 top-probability and 0.20 margin rules.

#### Scenario: Valid recommendation
- **WHEN** a valid response selects verify with probability 0.90 and all other probabilities total 0.10
- **THEN** status shows verify and preserves provider confidence separately without presenting it as proof of correctness

#### Scenario: Uncertain or malformed response
- **WHEN** the answer is missing, has an unknown label, an invalid distribution, or fails either display threshold
- **THEN** status shows an abstention with a fixed reason and poke remains unchanged

#### Scenario: Provider and cache consistency
- **WHEN** identical evidence is evaluated on a cache miss and a hit under either supported provider configuration
- **THEN** the typed result has the same shape and uses the resolver's correct endpoint/model/credential policy without cross-provider credential fallback

#### Scenario: Service unavailable
- **WHEN** credentials are absent or the provider returns an error
- **THEN** the request abstains with a sanitized reason and existing continuation completes independently

### Requirement: Bounded asynchronous lifecycle

The system SHALL allow at most one in-flight assessment and one result per evidence revision per session, at most 20 launched requests per session, no automatic retries, and a two-second overall deadline. Connected TUI clients SHALL share one daemon-owned assessment lease. Cancellation SHALL invalidate the consent generation. Status SHALL retain only the latest result in memory.

#### Scenario: Slow provider or exhausted budget
- **WHEN** the provider exceeds two seconds or 20 requests have already launched
- **THEN** the system returns deadline or budget abstention without blocking the TUI, retrying, or delaying poke

#### Scenario: Duplicate clients and duplicate evidence
- **WHEN** two attached clients request the same session assessment or one client repeats an evidence revision
- **THEN** at most one provider request launches and only the lease owner receives its result

#### Scenario: Stale result
- **WHEN** input, cancellation, session switch, disconnect, `/poke off`, or `/poke shadow off` occurs before a result arrives
- **THEN** the old generation is invalidated and its result is not shown or acted on

#### Scenario: Restart and re-enable
- **WHEN** the TUI restarts or reconnects
- **THEN** shadow mode starts disabled and the previous result is not restored
- **AND** toggling shadow off and on without ending the server session does not reset that session's request budget
