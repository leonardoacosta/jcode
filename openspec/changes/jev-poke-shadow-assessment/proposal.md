---
execution:
  version: 1
  depends_on: [unified-jev-service-config]
---

## Why

Poke currently reasons primarily from todo records and self-reported completion confidence. Identical todos can hide actual progress, while edited todos can hide repeated failures. A separate evidence-based judgment could distinguish useful continuation, missing verification, loops, and human blockers. Its quality is not established, so the first feature must measure recommendations without giving them control.

Source exploration: `research/jev-assisted-poke-exploration-2026-09-24.md`.

## What Changes

- Add explicitly enabled, session-scoped shadow assessment for TUI poke. Shadow means observing and recommending without changing continuation behavior.
- Add `/poke shadow on`, `/poke shadow off`, and `/poke shadow status`. Default is off, including after restart or reconnect. Existing `/poke` behavior remains unchanged.
- Use the native Jev service through the resolver delivered by `unified-jev-service-config`. MCP transport is not required by this proposed scope. Approval of this proposal accepts that interpretation of “jev-mcp”. If MCP is mandatory, revise before approval rather than implementing both transports.
- Capture bounded, minimized evidence and make asynchronous, deadline-limited assessments. Expose only the most recent structured result through status, not a new persistent event collection system.
- Provide labeled fixture replay and public-interface acceptance tests before claiming readiness.

## Capabilities

### New Capabilities

- `poke-shadow-assessment`: opt-in observation, typed recommendations, safe lifecycle handling, and read-only status.

### Modified Capabilities

None. Existing poke policy is preserved. This change adds a consumer of the shared Jev resolver without altering its provider selection contract.

## Actors and User Actions

The TUI user enables shadow mode after reviewing its disclosure. The TUI requests observations, and the app-core Jev service performs hosted evaluation. Users must provision a supported credential privately if none exists. Merely having credentials or auto-poke enabled does not consent to shadow evaluation.

The enable command must explain that bounded request/todo text and recent sanitized outcomes may be sent to the selected provider and may incur charges. Help documents this before use. Executing the documented command is explicit opt-in. Status identifies the provider without credentials. No configuration file or credential is modified by feature authoring or by the command.

## Scope and Non-scope

In scope: local and daemon-connected TUI sessions, session-scoped consent, asynchronous assessments, read-only status, deterministic fixture replay, and provider contract tests.

Out of scope: acting on recommendations, declaring completion, changing todo confidence, new MCP installation/registration, Foreman changes, overnight policy, headless poke, persisted shadow history, automatic enabling, general evaluator cleanup, and claiming model accuracy from mocks.

## Dependencies and Migration

Hard prerequisite: `unified-jev-service-config` must be implemented and its resolver contract verified first. Its current tasks are incomplete. This proposal does not change that proposal's scope or files. Its missing execution metadata must be resolved by its owner before dependency-graph admission can pass.

No migration of existing settings or todos. Shadow consent and results are not restored across restart/reconnect. Multiple attached clients share one server-side assessment lease per session, but existing TUI continuation ownership is unchanged. Approval is required before implementation.

## Impact and Acceptance Evidence

Likely surfaces: `crates/jcode-tui/src/tui/app/commands.rs`, `input.rs`, `remote.rs`, `app.rs`, `crates/jcode-protocol/src/wire.rs`, app-core server dispatch, the shared Jev service, and focused TUI/protocol tests. Confirm exact module paths during implementation because this tree is changing.

Acceptance requires commands exercised through a real TUI tester on an isolated socket, remote request/result handling, exact existing-poke queue parity with shadow on/off, and provider boundary tests. Hosted model-quality evaluation remains a separately consented experiment, not a prerequisite for a shadow-only feature's mechanical correctness. See `tasks.md` and `verification.md`.
