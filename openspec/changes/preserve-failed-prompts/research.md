# Research: failed-prompt recovery

## Scope and evidence binding

- Request: author a native proposal in the Jcode source repository. Preserve failed prompts and surface their full text in a copyable block. Do not implement the harness fix. Repository: `https://github.com/leonardoacosta/jcode.git`; inspected revision `bd71f149baf175326ffa55e2e467951b3707a08e` on `main`, 2026-10-30 UTC.
- The source files cited below match that revision. `git diff --exit-code` over those five input/history/error files returned zero.
- Existing configuration and `crates/jcode-base/src/mcp/` edits are outside this change. Their tracked diff SHA-256, excluding this change directory, is `e5c3504f5743cf0f16c67e56cff62cead3c67230e07e965cd85ec9ab30fce492`. Preserve existing untracked files without inspecting their contents.
- Native OpenSpec resolves `/home/nyaptor/.jcode`, schema `spec-driven`. The five existing capabilities do not define interactive failed-prompt recovery. The potentially adjacent `personal-prompt-experiment-adapters` capability concerns confined experimental trials, not composer recovery.

## Observed source behavior

| Source | Observation |
| --- | --- |
| `crates/jcode-tui/src/tui/app/input.rs:3933-3947` | Submission takes the composer text, records eligible history, expands paste placeholders, then clears paste storage and undo before skill validation finishes. The size-limit rejection already restores raw input before clearing paste storage. |
| `crates/jcode-tui/src/tui/app/input.rs:4074-4094` | Unknown or unavailable skills emit an error and return before `last_submitted_input` is assigned or the expanded user message is recorded. |
| `crates/jcode-tui/src/tui/app/prompt_history.rs:212-232` | Persistent history excludes secret-intercept inputs, oversized input, slash commands and shell commands. History is not a universal recovery mechanism. |
| `crates/jcode-tui/src/tui/app/model_context.rs:1236-1264` | Error recovery takes `last_submitted_input` before checking whether a newer composer draft exists. It avoids overwriting that draft but does not retain the taken value there. |
| `crates/jcode-tui/src/tui/app/input.rs:3031-3044` | `PreparedInput` already carries raw input, expanded text and images, but preparation clears paste storage. |
| `crates/jcode-tui/src/tui/app/remote/input_dispatch.rs:65-72,147` | Remote recovery restores raw text and images; the last-submitted fallback also stores raw input. A raw paste placeholder alone is insufficient after its backing storage is cleared. |
| `crates/jcode-tui/src/tui/app/remote/server_events.rs:569,1460-1472` | Remote errors have an `id`; existing fallback handling already snapshots a failed turn's payload before cleanup. Reuse these boundaries. |
| `crates/jcode-tui/src/tui/app/copy_selection.rs:375-391` | Existing copy behavior accepts a clipboard callback and preserves selection on copy failure. Recovery must copy authoritative text, not a truncated viewport. |

These observations establish loss-prone paths, not the cause of the historical incident. Its original prompt was not recovered here. No historical-prompt recovery, live provider call or harness modification was attempted.

Repository-local Graft located input tests and copy integration. Configured Graft MCP resolved an unrelated workspace, so lookup fell back to the local index, then narrow source reads verified the cited behavior.

## Alternatives and resolved design

1. **Restore the composer only:** smallest change, but cannot satisfy the copyable-block requirement and risks losing recovery when a newer draft exists.
2. **Persist every submission in history or a new journal:** helps crash recovery but changes privacy/retention semantics, including intentional slash-command and secret exclusions. Not selected.
3. **Retain the expanded user-authored text for the submission and expose it with the error:** selected. Reuse prepared input, existing failed-turn state, error display and clipboard handling. Restore only an empty composer; retain a separate recovery block when it is occupied.

The authoritative recovery text is the user's logical prompt after paste expansion, including a typed slash invocation, not injected skill instructions, system context or image bytes. Recovery is in-session, not a new disk journal. No new automatic retry, dependency or wire-format change is planned. Existing retry policy remains unchanged. Transport ambiguity must be visible before a manual resubmission.

## Pre-draft readiness

Assessment: `preflight-check-v2-shadow`, phase `pre-draft`, 2026-10-30T00:59:47Z. Actor: Jcode; target: this local source checkout; permitted effects: planning artifacts and scoped local persistence only.

