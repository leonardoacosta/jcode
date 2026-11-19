# Tasks

All tasks are implementation work and remain unchecked during proposal authoring. Execute in order. Treat `tasks.md` as the only execution ledger. Recheck source revision, current world-policy ownership, provider API contracts, configuration implementation status and execution authority before Apply. Do not query or mutate live tracker data during verification.

## 1. Binding and provider boundary

- [ ] 1.1 Inventory the current workspace resolver, canonical-path policy, config-composition implementation, Linear/ADO integrations and provider workflow-state APIs. Record verified source paths and identify which contracts are absent; do not infer an ADO adapter or idempotency support.
- [ ] 1.2 Add typed per-world backend/team bindings with optional project narrowing through the selected existing config owner. Preserve composition/provenance and do not add an implicit global or last-used fallback. Test canonical paths, symlinks, worktrees, unknown/ambiguous worlds, invalid config and Brown-to-Linear refusal.
- [ ] 1.3 Implement allowlisted scoped reads for workflow states and issues. Enforce provider, stable team and optional project identifiers at the adapter boundary. Add cache-key isolation tests for world, auth identity, backend, team and project, plus no-fallback tests for provider and authorization failures.

## 2. TUI Kanban

- [ ] 2.1 Add a board entry point and view over the selected workspace binding. Render columns in provider workflow-state order; implement keyboard movement among columns/issues and Escape back to the prior view.
- [ ] 2.2 Add explicit loading, empty, provider error and unauthorized states. Truncate compact chrome and issue text by Unicode display-cell width; test narrow terminal sizes, long/multicell text and one-row chrome.
- [ ] 2.3 Verify the view with focused `cargo test -p <crate> <board_test_name>` tests after identifying actual crate/test names; cover scoped data, cache separation, keyboard paths and all view states.

## 3. Status-change contract and audit

- [ ] 3.1 Implement a Rust-side allowlist for only issue-state updates, enabled for Linear bindings only; ADO bindings stay read-only and refuse writes before confirmation. Require a separate explicit confirmation per change, and bind confirmation to issue, verified world, backend/team/project, expected prior state and requested state.
- [ ] 3.2 Enforce expected-prior-state comparison before dispatch and retry. Use backend idempotency only where its contract is verified; otherwise refuse uncertain automatic retry. Re-resolve canonical path/world immediately before write. Verify labels, assignees and comments cannot be written through this surface.
- [ ] 3.3 Implement append-only local audit intent before dispatch and append-only outcome events. Prevent dispatch on audit failure. Test crash/uncertain response, duplicate retry, stale state, changed world, audit I/O failure and absence of credentials/content in records using fake provider and isolated local storage.

## 4. Integrated verification

- [ ] 4.1 Run focused board tests for each affected crate, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check` and `openspec validate bind-world-issue-board --strict`. Do not compile against live tracker credentials or mutate tracker data.
- [ ] 4.2 Verify all scenarios, including Personal/Priceless Linear allow, Brown Linear refusal, unknown world no query/fallback, ADO unavailable/unauthorized, state conflict, uncertain retry and append-only audit. Use fake adapters and test-only storage.
- [ ] 4.3 Obtain independent review of the final diff after fresh gates. Repair actionable findings and rerun affected gates and review. Record exact final-diff identity and checks before any implementation commit or archive.

## Dependencies and verification constraints

`workspace-aware-context` owns existing workspace detection and verified target propagation. `add-config-composition` owns existing TOML composition behavior. Reuse those implementations; do not duplicate them. Their actual landed/implemented state must be checked at execution because the current checkout contains in-progress changes. If a required contract is absent or not verified, update the implementation plan while retaining fail-closed behavior. Tests MUST use fake provider transports and isolated local storage; no live Linear or ADO write is part of acceptance. The current repository is a conflicted working tree, so do not resolve conflicts or perform broad Git operations as part of these tasks.
