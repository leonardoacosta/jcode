# Verification ledger

## Approval and evidence boundary

Definition-only change. Awaiting user approval. Planned runtime checks below have NOT run against an implementation of this feature. Existing-tool reconnaissance is baseline evidence, not acceptance of proposed behavior. No approval is inferred from automated continuation messages.

## Changed deliverables: checks actually executed

| Deliverable | Concrete check | Observed result |
|---|---|---|
| proposal.md | Installed `openspec validate native-tool-interaction-and-routing --strict --no-interactive` and `openspec status --change native-tool-interaction-and-routing` | Valid; proposal recognized complete |
| specs/ (three capabilities) | Same installed commands plus requirement/scenario audit | Specs recognized; 12 unique ids and 24 scenarios |
| design.md | Installed status plus referenced source path and scope audit | Design recognized; async tool support, override precedence and browser drift grounded in source, not runtime acceptance |
| tasks.md | Installed status plus dependency-order audit | Tasks recognized; 12 tasks, dependencies precede consumers, all implementation tasks unchecked |
| README.md and .openspec.yaml | Installed change discovery/status; compare summary against counted artifacts | spec-driven change discovered; summary reflects 12 tasks and pending approval |
| All committed artifacts | `git diff --cached --check` before commit; path-limited status after commit | Whitespace passed after EOF correction; committed definition directory clean |

## Baseline runtime observations, not feature acceptance

Actual existing webfetch and Firecrawl CLI were exercised on static HTML, long docs, JS content and a missing page. Native fetch omitted JS quote content; Firecrawl returned it. Native 404 was an error; Firecrawl CLI exited 0 with target status 404. Cache-bypass flag was requested in a follow-up, but cacheState was absent. These observations justify F1/F3 boundaries, not proof the unimplemented native adapter normalizes them. Debug socket inspection was blocked by disabled control. No new question UI, prompt module, browser repair or packaged adapter has been executed.

## Requirement/scenario-to-check map

Status for every row is **NOT RUN: implementation awaits approval**. Provider, browser, Firecrawl and macOS prerequisites must be checked at execution. Controlled fixtures are supplementary; they cannot substitute for the real integration checks specified alongside them.

| Requirement | Scenario | Task(s) | Concrete check and expected observation |
|---|---|---|---|
| Q1 | Valid answer | 2.1 | Invoke real question tool through capable TUI; select an option and Other text; inspect matching tool result. |
| Q1 | Invalid input | 1.1,2.1 | Invoke tool with duplicate ids and over-limit payload; submit invalid single-select response via protocol; verify rejection and no completion. |
| Q2 | Answer a batch | 2.2 | In real TUI answer four questions by keyboard, including multi-select and Other; verify atomic result and restored draft. |
| Q2 | Cancel or small terminal | 2.2 | Resize real TUI to narrow viewport, navigate wrapped descriptions and press Esc; verify one cancellation and preserved draft. |
| Q3 | Reconnect | 2.1,2.2 | Detach/reconnect actual client while question pending; answer replay and inspect original tool call resumption. |
| Q3 | Race or termination | 2.1,5.2 | Exercise duplicate/foreign replies, concurrent requests, turn cancel, clear and isolated daemon restart; verify each terminal outcome and no leaked wait. |
| Q4 | Unsupported caller | 1.1,2.1 | Exercise actual CLI/direct invocation, worker, restricted root and old-capability client; check tool list and immediate unavailable responses. |
| Q4 | Provider resume | 5.1,5.2 | Answer via actual direct-API and SDK-native routes on isolated daemon; inspect one matching result followed by model continuation. |
| F1 | Legacy call | 3.2,5.2 | Start packaged isolated binary with Firecrawl absent from PATH and credentials absent; fetch text/markdown/html with omitted backend. |
| F1 | Unsupported format or missing setup | 3.1,3.2 | Invoke native hosted backend with unsupported format, missing executable/auth/enablement; verify actionable failure and no install or fallback. |
| F2 | Public allowlisted URL | 3.1,3.2 | Invoke native hosted backend for explicit allowlisted public URL; inspect result and sanitized dispatch evidence for literal argv/final URL. |
| F2 | Unsafe URL | 3.1 | Exercise public-policy validator for URL userinfo, IP/local host, mixed DNS and disallowed final host using controlled resolver/redirect tests; do not contact metadata/private services. |
| F3 | Rendered page | 3.2 | Invoke new native hosted backend on public JS page; check quote text, backend and untrusted-content marking. |
| F3 | Target failure or missing metadata | 3.1,3.2 | Invoke native backend on target 404; verify failure. Exercise omitted-cache response parsing; ensure unknown remains unknown. |
| F4 | Large result | 3.1 | Drive real CLI-adapter boundary with bounded controlled oversized output and long content; observe memory/output bounds. Mark controlled process evidence separately from hosted service. |
| F4 | Failure or cancellation | 3.1,3.2 | Cancel/timeout real adapter subprocess and inspect process exit/reaping; use controlled error responses for malformed JSON, rate limit and auth failure without provoking quota abuse. |
| P1 | Enabled registry | 4.2 | Compare effective tool definitions and assembled prompt for normal/restricted/ambient/selfdev sessions; require family coverage for every name. |
| P1 | Platform difference | 4.3 | Inspect assembled Linux and macOS prompts from actual builds; run harmless macOS permissions/discover checks on a macOS host. |
| P2 | Global replacement | 4.2,5.2 | Launch isolated sessions with project/global overrides and overlay; inspect assembled prompt for one routing module, precedence and unchanged files. |
| P2 | Safety or unavailable service | 4.2 | Exercise prompt composition with disabled hosted tools and stricter browser skill constraints; inspect no contradictory bypass instructions. |
| P3 | Scoped selection | 4.1 | Use authorized harmless real browser task in owned tab/window/frame; verify observation/action scope and advertised enum. |
| P3 | Invalid selection | 4.1 | Use controlled stale/unsupported/missing-value selections and unavailable Jev; verify no dispatched mutation and manual actions remain usable. |
| P4 | Task selection | 4.2,5.1 | Run bounded provider tasks representing preference/evaluation/document/live UI; inspect actual tool choices and capability-aware fallbacks. |
| P4 | Mutating action | 4.2,5.1 | Use non-destructive simulated send/purchase/delete scenarios; inspect model/tool behavior for consent checks without executing real side effects. |

## Packaging and likely failure modes

Task 5.2 covers optional dependency absence, isolated-server startup, actual binary provenance, old-client negotiation and custom-prompt packaging. Q3/F4 cover pending waits and orphan children. F2/P3 cover scope/egress and unintended mutation. Auth/rate-limit/malformed payload tests remain controlled until a safe live reproduction exists. macOS checks are BLOCKED if no macOS host is available. No broad reliability or performance improvement is claimed from reconnaissance or structural validation.