**Eligible:** author proposal, delta spec, design, research and tasks. Material scope decisions are resolved by the request and the bounded defaults above. **Blocked:** implementation until a new explicit apply request. **Outside scope:** publication, runtime reload and archive. Runtime acceptance remains unverified and must not be inferred from planning validation.

| Group | Score | Evidence and limits |
| --- | --- | --- |
| G1 Scope and authority | 2 | Explicit proposal-only request and full-copy requirement. No source edits authorized in this turn. |
| G2 Verification route | 2 | Native CLI resolves the root and schema. Documentation validation is available; implementation acceptance has a separate future route in tasks. |
| G3 Delivery and checkout | 2 | Current root `AGENTS.md:28-35` explicitly prefers the primary checkout. The checkout-only policy points to root `AGENTS.md` for repository instructions; retain `main` rather than follow its older duplicated own-branch line. Configured Git identity is present. No publication authorized. |
| G4 Runtime ownership | N/A | This selected operation changes documentation only, not any running daemon or client. |
| G5 Existing work | 2 | Dirty paths inspected, unrelated diff bound above, unique new change directory selected. No overlapping source edits. |
| G6 Completion coverage | 1 | Required schema artifacts and acceptance cases identified; concrete artifact-to-task mapping awaits authoring and handoff recheck. No prerequisite change identified. |

Shadow quality: `100 × 9 / 10 = 90`; applicable groups G1, G2, G3, G5 and G6. This is an uncalibrated documentation-readiness assessment, not confidence that the fix works. The product roadmap explicitly calls itself a strategic map, not an execution ledger; no roadmap binding or cross-change dependency metadata is required.

## Handoff verification

Assessment: `preflight-check-v2-shadow`, phase `handoff`, 2026-10-30 UTC, same source revision and preservation boundary. The four native planning artifacts exist and strict validation returned `Change 'preserve-failed-prompts' is valid`. Native artifact status is complete; that is not implementation status. All 10 implementation checkboxes remain unchecked.

A local integrity check passed: six requirements, 13 scenarios, all requirements mapped to tasks, exact expected file set, resolved relative links, balanced Markdown fences, no pre-existing staged work and unchanged unrelated tracked-diff digest. Coordinator review clarified that existing transcript retention remains authoritative rather than promise a separate persistence redesign. No native dependency check is claimed or needed because this change has neither roadmap execution bindings nor cross-change prerequisites.

**Eligible:** review and hand off this proposal; scoped local documentation persistence under the verified root checkout policy. **Blocked:** tasks 1.1 through 4.2 until a new implementation request. Their Rust tests, live local/remote TUI behavior and clipboard acceptance have not been run. Publication and archive are not authorized or performed. No build, install, credential/routing change or runtime reload occurred.

Handoff shadow quality: G1=2, G2=1, G3=2, G4=N/A, G5=2, G6=2; `100 × 9 / 10 = 90`. G2 is partial because runtime/clipboard acceptance is intentionally unobserved in this proposal-only turn. G6 now has the concrete requirement-to-task matrix. Other reasons remain as above. This uncalibrated score does not establish implementation readiness or a working fix; no extra runtime work is authorized merely to improve it.

Artifact binding, SHA-256, excluding this self-referential research record:

| Artifact | SHA-256 |
| --- | --- |
| `proposal.md` | `669033154698ebe408989d00129cb9f884d84017943e298fe5f10d30dd6ed847` |
| `design.md` | `bff329f7cc26dd39f3561ef73c95bfcbfb2277fe37dd84b01828f9dea36be499` |
| `tasks.md` | `5f5af194eb32f99b5acc2cb959025e93ea59a82b1a3b9ba47d708e240b7cb42c` |
| `specs/failed-prompt-recovery/spec.md` | `85a8d935b67502a45ecdc849641156f73ed3e922891a9ed6970c8ca83f042e64` |

