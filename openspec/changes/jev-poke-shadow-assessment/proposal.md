---
execution:
  version: 1
  depends_on: [system-one-service]
---

## Why

Poke currently reasons primarily from todo records and self-reported completion confidence. Identical todos can hide actual progress, while edited todos can hide repeated failures. A separate evidence-based judgment through the System One service could distinguish useful continuation, missing verification, loops, and human blockers. Its quality is not established, so the first feature must measure recommendations without giving them control.

Source exploration: `research/jev-assisted-poke-exploration-2026-09-24.md`.

## What Changes

- Add explicitly enabled, session-scoped shadow assessment for TUI poke. Shadow means observing and recommending without changing continuation behavior.
- Add `/poke shadow on`, `/poke shadow off`, and `/poke shadow status`. Default is off, including after restart or reconnect. Existing `/poke` behavior remains unchanged.
- Use the System One service trait from `crates/jcode-system-one` (delivered by the `system-one-service` change). No MCP subprocess, no external binary, no new credential or configuration paths.
- Capture bounded, minimized evidence and make asynchronous, deadline-limited assessments. Expose only the most recent structured result through status.
- Provide labeled fixture replay and public-interface acceptance tests before claiming readiness.

## Capabilities

### New Capabilities

- `poke-shadow-assessment`: opt-in observation, typed recommendations, safe lifecycle handling, and read-only status.

### Modified Capabilities

None. Existing poke policy is preserved. This change adds a consumer of the System One service without altering its trait or provider selection contract.

## Actors and User Actions

The TUI user enables shadow mode after reviewing its disclosure. The TUI requests observations; app-core's System One service performs hosted evaluation. Users must provision a supported credential privately if none exists. Merely having credentials or auto-poke enabled does not consent to shadow evaluation.

The enable command must explain that bounded request/todo text and recent sanitized outcomes may be sent to the selected provider and may incur charges. Help documents this before use. Executing the documented command is explicit opt-in. Status identifies the provider without credentials. No configuration file or credential is modified by feature authoring or by the command.

## Scope and Non-scope

In scope: local and daemon-connected TUI sessions, session-scoped consent, asynchronous assessments, read-only status, deterministic fixture replay, and provider contract tests.

Out of scope: acting on recommendations, declaring completion, changing todo confidence, new MCP installation/registration, Foreman changes, overnight policy, headless poke, persisted shadow history, automatic enabling, and claiming model accuracy from mocks.

## Dependencies and Migration

Hard prerequisite: `system-one-service` must be implemented and its crate plus trait contract verified first. Its tasks are currently incomplete; implementation must precede this change.

No migration of existing settings or todos. Shadow consent and results are not restored across restart/reconnect. Multiple attached clients share one server-side assessment lease per session, but existing TUI continuation ownership is unchanged. Approval is required before implementation.

## Impact and Acceptance Evidence

Touched surfaces: `crates/jcode-tui/src/tui/app/commands.rs`, `input.rs`, `remote.rs`, `app.rs`, `crates/jcode-protocol/src/wire.rs`, app-core server dispatch, the System One service, and focused TUI/protocol tests. Confirm exact module paths during implementation because this tree is changing.

Acceptance requires commands exercised through a real TUI tester on an isolated socket, remote request/result handling, exact existing-poke queue parity with shadow on/off, and provider boundary tests. Hosted model-quality evaluation remains a separately consented experiment. See `tasks.md` and `verification.md`.
