# clippy-clean-workspace Specification

## Purpose
Keeps the workspace free of clippy warnings under the CI command (`cargo clippy --workspace --all-targets --all-features -- -D warnings`) without lint suppression.

## Requirements

### Requirement: Workspace clippy clean
The workspace SHALL pass `cargo clippy --locked --all-targets --all-features -- -D warnings` with zero errors.

#### Scenario: Clean tree passes
- **WHEN** clippy runs on all targets with warnings denied
- **THEN** it exits 0 for every workspace crate

#### Scenario: New lint fails CI
- **WHEN** a change introduces any clippy warning
- **THEN** the CI clippy step fails and blocks merge

### Requirement: No sync lock held across await
Code in `jcode-app-core` SHALL NOT hold a `std::sync::MutexGuard` across an `.await`.

#### Scenario: Guard dropped before await
- **WHEN** `turn_failover.rs` reads shared state then awaits
- **THEN** the guard is released before the await point

#### Scenario: Regression detected
- **WHEN** a guard is held across an await
- **THEN** clippy `await_holding_lock` fails the build