Native consumer acceptance recheck, 2026-10-30 UTC: `openspec show preserve-failed-prompts --type change --json` parsed six `ADDED` deltas for the intended capability. Exact comparison against the authored spec confirmed all six normative bodies and all 13 scenario bodies survive native parsing unchanged, including the full-copy obligation and explicit rejection of truncated previews. `openspec list --json` reported `totalTasks: 10`, `completedTasks: 0`, `status: in-progress`. This verifies that downstream native consumers receive the intended contract without implying a completed fix. The initial comparison harness assumed a scenario `name` field; inspecting the actual schema showed `rawText`, and the corrected exact-body comparison passed. No proposal repair was needed.

Recheck source revision, dirty-target ownership, actual runtime/daemon identity and the relevant acceptance route before implementation. Artifact edits invalidate their bindings. No material decisions remain unresolved for proposal handoff. Historical incident attribution and future runtime acceptance remain unverified, not silently assumed.

## Apply execution preflight

Selected by the user on 2026-10-30, pinned start `efb6f3dc534d90fe948b31836c525908b00f4f82`. The separate apply request satisfies the authoring-only gate in tasks. Native apply instructions report `ready`. Proposal/design/task bindings still match the prior handoff; there are no prerequisite changes or experimental evidence dependencies. The product roadmap remains strategic, not an execution ledger.

Implementation and isolated tests are eligible. One remote-slice worker owns remote submission/event files and its existing remote test file; the coordinator owns local input/recovery, presentation/copying and root tests. Git mutations remain serialized. Existing configuration/MCP changes and untracked files remain outside this change. Current root `AGENTS.md` retains the primary checkout; no worktree or branch switch is made.

Observed toolchain: configured Rust/Cargo 1.98.1, with no repository `rust-toolchain` override. Repository CI uses formatting and Clippy with warnings denied. Selected checks are targeted and full TUI tests, formatting, affected-crate Clippy, strict OpenSpec validation and isolated source-built runtime observation. New local regression tests exercise actual app submission and restoration. The initial build is compiling dependencies, not a confirmed test outcome.

Assessment `preflight-check-v2-shadow`, phase `initial-dispatch`: G1=2 (explicit apply selection), G2=1 (test route available; runtime/clipboard observation pending), G3=1 (local commit route established; external push not authorized), G4=1 (isolated test target required; actual new binary/session not yet observed), G5=2 (disjoint ownership and preserved unrelated state), G6=2 (all ten authoritative tasks and requirement mapping loaded). Shadow quality is `100 × 9 / 12 = 75`; this is not an admission gate. Repairs are to establish source-built runtime evidence and complete fresh checks, not to modify live credentials or routing.

Closeout remains blocked until gates, independent review, verified task state, spec sync/archive and applicable persistence are satisfied. Native selfdev/debug_socket MCP tools are not exposed in this harness; documented CLI and repository build fallback are being investigated.

## Apply verification, 2026-10-30

Expanded user-authored text is retained before destructive preparation. A UI-only `failed_prompt` block holds exact source; only an empty composer is restored. Rejections state `Not sent.` Remote failures retain the typed slash invocation, not injected skill instructions. Existing secret interception, history exclusions and attachment handling remain intact. No dependency, prompt journal, credential or live routing change was added.

Full-source copying uses **copy mode → P**, plus the existing raw-copy badge. Real keyboard testing exposed Alt+Y's existing copy-mode binding, so the conflicting candidate shortcut was removed. Failed text is plain-rendered with control characters escaped; the clipboard payload remains unmodified. A failing injectable clipboard callback preserves recovery and newer drafts with failure feedback. This is callback-boundary evidence, not an observed OS clipboard outage.

### Native acceptance

The private fixture `$JCODE_SCRATCH_DIR/failed-prompt-acceptance-wi61hla_/result.json` reports `passed`. Reproduction uses `$JCODE_SCRATCH_DIR/failed_prompt_acceptance.py` and `/home/nyaptor/.jcode/target/selfdev/jcode`, built via `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode`. Passing-run binary SHA-256: `857bd2de18256dd2a95aad7acc45305f3b1793d11614880cedf9d303458bcd06`.

An owned private daemon, isolated HOME/runtime/session and localhost synthetic HTTP provider avoid live credentials, routing and active sessions. Cleanup terminates owned processes in `finally`. Unknown-skill multiline paste rejection exposed the complete failed source with zero provider requests. Actual copy-mode keyboard events yielded OSC52 clipboard bytes exactly matching Unicode and preserved whitespace, including offscreen source. A real daemon-connected HTTP 400 failure retained a newer draft and copied the failed source exactly. Exactly one `/v1/responses` request occurred; rendering/copying caused no resend.

