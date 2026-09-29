# Consolidation inventory

Evidence snapshot: main `5b90904c7`. Source refs and stashes remain recoverable in the protected snapshot. Runtime files are not source features and are not added to Git.

## Branch history

| Source branch | Disposition | Evidence |
| --- | --- | --- |
| codex/agentmail-ambient-channel | Included by ancestry | 268f9b117 |
| codex/fix-cliproxy-stream-idle | Included by ancestry | 399614475 |
| codex/fix-jcode-session-stall-20260922 | Included by ancestry | 467392fc4 |
| codex/session-lock-order-source | Equivalent lock-order implementation included | Added and removed code lines equal to d8de03c2a; lock-order-equivalence.json |
| codex/session-stall-and-proxy-notes-20260923 | Included by ancestry | 467392fc4 |
| codex/systemone-config-merge | Included by ancestry | bb7c3b84f |
| codex/systemone-config-routing | Included by ancestry | 56a302f12 |
| feature/question-pane-and-overnight-advice | Included by ancestry | fa773a4a8 |
| main | Included by ancestry | 3ddd14521 |
| upstream-master | Included by ancestry | 4834a881a |
| backup/candidate-before-master-refresh | Backup history retained, equivalent lock patch | 132284722 and d8de03c2a stable patch IDs match; historical docs retained in backup |
| codex/pre-unified-custom-port | Included by ancestry | e5122e2bd |
| codex/systemone-config-088 | Included by ancestry | b4f04fb86 |
| codex/systemone-config-088-deploy | Included by ancestry | d8de03c2a |
| codex/unified-history-20260924 | Included by ancestry | d8de03c2a |
| master | Included by ancestry | e5122e2bd |

## Custom capability batches

| Capability | Destination/evidence | Remaining acceptance |
| --- | --- | --- |
| Automation daemon and private bulletin | c9382edfa then 233d2646c; 42 focused tests and full app-core passed | Final committed-main suite and artifact smoke |
| AgentMail and stream-idle fixes | Original main ancestry retained; v0.88 compatibility compile/full app-core passed | Final runtime/accounting review |
| Source editor links and repository-aware release prompt | Seven root custom paths included by explicit b7d98c9cb comparison baseline; full TUI suite passed | Final artifact smoke |
| Jev cache/evaluate/review/compaction/router/foreman/poke | Custom deltas retained in 233d2646c; schema sweep 59 tests passed, poke 22 | Final caller/runtime review, especially runtime-only paths |
| System One route and explicit subscription | 5b90904c7; Jev 56 passed, memory 5 passed | Final suites/build |
| Schema/downgrade guards and code intelligence | 39 schema, 52 build-support, 3 script tests passed in first batch | Final combined tests |

## Dirty work and stashes

- Original primary dirty files and both stashes remain untouched. Protected inventory lists 96 tracked paths and all untracked paths. Counts reflect snapshot time.
- Unfinished question-pane UI, remote-desktop integration and browser-profile replacement are deferred with original source, stash objects and verified recovery archives retained. No new feature development is authorized as part of consolidation.
- Disconnected question/remote-desktop hooks that prevented compilation were removed only from combined main. Their partial implementations remain recoverable.
- Current System One subscription additions were ported into shared Jev route; remaining custom service adapter consistency requires review, not an assumption of completion.
- Stash comparisons: 1804 identical entries, 17 different, 12 absent locally, four deletions/non-files. Two differing code files add subscription enum/mapping; runtime/index differences remain preserved. Neither stash is dropped.
- This inventory is a progress ledger, not final item-by-item closure. Do not remove worktrees until remaining acceptance and unique-item review are complete.

## Remaining-work reconciliation, 2026-09-29

