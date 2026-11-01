## ADDED Requirements

### Requirement: Ordered fallback on transient errors
The agent SHALL, after retries are exhausted for a transient model error, retry the turn on the next model in the configured `model_fallback` chain for the current model.

#### Scenario: Rate limit triggers fallback
- **WHEN** the primary model returns HTTP 429 after retries and `model_fallback` lists model B
- **THEN** the turn is retried on model B

#### Scenario: No chain configured
- **WHEN** a transient error occurs and the current model has no `model_fallback` entry
- **THEN** no automatic switch occurs and the error is surfaced as today

### Requirement: Permanent errors never switch
The agent SHALL NOT switch models automatically for authentication, configuration, or invalid-request (4xx other than 408, 409, 429) errors.

#### Scenario: Auth failure
- **WHEN** the primary model returns HTTP 401
- **THEN** no fallback model is tried and the manual fallback offer is shown

### Requirement: Model cooldown and cycle cap
The agent SHALL treat a model with 3 or more failures within the last 60 seconds since its last success as cooling, SHALL try cooling models after healthy ones, and SHALL stop after 3 full passes over the chain.

#### Scenario: Cooling model deprioritized
- **WHEN** fallback model B failed 3 times in the last 60 seconds and model C is healthy
- **THEN** model C is tried before model B

#### Scenario: Chain exhausted
- **WHEN** every candidate fails for 3 full cycles
- **THEN** the original error is surfaced and no further switches occur

### Requirement: Visible switches
The agent SHALL record each automatic model switch and each skipped candidate as a transcript notice naming the source model, target model, and error class.

#### Scenario: Switch shown
- **WHEN** the agent switches from model A to model B after a 503
- **THEN** the transcript shows a notice containing `A`, `B`, and the overloaded/transient reason
