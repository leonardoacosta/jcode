# overnight-admission Specification

## Purpose
Overnight coordinator turns are admitted only on recorded evidence (task card change, worker signal, schedule), never on timers or bus overflow alone, and a run blocks fail-closed when route authority or task policy is unknown.

## Requirements

### Requirement: Evidence-gated coordinator turns

The overnight supervisor SHALL start a coordinator turn only after recording an `admitted` decision citing an eligible task or new signal.

Every signal SHALL trigger deterministic revalidation of original authorization, current strict task/worktree policy, dependencies, owner/generation, cancellation and finite remaining limits, including required authoritative monetary reservations. A signal or usable credential alone SHALL NOT grant dispatch. Unsupported chargeable providers SHALL remain blocked.

#### Scenario: Eligible work admitted

- **WHEN** a background task completes during a run
- **THEN** the supervisor SHALL revalidate deterministic admission and only when eligible record `admitted` citing that completion and start one turn, otherwise record waiting/blocked without dispatch

#### Scenario: No eligible work

- **WHEN** the previous turn reported no remaining eligible work and no new signal arrived
- **THEN** the supervisor SHALL record `waiting` and SHALL NOT start another turn before a new signal or the wake boundary

### Requirement: Auth-blocked preflight

The run SHALL NOT start coordinator turns when the explicitly selected permitted provider route lacks usable auth. It SHALL NOT probe unrelated configured accounts or silently switch routes to find one that works.

#### Scenario: All auth expired

- **WHEN** the explicitly selected permitted route reports missing or expired auth at preflight
- **THEN** the run SHALL record `blocked` with the provider reasons and end without a coordinator turn

#### Scenario: One provider usable

- **WHEN** the selected permitted provider authenticates
- **THEN** the run SHALL proceed to deterministic admission checks without inferring work or budget authorization
