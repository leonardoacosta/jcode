# Execution evidence

Approved execution: 2026-09-30. Baseline: 141fb08c3, recovery ref: recovery/upstream-v0.89.3-20260930. Target: de65ade33d514b31a43885318179b3622f321170. Added authoritative upstream remote. Kept main tracking origin/main. Merge started without commit; 14 conflicts identified.

Ownership: coordinator owns lockfile, telemetry deletion, base auth/browser fixtures and module declarations. Dedicated workers own Jev/config, MCP, and lifecycle/bash/tool-tests/CLI conflicts. No competing sessions were active at start.

Telemetry policy: retain deliberate local TELEMETRY.md deletion from 4c40615ee; do not reactivate removed local telemetry documentation. Auth fixture incorporates upstream broader environment cleanup. Browser timeout fixture retains local shared environment lock to preserve parallel isolation. Both workflow modules retained.

Lockfile: start from upstream pins, run cargo update --workspace --offline for merged local dependencies, rather than updating all dependencies.

Validation and integration commit pending.

## Final validation

Coordinated final regression job 2664063svc passed: base 1730 tests (5 ignored), core 1548 tests (13 ignored) plus one subprocess isolation test, root library 279 and main binary 8 tests. Formatting and whitespace checks passed. Final TUI build 440457rffa passed. Fresh isolated JCODE_RUNTIME_DIR daemon answered `debug sessions` with `[]`, without inference. Built version: v0.89.3-dev. Shared daemon remains the 172691406 build, config.toml and mcp.json timestamps unchanged. No claim of a paid/live provider evaluation or independent other-session MCP invocation.

Compatibility resolutions preserve local System One route configuration and credentials, explicit subscription selection, MCP HTTP transport, question channel dispatch/cancellation, terminal environment propagation, icon-free terminal titles, and environment-isolated tests. Incorporate upstream Jev purpose budgets/voice clients, MCP EOF fail-fast handling and new applet schema. Long schema guidance lives in comments rather than invented inputs.

Known limitations: live credential-dependent tests remain ignored. Build emits existing dead-code warnings. The oversized-test ratchet fails on imported upstream test growth against the older local baseline; do not silently reset its budget or refactor unrelated tests. Full all-features CI/clippy not run. No push or shared-daemon deployment.

## Future update procedure

Start with a clean, coordinated main. `git fetch --no-tags upstream master refs/tags/<release>` then independently verify the selected release commit. Create a unique recovery branch at HEAD. `git merge --no-commit --no-ff <verified-release-commit>`, resolve compatibility, run relevant regression/build and isolated-runtime checks, then commit and verify both baseline and release ancestry. Keep main tracking origin/main. Do not use FETCH_HEAD as the release target when fetching multiple refs. Push and deployment require separate authorization.

## Post-commit requirement traceability

Committed source f73abfb6a was re-tested in coordinated job 6472722k23: all base/core/root regression gates passed again.

| Requirement or output | Concrete observation | Result |
| --- | --- | --- |
| Preserve both ancestries | merge-base --is-ancestor for pinned release and recovery ref | Both pass; release-to-HEAD missing commits = 0 |
| Repeatable upstream alignment | upstream URL and main tracking queried after commit | Authoritative upstream configured, origin/main unchanged |
| Local routing and credential preservation | systemone_custom_url_binds_only_to_matching_provider; selected-key/no-fallback/subscription tests | All pass |
| MCP compatibility and error paths | streamable_http_initialize_list_and_call_roundtrip; redirect/error/SSE bounds; connect_fails_fast_when_server_exits_before_initialize | All pass |
| Local question public workflow | question_socket_protocol_multi_question_success_and_closed_rejection; cancel/socket/capability tests | All pass |
| Terminal environment propagation | concurrent_client_terminal_environments_remain_isolated; hook_process_replaces_daemon_terminal_env_with_client_snapshot | Both pass |
| Source updater behavior | source_update_check_real_git_upstream_states; invalid checkout and status mapping tests | All pass |
| Provider-sendable applet/tool outputs | tool_schemas_are_sendable_to_every_provider_dialect; description caps; documented_shapes_validate | All pass |
| Actual TUI build and daemon startup | build 440457rffa and fresh isolated runtime debug sessions | Build passes, daemon returns empty session list |
| Shared daemon/config unchanged | shared-server symlink and config file mtimes checked | Old binary retained and mtimes unchanged; independent other-session MCP invocation not tested |

These observations establish the requested source-history improvement and representative compatibility, not all possible upstream behavior. Live inference, all-features CI/clippy, oversized-test ratchet remediation, and production deployment remain outside the completed validation.
