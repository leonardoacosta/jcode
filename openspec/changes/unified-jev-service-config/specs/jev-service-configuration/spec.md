## ADDED Requirements

### Requirement: Shared Jev service resolution

The system SHALL resolve the provider, endpoint, credential, and effective model for typed Jev decisions through one shared configuration path used by the `evaluate` tool and Review screening. This requirement does not cover AutoReview/AutoJudge conversational model selection or the compaction Jev caller; both remain unchanged.

#### Scenario: OpenRouter default
- **WHEN** no provider is explicitly selected and an OpenRouter credential is available
- **THEN** the system sends typed Jev decisions to OpenRouter using `~typesafe/jev-latest` unless a compatible model override is configured

#### Scenario: TypeSafe compatibility fallback
- **WHEN** no provider is explicitly selected, no usable OpenRouter credential is available, and a TypeSafe credential is configured
- **THEN** the system uses the TypeSafe System One endpoint and `jev-latest`

#### Scenario: Explicit provider selection
- **WHEN** the user explicitly selects a provider
- **THEN** the system uses that provider's endpoint and credential only
- **AND** reports a sanitized missing-credential error rather than silently sending credentials to another provider

#### Scenario: Shared behavior across callers
- **WHEN** Evaluate and Review screening run with the same service configuration
- **THEN** both resolve to the same provider, endpoint, credential source, and effective model

### Requirement: Secret-safe configuration

The system SHALL keep provider credentials out of TOML and SHALL NOT expose credential values in logs, errors, status output, or serialized configuration.

#### Scenario: Environment credential resolution
- **WHEN** a supported environment credential is unset, empty, or whitespace-only
- **THEN** the resolver treats it as unavailable and applies the documented provider-selection behavior

#### Scenario: Missing credential diagnostic
- **WHEN** the selected provider has no usable credential
- **THEN** the system identifies the selected provider and expected credential variable without revealing any credential value

### Requirement: Safe screening degradation

Review screening SHALL NOT interpret a failed, unavailable, or malformed Jev response as a low-risk result.

#### Scenario: Screening service unavailable
- **WHEN** the Jev request fails or returns an invalid result
- **THEN** the affected item is routed to full LLM review

#### Scenario: Evaluation service unavailable
- **WHEN** the Evaluate tool cannot resolve or reach its configured Jev service
- **THEN** it returns a sanitized actionable error and does not fabricate a judgment

### Requirement: Review/Judge and compaction scope boundaries

This change SHALL leave AutoReview/AutoJudge chat provider/model selection and the compaction Jev caller unchanged.

#### Scenario: Chat settings unchanged
- **WHEN** the unified Jev service configuration is used by Evaluate or Review screening
- **THEN** AutoReview/AutoJudge provider and model settings, defaults, and session selection remain unchanged
- **AND** `~typesafe/jev-latest` is never used as their natural-language chat model

#### Scenario: Compaction caller unchanged
- **WHEN** the unified Jev service configuration is added to Evaluate and Review screening
- **THEN** the compaction Jev caller retains its existing endpoint, credentials, model, and fallback behavior
