# Native Initiative Command Center Specification

## Purpose

Define the native Jcode authority and projection boundary for durable initiatives and the Command Center surfaces.

## ADDED Requirements

### Requirement: Native initiative authority is explicit

Jcode SHALL treat app-core goal persistence as the sole authority for initiative identity, status, milestones, steps, checkpoints, revisions, and idempotency state.

#### Scenario: A client reads an initiative

- **WHEN** the initiative tool, TUI, or Command Center requests an initiative
- **THEN** the request SHALL resolve through the native goal persistence authority
- **AND** no client-specific persistence store SHALL be consulted.

### Requirement: Command Center web UI is a projection

The Command Center web UI SHALL read and mutate initiatives through the native daemon/API contract and SHALL NOT directly read or write durable persistence files.

#### Scenario: A browser updates a milestone

- **WHEN** an authenticated browser submits a milestone update
- **THEN** the daemon SHALL apply revision, idempotency, authorization, and CSRF/origin checks
- **AND** the browser SHALL install the authoritative replacement snapshot returned by Jcode.

### Requirement: Native surfaces share lifecycle semantics

The initiative tool, TUI, daemon API, and web UI SHALL use the same status, milestone, step, checkpoint, revision, degraded, unavailable, and linked-run vocabulary.

#### Scenario: A capability is unavailable

- **WHEN** Orca, the scheduler, or the browser host is unavailable
- **THEN** the affected surface SHALL expose an explicit unavailable or degraded state
- **AND** it SHALL NOT report an unobserved operation as successful.

### Requirement: Extensions preserve authority boundaries

New initiative views and commands SHALL extend existing native DTOs, commands, projections, and security boundaries before introducing new storage or parallel lifecycle models.

#### Scenario: A new browser view is proposed

- **WHEN** a contributor adds a new initiative view
- **THEN** the implementation SHALL identify the existing native read and mutation seams
- **AND** it SHALL document why an existing seam cannot satisfy the requirement before adding a new one.
