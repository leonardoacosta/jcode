## ADDED Requirements

### Requirement: Custom automatic model fallback is retired
The fork SHALL NOT configure or execute the custom automatic `model_fallback` chain, including its automatic model substitution, cooling-order state, and chain-cycle behavior, once the approved upstream v1.0.1 reconciliation removes that fork-only feature. The retirement SHALL be explicit in source/tests and SHALL NOT use a supersession disposition or skipped spec synchronization as a substitute. Shared provider failure classification, auth/quota safeguards, selected route/account identity, and supported user-directed recovery remain authoritative.

#### Scenario: Transient model error after retirement
- **WHEN** the selected model encounters a transient error and the old custom `model_fallback` configuration would have listed another model
- **THEN** the retired custom chain does not automatically switch models and the normal upstream error/recovery behavior is used

#### Scenario: Confirmed quota or billing exhaustion
- **WHEN** shared classification identifies a confirmed quota or billing failure, including a 429 response
- **THEN** it is not treated as a transient signal for automatic model, provider, account, or gateway substitution

#### Scenario: Authentication or refresh transport failure
- **WHEN** authentication fails or refresh transport is unavailable
- **THEN** no model fallback is attempted, revocation is not inferred from transport failure alone, and the existing auth remedy/safeguard is preserved

#### Scenario: Supported user-directed model selection
- **WHEN** the user explicitly selects another model through an upstream-supported control
- **THEN** the explicit selection may take effect without restoring the retired automatic chain