- `bf08fe71e` → `ba9e4138a`: SSE recovery order and mixed newline delimiters. Independent review approved; entire OpenRouter provider suite38 passed.
- `00a1cb71b` → `c85d46505`: terminal-context propagation into foreground/background shell and InputShell. Independent review approved; client-actions15, hooks14 and scoped subprocess regression passed.
- `2ad3e7230`: machine-local config, MCP config and prompt snapshots, not a source feature. Retained on original branch; do not copy secrets/config into main.
- `a81fb03dd`: mixed139-file capability snapshot, only partially converged. System One routing is integrated, but browser-profile, CLI, question UI, desktop and other capability groups require separate item-level review. Do not mark the entire commit equivalent or port wholesale.
- MCP/evaluate commits `60db09a8d`, `50035dbee`, `bf9d2db37`, `c080c32d1`, `6f2af8f81`, `40ab1b4f4`, `a3980f858`: runtime behavior converged in `afbbd12f5` and live-verified `a65876997`; historical tests/evidence stay recoverable on source branch, not wholesale imported.
- Eight dirty Rust files inspected on original checkout: formatting-only deltas, not a new functionality batch. Left untouched. Machine-local config/MCP changes and unrelated untracked data remain private.
- Both stashes and two orphan source archives remain preserved. Their contents are not declared merged merely because backups exist. Detailed private reports: `scratch/worktree-integration-audit/remaining-{commits,dirty-source,preservation}-20260929.md`.
- Fresh private tracked-delta snapshot: `scratch/worktree-integration-audit/remaining-20260929/working.patch`, with refs separately preserved. No reset/stash/pop/delete or remote push performed.
- `bca5fa660` lock-order candidate: core implementation already present through main ancestry `d8de03c2a`. Main retains newer resume/prewarm behavior. No new source delta remains in its four paths, so no duplicate commit. Fresh client-session33 and swarm30 tests passed, including deterministic coordinator-election/subscribe coverage. Initial candidate readiness was corrected after examining exact main history, not inferred from branch reachability.

### Remaining capability gates and recovery

The mixed snapshot is **140 paths by `git diff-tree`** (earlier139-file label was approximate). Capability audit is preserved privately at `scratch/worktree-integration-audit/mixed-capabilities-20260929.md`; it is not proof of whole-commit equivalence.

| Remaining group | Disposition / gate | Recovery |
| --- | --- | --- |
| Browser profile and ChatGPT browser runtime | Deferred: profile contract and implementation tasks remain open. Require scoped source comparison, supported profile contract and real isolated-browser acceptance before any port. | `a81fb03dd`, original dirty checkout, browser-profile proposal/evidence |
| Question-pane, overnight and remote desktop | Deferred unfinished capability work. Require feature-specific design/implementation completion and TUI/desktop acceptance, not implicit completion during consolidation. | `a81fb03dd`, `feature/question-pane-and-overnight-advice`, retained stash |
| Remaining CLI/TUI/agent compatibility and fixtures | Deferred/unclassified at item level. A matching identifier or already-integrated routing subset does not establish equivalence for whole files. Require scoped semantic comparison before inclusion or duplicate classification. | Own source branch `a3980f858`, mixed source commit `a81fb03dd` |
| Machine-local config, MCP entries and runtime artifacts | Preserve privately, exclude from source ports. | Original checkout and private snapshots |
| Generated Graft stash | No tracked-tree delta against parent; inspect untracked third parent separately if retrieval is needed. Do not pop into main. | `1fd03d9c31821987e2d55a0850fa5514bec412db` |
| Unrelated workspace stash | 92 tracked paths, mixed unfinished work. Preserved without applying. Require per-capability extraction rather than whole-stash merge. | `b7095d47bb2d0136d172d5b3994aff6407ad053f` |
| Orphan linked checkout source | Deferred: archived and checksummed, not proven equivalent to current main. Keep until individual source accounting and restore evidence permit retirement. | Private `scratch/worktree-integration-audit/` manifests/archives, original orphan directories |

The two ready source batches are complete. Remaining item-level accounting is explicitly deferred, not silently marked done. Original dirty diff SHA256 still exactly matches the pre-integration snapshot. No recovery source, stash, branch or orphan directory was deleted. Combined main9109a7b10 passed isolated public-shell and configured Omni streaming acceptance; deployment was not changed.