The native probe initially failed because deterministic rejection stopped auto-poke and returned before recovery. Terminal stopped-auto-poke, overnight and credential-breaker branches now restore recovery. Server live-turn failures legitimately use broadcast ID `0`, which the stale-request guard must allow. Temporary state-only tracing used during diagnosis was removed.

### Fresh checks before final association review

- `cargo fmt --all -- --check`: exit 0.
- `scripts/dev_cargo.sh clippy --locked -p jcode-tui --all-targets --all-features -- -D warnings`: exit 0.
- Isolated serial TUI library suite: **2864 passed, 0 failed, 20 ignored, 1 filtered**, exit 0. Invocation follows CI serial execution and named color/shimmer exclusions, with `COLORTERM=truecolor`, private HOME/JCODE_HOME, a credential-free environment, system PATH and scratch Git ceiling. An earlier parallel run raced environment-mutating tests. Earlier serial failures exposed an expected-message update and clipboard fixture PATH issue, resolved before this passing run.
- `openspec validate preserve-failed-prompts --type change --strict` and `git diff --check`: exit 0.
- Unrelated configuration/MCP diff is byte-identical to preflight: SHA-256 `e5c3504f5743cf0f16c67e56cff62cead3c67230e07e965cd85ec9ab30fce492`. Concurrent unrelated documentation/untracked changes remain outside this change.

A separate final requirements/diff review identified a possible stale nonzero error while a newer prepared submission has no current message ID. A bounded implementation worker is adding a reachable-path regression and any necessary minimal association repair. A manually injected lifecycle state is not sufficient evidence that normal submission can enter that state.

The same review found that `remote::handle_disconnect` had no recovery call when neither automatic retry nor queued-continuation recovery was scheduled. `failed_prompt_disconnect_preserves_source_and_newer_draft_without_resending` first failed on the real handler's missing clipboard source. The minimal repair restores before pending-state cleanup and adds `Delivery unconfirmed` with advice to check the session before resending, without changing automatic retry/queue policy. The library-only regression then passed, exit 0. An intervening whole-package filtered invocation ran the test successfully but the build wrapper returned 97 for unrelated integration binaries with zero matching tests; rerunning with `--lib` removed that wrapper ambiguity. Same-session History refresh retention is also being verified so reconnect cannot erase the only UI recovery block. Final task completion, gates and archival wait for these review follow-ups.

### Final frozen implementation checks

The reachable deferred-submission regression shows that model-switch deferral keeps the new source in prepared state before assigning `last_submitted_input`. A stale error does not adopt or lose that prepared source. The earlier manually injected state did not justify a new request-ID journal; no speculative state was added.

Same-session and initial-bootstrap History now retain failed records, while switching between two known sessions discards them. The first-bootstrap regression reproduced loss (`bootstrap=false`, 0 recovery records instead of 1), then passed after retaining records before History clears the UI. Repeated History remains deduplicated. Terminal regressions cover both unsupported-model and actual HTTP400/`invalid_request_error` stopped-auto-poke paths, a newer draft and no queued retry. Both lowercase `p` and shifted `P` are independently observed through the real keyboard/clipboard capture fixture. Shifted `P` first failed, then passed after the one-line key match repair.

Frozen-patch task `3804173f93` finished exit 0 after every command below:

- `cargo fmt --all -- --check`.
- `scripts/dev_cargo.sh test --locked -p jcode-tui --lib failed_prompt_ -- --test-threads=1`: **21 passed**.
- `scripts/dev_cargo.sh test --offline --locked -p jcode-tui -- --test-threads=1 --skip test_prompt_entry_shimmer_color_moves_across_positions --skip right_fact_stack_uses_neutral_gray_except_for_context_usage`, through the isolated environment documented above: **2868 library tests and 7 integration tests passed**, 20 library tests and 1 doctest ignored, 1 library test filtered. A nested subprocess test also passed. No failures.
- `scripts/dev_cargo.sh clippy --locked -p jcode-tui --all-targets --all-features -- -D warnings`.
- `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode` and the private native acceptance script: **passed**. Final fixture `$JCODE_SCRATCH_DIR/failed-prompt-acceptance-vswlxdwx/result.json`; binary SHA-256 `9129f45896e28fe28b8687d6ce4523b3b307d24f57f51f764033081e47009c72`. Exactly one provider request and exact keyboard clipboard source, with the newer draft intact.
- `openspec validate preserve-failed-prompts --type change --strict` and `git diff --check`.

