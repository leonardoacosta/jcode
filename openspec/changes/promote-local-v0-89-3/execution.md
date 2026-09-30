# Promotion validation

Candidate snapshot: a8de30eb9 plus verified prompt-fixture repair commit 290744e84.

Root cause of repeated ineffective repairs: inherited cargo shell function invokes main scripts/dev_cargo.sh, which changes cwd from nested scratch worktree back to main. Previous scratch commands tested main, not edited scratch files. Removed speculative owned Jev/mock-route edits and restored main socket fixture. Using scratch's own scripts/dev_cargo.sh compiled the actual candidate.

Actual candidate full base/core/root suite passed in task 9814374a9b. Formatting passed. Minimal prompt fixture isolates JCODE_HOME, preserving actual fallback behavior. Exact snapshot build passed task 225500qisl.

Real provider: automatic provider selection failed Cerebras 401. Explicit configured 9router/gpt-6-luna candidate run passed, exact output JCODE_0893_OK (326239gg2o).

Real MCP: candidate run attempted discovery/read-only call, returned no MCP tools discovered and no server call succeeded (360806oejk). Configured shared mcp.json contains four servers. Required MCP acceptance has NOT passed.

Pending shared activation repeatedly contains test-reload-hash with changing session owners, most recently session_llama_1790796128960_f0272bf2d63e2e08. Ownership not safely resolved.

PROMOTION BLOCKED. No shared daemon reload performed. Keep rollback immutable binary 172691406-dirty-07c9d9104c61. Scratch rollback copy SHA256 6c3889733946ab98891f7c2a26c2279974e47b2870880ea80883ef73f863d42c. Selfdev builds may update current/launcher candidate channel before live acceptance; shared daemon still old. Do not claim both channels untouched or promotion complete.

## Successful completion

MCP retry with management tool exposed passed: task 484983whr8, session_zebra_1790796485237_5169f01baa61a469 recorded successful mcp, mcp_search and mcp_call results, Graft read-only query returned data. Initial tools-only discovery was insufficient, not a demonstrated transport failure.

Verified repair fast-forwarded main to 290744e84. Clean coordinated build 6177455yws passed after moving generated client_sessions files into preserved scratch storage. Supported selfdev reload succeeded, same coordinator session resumed, PID 454202 reported SocketReady. Current/shared-server/launcher all resolve builds/versions/290744e84/jcode. Active version v0.89.3-dev. Previous immutable v0.88 binary remains executable for rollback.

Post-promotion MCP discovery and real read-only Graft call succeeded in resumed session. Graft itself warned graph refresh skipped due maximum call stack size and returned a ranked result; MCP transport succeeded, graph indexing has a separate limitation. config.toml unchanged. mcp.json mtime differs from initial preflight, so unchanged MCP file bytes cannot be claimed without prior hash. No deliberate credential edits were made.

Remaining limitations: automatic provider selection chose invalid Cerebras credential; explicit configured 9router passed. Full all-features/clippy gates not run; oversized-test baseline previously failed. No remote push. All other-session continuity not independently observed; coordinator continuity verified.


## Post-promotion audit correction

Pending activation is now None. Current and shared-server targets remain 290744e84. Shared daemon PID 454202 remains live and coordinator continuity was observed after supported reload. Debug `sessions` audit was rejected because debug control is disabled; no shared settings were changed to bypass it. Therefore other-session continuity and explicit owner-by-owner coordination remain unverified, not passed. The task checklist's coordination item denotes supported reload completion only and must not be interpreted as proof all owners were contacted. Promotion already occurred; this audit records the evidence gap rather than requesting retroactive authorization.
