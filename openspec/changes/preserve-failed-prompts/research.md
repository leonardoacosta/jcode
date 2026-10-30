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

Recheck source revision, dirty-target ownership, actual runtime/daemon identity and the relevant acceptance route before implementation. Artifact edits invalidate their bindings. No material decisions remain unresolved for proposal handoff. Historical incident attribution and future runtime acceptance remain unverified, not silently assumed.