Unrelated configuration/MCP diff still matches the preflight fingerprint. The checkout remains on its pre-existing `main`; local `origin/HEAD` points to `dev`. No branch correction, worktree creation, publication or live-daemon reload was performed. The mandatory independent apply review is running against this frozen patch before verified task updates, scoped commits and native archival.

Terminal-operation preflight: local scoped commits are authorized and use the configured identity, but depend on independent review approval. Native archival/spec synchronization depends on those commits and all ten verified tasks. The new spec and dated archive destinations are unused, so neither operation will overwrite an existing capability or archive. Publication and live-daemon reload are not selected operations. The source diff fingerprint under review is `ef92712d675a9dff57b6a6cee4d08f704e111dae40257c585e1cb67af92260e7`; the index is empty and the implementation parent remains `efb6f3dc5`.

### Independent review iteration 1 and repaired candidate

Independent reviewer `herb` found a blocking gap: after prompt A fails with broadcast `Error { id: 0 }`, a delayed duplicate can consume prompt B's fallback. The previous broadcast regression only repeated the error before another submission. A public A/B submission regression was expanded. After correcting its runtime setup, task `539763te7o` reproduced the actual defect: B's saved source became `None` instead of `Some("new submission B  \\n")`.

The existing protocol documents an ordered `TurnStopped` before the terminal event. The real zero-ID live-turn failure emitter sends that stop and the same error message consecutively; other zero-ID emitters are unassociated control/invalid-request diagnostics. A single bounded, one-use App marker now binds that stop to the active request and expected diagnostic. Matching zero-ID errors consume it; unassociated zero-ID errors remain visible diagnostics without settling the current prompt. New sends, completions, disconnects and actual known-session switches clear it. Initial startup adoption retains an in-flight fence. No protocol change, text-cache deduplication or prompt/request journal was added.

The expanded regression now proves A failure, B submission, ignored delayed A error, then a genuine B failure with the **same diagnostic text** and a fresh stop signal. B's source and newer draft survive the duplicate; the genuine failure remains recoverable and copyable. Targeted checks passed 21 tests.

All fresh post-repair gates in task `735801x1rc` finished exit 0: formatting, 21 targeted regressions, full isolated serial package (**2868 library + 7 integration tests passed**, existing CI exclusions/ignored tests as above), all-target/all-feature Clippy, selfdev build, native acceptance, strict OpenSpec validation and diff checks. Native result: `$JCODE_SCRATCH_DIR/failed-prompt-acceptance-gtcil3dd/result.json`, binary SHA-256 `3f4be825d678ada356de70ecf721f8227b830cfe8ebe3cd70bf8e926dc4f5f10`. The actual daemon stop/zero-ID failure recovered and copied exactly, preserved the newer draft and made exactly one provider request. This supersedes the preceding candidate's final-gate identities.

Repaired source diff SHA-256: `4105b9382589d7205c0d3ebee7a91c4e56a5d5e68fecbb00691da9b85356e484`. Unrelated configuration/MCP fingerprint is unchanged. Independent reviewer `herb` approved the repaired candidate with **no remaining actionable findings**, after the fresh post-repair gates. The reviewer independently reran `cargo test -p jcode-tui failed_prompt_` (**21 passed**) and `git diff --check`, inspected the final diff, requirement coverage, gate output and native evidence, and confirmed that stale zero-ID errors cannot settle B while a genuine same-diagnostic B failure still recovers. No reviewer edits were made. All ten authoritative tasks are now checked from that evidence. Scoped implementation persistence and native archival follow this approval; final commit-range/archive inspection remains required before workflow completion. Remote publication requires separate authority; do not infer push permission from this apply selection. Continue authorized implementation and local verification before consolidating any remaining decisions.
