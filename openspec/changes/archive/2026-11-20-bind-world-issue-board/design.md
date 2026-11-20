# Design

## Context

The board combines a verified local policy boundary with remote tracker data and narrowly authorized writes. The live Linear check showed that one authenticated connection can see both Priceless and BrownandBrown teams. Therefore authentication identity is not a sufficient world boundary. The pinned source also shows MCP `workspaces` is a cwd-based server-start filter, not a record-level query filter (`crates/jcode-base/src/mcp/protocol.rs:238-245,264-279` at `dc0008591`).

`workspace-aware-context` owns verified world resolution and worker target policy. `add-config-composition` owns typed TOML composition and save semantics. This feature consumes those boundaries rather than defining a parallel resolver or configuration format.

## Goals and non-goals

Bind a board to a verified world and explicit stable tracker IDs; display that scope in a keyboard-operable TUI; restrict status writes to one confirmed state transition with expected-state protection and durable local evidence.

Do not implement tracker credential provisioning, a generic MCP proxy, cross-world aggregation, or writes to any fields other than issue state. Do not infer ADO availability, API semantics or retry guarantees from the Linear audit.

## Decisions

### 1. World policy gates backend selection

Resolve the active repository and world through the existing verified canonical-path policy immediately before querying and again before writing. Configuration supplies the explicit backend/team and optional project IDs, but cannot override the world rule: Brown plus Linear is rejected before provider dispatch. Unknown or ambiguous identity has no board. No global/last-used fallback is permitted.

A shared credential or MCP process may serve more than one permitted binding, but every request still receives an explicit bound scope and every response/cache entry remains associated with that scope. Server tool visibility is not treated as record authorization.

### 2. Provider boundary owns an allowlist

The board adapter exposes only the required read operations (workflow states and scoped issues) and one write operation (issue state update). Do not expose arbitrary MCP tools or translate a generic mutation request. Use provider workflow order and state identity rather than hardcoded Linear statuses. If the selected integration cannot provide a needed operation, show the unavailable/error state instead of switching providers or guessing.

The audit verified Linear visibility and tool categories, not a stable Linear mutation contract, ADO API, or either backend's idempotency guarantees. During implementation, inspect the actual integrations. A confirmed write may be retried only when the backend can safely enforce an operation identity or a live expected-prior-state recheck proves the update has not happened. If uncertainty remains and idempotency is unavailable, stop and report uncertainty rather than resend.

### 3. Confirmation binds one transition

Confirmation is per issue transition and is bound to the verified world, configured backend/team/project, issue ID, expected prior state and requested state. Revalidate the world at dispatch. If the issue no longer has the expected state, reject the transition. User confirmation authorizes only the state change; it does not authorize labels, assignees, comments or later changes.

### 4. Audit intent precedes remote effects

Append a local intent event before dispatch, then append an outcome event when known. Never rewrite earlier records. If the intent cannot be durably appended, do not write remotely. Keep records minimal: identifiers, states, timestamp, operation identity and outcome, without credentials or issue descriptions. Treat unknown outcomes as unknown, not as success or failure eligible for blind retry.

## Risks and trade-offs

- Provider workflow states vary. Showing provider-defined states avoids a fabricated cross-provider taxonomy but may create different columns across worlds.
- An append-only audit can contain sensitive issue identifiers and grows over time. Settled as decision 1 below: owner-only append-only file under the Jcode data directory, retained until an approved retention policy exists.
- If provider idempotency cannot be verified, uncertain writes reduce availability because Jcode must stop instead of retrying. This is preferable to duplicate or cross-world mutations.
- The current worktree contains a large in-progress merge. Implementation must remain isolated to its authorized scope and must not resolve unrelated conflicts.

## Settled decisions (user, 2026-11-19: `1a 2a 3a`)

1. **Audit storage and retention (1a).** Use a per-user local append-only file under the existing Jcode data directory, created with owner-only permissions, inspectable and exportable by the user. Retain all records until a separately approved retention policy exists; the board never rotates or deletes them.
2. **Board scope (2a).** A binding requires an explicit stable team ID; a project ID is optional. A team-wide board is labeled as team-wide. Never auto-select a project from visible results.
3. **ADO writes (3a).** Brown/ADO boards are read-only in v1. Status writes for ADO stay disabled until its conditional-update and safe-retry contract is pinned and verified; enabling them requires a follow-up change.
