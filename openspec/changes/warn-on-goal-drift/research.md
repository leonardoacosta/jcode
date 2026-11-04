# Local authoring evidence and readiness

## Binding and existing owners

Observed 2026-11-03 UTC. Repository `/home/nyaptor/.jcode`, origin `https://github.com/leonardoacosta/jcode.git`, HEAD `54f920477a1e32e71547f514ecee093d0340199f`. Relevant tracked targets clean at inspection; unrelated untracked councils/devices/exit records/swarm claims and existing `add-config-composition` remain preserved. This task owns only `openspec/changes/warn-on-goal-drift/`. No tracker or roadmap outcome was selected. Read repository native `spec-driven` schema/templates and current capability/active/archive inventories.

Source SHA-256 bindings:
- `crates/jcode-base/src/goal.rs`: `53abc4fa1dd790213dfdfad0a3606fe4f23dac762de6425650fa664000f023fc`
- `crates/jcode-base/src/compaction.rs`: `98143530acbe87122e86432e9ce2dda4c20b8a91fe5a2c123d4091b022ad8db6`
- `crates/jcode-app-core/src/agent/turn_execution.rs`: `09de6c49a5d36cf450efcb882fabeff0007ed0bd1f3b1cc1d72780685349d2e0`
- `crates/jcode-base/src/mcp/manager.rs`: `02d9544adf488c5bf76ff58df123000fca0ca8e68f06bd14c7ff77e3d577d57f`

`goal.rs` owns persistent create/update/attach/resume and goal-memory mirroring. `initiative` tool owns goal actions. `turn_execution.rs` has normal, capture and streaming paths, with hooks not equally covering every path; implementation must inventory the shared inner boundaries, not assume one hook suffices. `compaction.rs` compares older/newer halves of embedding history, not explicit goal alignment. `McpManager::call_tool` is reusable, but internal invocation must preserve existing tool policy rather than bypass it.

`route-jev-supervision-explicitly` defines a different direct HTTP route. No existing runtime consumer of `JevClient::for_supervision` was found outside tests. Do not create a dependency or reuse memory credentials implicitly. Existing continuation-evidence spec explicitly preserves continuation behavior; this proposal adds advisory behavior only and retains that contract.

## Jev capability observation

MCP discovery through configured proxy returned `Jev:jev_classify`: shared classes/context, up to 64 items, item text truncated at 2,000 characters, class probabilities/confidence/margin and auto/review decision. Other tools include rerank and review. This was schema discovery only: no conversation or private repository content was submitted to Jev. Availability in discovery does not prove an authorized native daemon connection, healthy execution or permission for per-turn disclosure.

The chosen proposed design is opt-in, one bounded classification per completed user-visible turn, advisory-only and explicitly selected MCP transport. No user decision grants live disclosure by artifact creation. No live Jev check ran. Initial 0.85/0.50/two-check thresholds are uncalibrated implementation defaults; fixture and live schema checks are not accuracy guarantees. No raw transcript, hidden reasoning, tool/file contents or memory corpus enters the proposed payload.

## Pre-draft and handoff assessment

Rubric `preflight-check-v2-shadow`, phases pre-draft and handoff, all six groups inspected over implementation, offline acceptance, independent review, persistence and archive. Local authoring and validation eligible. Implementation not requested. Live monitoring unavailable until exact route/tool selection, permitted connection and data opt-in exist; these block live activation, not offline implementation preparation. No consequential decision is invented: remote monitoring defaults off.

| Group | Points | Evidence and limits |
| --- | --- | --- |
| G1 | 2 | Explicit Feature request, advisory-only opt-in scope, no implementation/publication authority. |
| G2 | 1 | Existing local Rust/MCP/goal interfaces and test route; final feature tests, independent review and optional synthetic live MCP check not run. |
| G3 | 1 | Native OpenSpec available and Git identity configured from earlier local observation; conflicting own-branch/preserve-checkout rules and main versus local origin/dev unresolved. No Git mutation. |
| G4 | 2 | Jcode TUI/CLI runtime and existing session goal owner only; no Desktop or external task-source ownership changes. |
| G5 | 2 | Clean relevant tracked targets and isolated new proposal directory, unrelated work preserved. |
| G6 | 2 | Every acceptance slice maps to numbered tasks including final review/commit/archive; no semantic prerequisite change, no roadmap rewrite. |

Shadow quality 10/12 = 83.33%, not an admission gate. Authorized repairs: named all turn paths, separated direct Jev route from MCP, specified no tool-policy bypass, freshness/uncertainty handling, existing-goal ownership and explicit live-data boundary. Remaining gaps are real execution evidence and Git policy, not removable by rewording. No provider/credential change, review bypass or policy alteration attempted.

No claim of independent execution-readiness review, runtime feature acceptance, calibrated semantic reliability, implementation, commit, publication or archive. First execution step after an explicit Apply is task 1.1 with refreshed source/ownership bindings. Final implementation must pass fresh gates then independent review before commit and archive. Structural validation alone does not make live monitoring enabled or authorized.

## Authoring validation and final preservation observation

`openspec validate warn-on-goal-drift --strict` passed. Canonical dependency validator returned exit 0, one valid selected change and no prerequisites. `openspec status --change warn-on-goal-drift` reported four of four planning artifacts complete. Explicit new-file checks passed: five requirements, ten scenarios, local references and trailing whitespace. No Rust implementation/tests or live private Jev evaluation ran for this new feature.

At final inspection, unrelated concurrent edits appeared in `crates/jcode-app-core/src/tool/mod.rs`, `crates/jcode-app-core/src/tool/tests.rs`, `crates/jcode-base/src/session_tests/mod.rs`, `crates/jcode-config-types/src/lib.rs` and untracked `crates/jcode-app-core/src/tool/plan_guard.rs`. They are not this task's changes and remain untouched. Configuration integration bindings must be refreshed before implementation; do not stage or commit these paths as proposal work. The proposal remains local and uncommitted under the previously recorded policy conflict. `ropen` is unavailable, so no tailnet review URL was published.

## Scoped planning publication, 2026-11-04 UTC

User explicitly requested committing and syncing proposals, implementation later. Publish only these planning artifacts on owned `docs/config-session-proposals` ref from the current HEAD without switching the primary checkout. This preserves unrelated concurrent changes and satisfies own-branch isolation without moving `main` (remote default remains `dev`). No implementation checkboxes are marked complete, no archive or merge is included. Fresh strict validation and selected dependency checks pass. Historical preflight observations above remain revision-bound; implementation must refresh them.
