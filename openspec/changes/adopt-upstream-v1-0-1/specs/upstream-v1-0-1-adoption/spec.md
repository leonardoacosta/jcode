## ADDED Requirements

### Requirement: Upstream history and fork behavior are reconciled without history loss
The integration SHALL merge the full approved upstream v1.0.1 history into the existing `main` history without squash, rebase, reset, force, or replacement of fork commits. It SHALL preserve nonconflicting fork features and attribution, explicitly retire behavior superseded by upstream, and record unresolved acceptance gates without claiming them complete.

#### Scenario: Full history merge
- **WHEN** the approved upstream v1.0.1 history is integrated
- **THEN** the resulting merge retains both the fork and upstream ancestry on the existing `main` checkout

#### Scenario: Conflicting fork behavior
- **WHEN** upstream behavior overlaps a custom fork behavior
- **THEN** the integration compares implementation and tests, applies the approved adopt/retire/preserve decision, and retains unrelated fork behavior

### Requirement: RC promotion uses the exact isolated green artifact
The integration SHALL identify each release candidate (RC) immutably and SHALL test the exact artifact in an isolated runtime before promotion. Promotion SHALL occur only when all required gates for that promotion are green and independent review covers the final candidate. Any repair SHALL create a new RC identity and require affected gates to be rerun. The shared daemon SHALL NOT be used as the candidate test environment.

#### Scenario: Failed or unavailable RC gate
- **WHEN** a required test, isolated runtime check, or independent review fails or is unavailable
- **THEN** the candidate is not promoted and the missing or failed gate remains explicit

#### Scenario: Candidate changed after test
- **WHEN** candidate bytes change after isolated testing
- **THEN** the changed artifact receives a new identity and is not represented as the previously tested RC

### Requirement: Retired fallback preserves provider safety
The integration SHALL remove the custom automatic model-fallback chain where upstream supersedes it, including configuration and automatic substitution behavior. It SHALL preserve shared auth/quota safeguards, manual recovery where supported, selected-route and account identity, and no-substitution rules. Explicit retirement SHALL be performed in the owning change and SHALL NOT be replaced by a disposition or skip-specs shortcut.

#### Scenario: Confirmed quota or billing failure
- **WHEN** provider classification confirms quota or billing exhaustion
- **THEN** the failure is not treated as a transient signal to substitute a model, provider, gateway, or account

### Requirement: Remote clipboard routing is explicit and client-safe
Remote clipboard writes SHALL use OSC 52 only and SHALL NOT invoke native host clipboard tools, unless `JCODE_CLIPBOARD=native` explicitly selects the native path. Remote links, `/links`, and `/mouse` behavior SHALL remain available.

#### Scenario: Remote session with host clipboard available
- **WHEN** the user copies in a remote session without the native override
- **THEN** the TUI sends OSC 52 and does not write through the remote host's native clipboard

#### Scenario: Explicit native override
- **WHEN** `JCODE_CLIPBOARD=native` is set
- **THEN** the configured native clipboard path is selected explicitly
