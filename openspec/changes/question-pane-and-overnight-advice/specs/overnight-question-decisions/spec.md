## ADDED Requirements

### Requirement: Explicit bounded delegation
Automatic question answers SHALL require explicit per-run opt-in, user-defined delegation scope, a matching Running coordinator before target wake, compatible client capabilities, and policy-eligible reversible local decisions. Unknown risk and requests for consent, secrets, purchases, external communication, destructive changes, permissions or security/privacy/retention changes SHALL remain human-only. Jev probabilities SHALL NOT establish authorization.

#### Scenario: Ordinary or legacy run
- **WHEN** a question appears in an ordinary session or an old manifest without opt-in
- **THEN** it remains manual regardless of idle time

#### Scenario: Unsafe or ambiguous decision
- **WHEN** any question is outside delegation scope or requires human authorization
- **THEN** the batch remains pending with an explicit reason and existing action authorization is unchanged

### Requirement: Server-owned inactivity and resolution
Eligible unanswered single-choice batches SHALL submit after 300 seconds from current eligible advice availability or latest authenticated session user interaction, whichever is later. Every recommendation SHALL meet probability 0.90 and winner margin 0.20. Manual submit, cancellation and timeout SHALL resolve at most once.

#### Scenario: Eligible timeout
- **WHEN** every question qualifies and no human interaction occurs for 300 seconds
- **THEN** the server validates and submits the recommended IDs atomically with automatic provenance

#### Scenario: User interaction and partial answers
- **WHEN** the user navigates or edits the pending batch
- **THEN** navigation resets the idle timer and any selection or text edit makes that batch manual-only

#### Scenario: Ineligible batch
- **WHEN** a batch contains multi-select, an unknown option ID, abstention, low confidence or a partially answered question
- **THEN** no part of it is auto-submitted and the blocking reason is visible

#### Scenario: Concurrent completion
- **WHEN** a valid human submission or cancellation races the timer
- **THEN** a shared server arbitration path accepts only the first valid resolution and rejects duplicates

#### Scenario: Lifecycle and client changes
- **WHEN** the run stops, target wake arrives, automation is disabled or an incompatible client attaches
- **THEN** the timer disarms and does not synthesize an answer

#### Scenario: Disconnect and restart
- **WHEN** a capable client disconnects from an armed request
- **THEN** the server keeps its explicitly authorized deadline and replays current state on live reconnect, but daemon restart records interruption and never resurrects an overdue answer

### Requirement: Durable accountable automatic decisions
The server SHALL durably record automatic resolution provenance before releasing the answer. Tool results, transcript and deterministic completed/failed/cancelled run reviews SHALL distinguish automatic from human answers and surface each automatic decision plus blocked/interrupted questions.

#### Scenario: Decision report
- **WHEN** a run auto-answers a question and later completes, fails or is cancelled
- **THEN** its report includes count, question and selected label, timestamps, Jev confidence/provider, policy/reason and full-record reference

#### Scenario: Persistence failure
- **WHEN** the decision record cannot be durably written
- **THEN** no automatic answer is delivered and the request stays manual with an audit failure reason

#### Scenario: Recorded but undelivered
- **WHEN** the daemon fails after recording a resolution but before delivering it
- **THEN** recovery reports interruption rather than replaying the answer, and untrusted labels are escaped in rendered records
