# Tasks

Implementation was authorized by the subsequent apply selection. Completion is backed by the fresh verification and independent-review evidence in `research.md`. Groups run in order; each implementation task starts with a failing regression and finishes with its targeted check passing.

## 1. Lossless submission and composer recovery

- [x] 1.1 Add `failed_prompt_` regression tests using the existing TUI test app for unknown and unavailable skills, multiline paste expansion, preparation rejection and the existing input-size limit. Reuse prepared/last-submitted state in `input.rs` so complete expanded user text survives early returns and paste cleanup. Verify `cargo test -p jcode-tui failed_prompt_` passes these cases, with exact text equality and zero dispatches on pre-dispatch rejection.
- [x] 1.2 Update shared failed-input restoration in `model_context.rs` so an empty composer receives complete editable text and a newer draft remains unchanged without consuming the only recovery value. Verify targeted tests for both composer states and still-available attachment state; unavailable or deliberately stripped attachments must remain visibly identified rather than silently resent or omitted.
- [x] 1.3 Cover existing login/account/SSH interception and history exclusions before adjusting capture order. Verify sentinel input is absent from recovery/display/copy/persistence, slash and oversized conversational input stays excluded from persistent history, and recovery introduces no new prompt store or raw-prompt logging.

## 2. Full failed-prompt block and copying

- [x] 2.1 Reuse error/display machinery to present the associated diagnostic and a labeled, complete, inert failed-prompt block without adding a successful user turn. Add rendering regressions for nested fences, terminal-control text, Unicode, preserved whitespace and a prompt extending beyond the viewport/history limit. Verify full text remains inspectable, embedded content does not execute, and errors are not mistaken for successful or queued submissions.
- [x] 2.2 Connect a keyboard-accessible full-prompt copy target to existing clipboard handling, deriving its payload from retained source text rather than viewport cells. Update existing interaction/help text if an affordance changes. Verify the actual copy callback receives the exact complete prompt, with no error labels, fences, synthetic wrapping or placeholders; a failing callback must leave recovery and drafts intact and show failure feedback.

## 3. Remote association and integration boundaries

- [x] 3.1 Apply the same retained-text contract to prepared remote submission and remote error handling in `remote/input_dispatch.rs` and `remote/server_events.rs`. Preserve attempt association through pending-state cleanup and existing fallback/retry handling. Verify controlled send failure, provider/auth failure and post-dispatch disconnect tests expose the original prompt, retain a newer draft and correctly distinguish not-sent from uncertain delivery without changing retry policy.
- [x] 3.2 Add duplicate/delayed-event and successive-submission regressions using existing event IDs. Verify each failed attempt has at most one recovery record, older events cannot adopt newer input, and rendering, copying or restoring generates no additional provider request or user turn. Include a retry-budget-exhaustion path to ensure final cleanup cannot erase recovery.
- [x] 3.3 Extend existing successful plain-prompt and skill-with-image tests, including `skill_invocation_with_prompt_attaches_pending_image_to_user_message`, to verify single dispatch, correct attachment handling and no spurious failure block. Run the relevant existing fallback, remote-event and prompt-history tests, then `cargo test -p jcode-tui`; record any unrelated baseline failure without altering unrelated work.

## 4. Acceptance verification and handoff

- [x] 4.1 Run `cargo fmt --all -- --check`, `cargo test -p jcode-tui failed_prompt_`, `cargo test -p jcode-tui`, and `openspec validate preserve-failed-prompts --type change --strict`. Inspect the final scoped diff and confirm existing configuration/MCP work is preserved. Record commands, exit statuses and relevant failure causes, not just a build result.
- [x] 4.2 Use the repository's coordinated `selfdev` build and `debug_socket` verification route with an explicitly identified source-built binary and isolated test session. Exercise local unknown-skill rejection and controlled daemon-connected failure with a multiline pasted sentinel, then type a newer draft before a delayed error. Inspect the complete error block, invoke its keyboard copy path, compare the copy payload with the original text, and confirm a copy-backend failure does not lose recovery. Record binary/daemon identity and observed results in `research.md`; if runtime or clipboard observation is unavailable, mark that acceptance check blocked instead of claiming completion. Do not change live credentials, provider routing or the user's active session to stage a failure.

## Requirement coverage

| Requirement | Primary tasks | Acceptance evidence |
| --- | --- | --- |
| Lossless recovery across submission failures | 1.1, 3.1 | Exact expanded text survives each rejection/failure; pre-dispatch request count remains zero. |
| Full prompt block with lossless copying | 2.1, 2.2, 4.2 | Inspectable complete block and exact clipboard-boundary payload, including off-screen content and copy failure. |
| Non-destructive editing of failed input | 1.2, 3.1, 4.2 | Empty composer restores; occupied composer remains unchanged; failed text and attachment status remain available. |
| Submission identity and deliberate resubmission | 3.1, 3.2 | Correct request association, deduplicated errors, visible delivery ambiguity and unchanged dispatch counts/retry policy. |
| Safe rendering and privacy boundaries | 1.3, 2.1, 2.2 | Inert delimiter/control rendering, exact copy text and no secret/persistence regression. |
| Successful submission behavior remains unchanged | 3.3, 4.1 | Existing successful-turn, skill/image, history and fallback tests remain passing. |

Implementation, any later publication and archival remain separate operations. Do not mark this change implemented, archive it or sync its delta into main specs merely because planning validation passes.
