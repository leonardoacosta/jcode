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
