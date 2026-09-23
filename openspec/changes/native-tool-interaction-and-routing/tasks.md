## Execution gate

Do not start implementation before proposal approval. All tasks remain unchecked. Each task must record commands, actual binary/socket, observations and blocked prerequisites. Preserve unrelated dirty files. Numbers below form an acyclic dependency order, not mandatory generic phases.

## 1. Contracts and inventory

- [ ] 1.1 (depends: approval; Q1,Q4,P1) Inventory registry names/platform/session gates and lock question wire schema, payload bounds, capability negotiation and exact canonical name. Add validation/registry tests. Paths: tool-core, tool-types, protocol wire, app-core tool/mod.rs and tests.rs.
- [ ] 1.2 (depends: approval; F1,F2,F4) Define opt-in Firecrawl config, exact-host allowlist, secret boundary and literal-argv adapter. Inspect installed supported CLI contract; no installation. Validate parser defaults and rejection scenarios without network; keep global config unchanged.

## 2. Structured question lifecycle

- [ ] 2.1 (depends: 1.1; Q1,Q3,Q4) Implement native tool and session-owned pending store/events/replies. Wire both turn execution paths, cancellation, shutdown and replay. Verify wrong-session, duplicates, concurrent calls, old-client/headless/worker unavailability and no agent-mutex deadlock with focused protocol/server tests.
- [ ] 2.2 (depends: 2.1; Q2,Q3) Implement TUI question state, keyboard navigation, review/submit, Other, Esc, wrapping and preserved draft. Verify real tester frames for single/multiple questions, multi-select, text editing, narrow screen, cancel and reconnect. Do not repurpose command stdin UI.

## 3. Explicit hosted fetch

- [ ] 3.1 (depends: 1.2; F1-F4) Add optional backend and freshness parameters with strict format checks. Execute bounded child process, parse page/API outcomes, normalize metadata, enforce pre-request/final-URL policy, kill/reap on timeout/cancel. Test schema, unknown metadata, malformed output, HTTP 404, missing credentials, DNS rejection, limit handling and no automatic retries. Extend existing webfetch corpus tests without replacing them.
- [ ] 3.2 (depends: 3.1; F1,F3,F4) Exercise actual native tool on isolated new binary: direct static/docs and enabled allowlisted Firecrawl JS/404 with bounded live credits. Run without Firecrawl installed/in PATH to verify direct still works. Check timeout/cancel child cleanup. Record rate-limit/auth simulation separately from actual hosted checks; never claim simulated tests as hosted acceptance.

## 4. Existing custom tools and prompt

- [ ] 4.1 (depends: 1.1; P3,P4) Reconcile browser action enum/dispatch, scope propagation and selection validation. Preserve manual actions. Run browser_tests and a harmless real scoped-tab workflow under applicable browser skill/policy. Unavailable safe bridge resolution is a blocker, not permission to expand the archived browser project. No live destructive action or private Jev egress.
- [ ] 4.2 (depends: 1.1,2.1,3.1,4.1; P1,P2,P4) Add deterministic capability-aware routing module and composition tests in jcode-base prompt.rs/prompt_tests.rs plus agent prompting. Cover every inventory entry/family including evaluate, computer use, browser, MCP/skills/discovery, memory/history, swarm/bg/schedule, side_panel and conditional tools. Verify global/project replacement and overlay precedence with isolated directories. Do not modify user prompt files.
- [ ] 4.3 (depends: 4.2; P1,P4) Verify Linux prompt excludes macOS-only calls and macOS prompt includes actual computer discovery/AX/permissions/dry-run guidance. On a macOS host run harmless permission/observe workflow without focus theft; record platform blocker if unavailable. Verify evaluate is never treated as human approval and Jev provider guidance remains compatible with unified-jev-service-config.

## 5. Integration acceptance and delivery

- [ ] 5.1 (depends: 2.2,3.2,4.2,4.3; Q1-Q4,F1-F4,P1-P4) Run focused cargo tests for jcode-tool-core, jcode-protocol, jcode-base prompt and jcode-app-core tool/server areas, plus jcode-tui input/render tests. Use coordinated selfdev tests/build target=tui. Confirm exact package/test names before execution. Test direct API and SDK-native result paths with actual supported providers or record route blockers.
- [ ] 5.2 (depends: 5.1; Q3,Q4,F1,P2) Run newly built binary against its own socket and actual TUI client: ask, answer, resume model, cancel, reconnect, clear, restricted-tool exposure, and custom base prompt. Check packaged/default-direct startup with no optional Firecrawl executable. Validate no shared daemon replacement, no orphan processes, no user prompt/config edits and no secret artifacts.
- [ ] 5.3 (depends: 5.2; all) Publish requirement-to-evidence table with observed pass/fail/blocked for each scenario, feature docs and migration notes. Keep direct default and old-client unavailable behavior explicit. Commit only owned changes after tests. Shared rollout requires separate approval; never mark blocked macOS/provider checks passed.
