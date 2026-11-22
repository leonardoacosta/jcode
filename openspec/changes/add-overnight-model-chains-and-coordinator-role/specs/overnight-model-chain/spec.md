## ADDED Requirements

### Requirement: Per-role overnight model chains

An overnight run SHALL use an ordered coordinator model chain for coordinator turns and an ordered worker model chain for workers that the run spawns. Defaults SHALL be `claude-opus-5.5, gpt-6-sol, claude-sonnet-5.5, gpt-6-luna, DeepSeek-V4.1-Flash, qwen3.5-35b` (coordinator) and `claude-sonnet-5.5, gpt-6-luna, DeepSeek-V4.1-Flash, qwen3.5-35b` (worker). `[overnight] coordinator_model_chain` and `[overnight] worker_model_chain` SHALL override them. Every chain step SHALL stay on the session's selected provider route and account.

#### Scenario: Coordinator starts on the first served model
- **WHEN** an overnight run starts and the route serves `claude-opus-5.5`
- **THEN** the first coordinator turn uses `claude-opus-5.5`

#### Scenario: Worker starts on the worker chain
- **WHEN** the coordinator spawns a worker without an explicit model
- **THEN** the worker uses the first served model of the worker chain

#### Scenario: Explicit worker model wins
- **WHEN** the coordinator spawns a worker with an explicit model
- **THEN** that model is used first, and the worker chain applies after it fails

### Requirement: Chain advance on recoverable model failures

When a turn fails and shared provider classification returns `Transient`, `RateLimited`, `ModelUnavailable` or `QuotaExhausted`, the run SHALL switch to the next chain model on the same route, record a `model_chain_advanced` event (from, to, failure kind), and retry the turn. It SHALL NOT advance on `AuthExpired`, `AuthRevoked` or `Permanent`.

#### Scenario: Model unavailable
- **WHEN** a coordinator turn on `claude-opus-5.5` fails with `ModelUnavailable`
- **THEN** the run records `model_chain_advanced` and retries the turn on `gpt-6-sol`

#### Scenario: Auth failure does not advance
- **WHEN** a coordinator turn fails with `AuthRevoked`
- **THEN** the model does not change and the run ends `blocked` with the auth reason

#### Scenario: Provider never changes
- **WHEN** the chain advances
- **THEN** the provider route, account and gateway are the same as before the advance

### Requirement: Chain exhaustion

When the last chain model fails with a recoverable failure, the chain SHALL be exhausted. An exhausted coordinator chain SHALL end the run `blocked` with a reason listing every model tried. An exhausted worker chain SHALL fail only that worker and SHALL notify the coordinator; the run SHALL continue.

#### Scenario: Coordinator chain exhausted
- **WHEN** every coordinator chain model fails with recoverable failures
- **THEN** the run ends `blocked` and the reason names all six models

#### Scenario: Worker chain exhausted
- **WHEN** every worker chain model fails for one worker
- **THEN** that worker fails, the coordinator receives a notice, and the run stays running

### Requirement: Chain scope

The chains SHALL apply only to sessions owned by an active overnight run. Other sessions SHALL keep the retired-fallback behavior and SHALL NOT switch models automatically.

#### Scenario: Normal session unaffected
- **WHEN** a normal session's model fails with `ModelUnavailable`
- **THEN** no automatic model switch happens
