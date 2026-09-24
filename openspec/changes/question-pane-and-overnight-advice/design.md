# Design

## Evidence and diagnosis
- `crates/jcode-tui/src/tui/ui.rs`: `draw_inner` clears the screen then early-returns for questions. `draw_question_prompt_overlay` caps the box at 76x22 and joins labels/descriptions on one line. Structural: lost context. Surface: undifferentiated hierarchy and excessive blank space.
- The main renderer already allocates `input_height` and `chunks[7]` to the composer. Integrate there, not in a second top-level drawing path.
- `crates/jcode-tui/src/tui/mod.rs`: QuestionPromptState already tracks answers, review and Other editing.
- `crates/jcode-app-core/src/server/client_actions.rs`: pending requests are process-local, correlated and resolved under a lock. Extend this owner for deadlines and arbitration.
- `crates/jcode-app-core/src/server/client_lifecycle.rs`: forwarding currently belongs to a connection task. Move only pending-question ownership needed for live disconnect-safe deadlines into a session-scoped task.
- `crates/jcode-overnight-core/src/lib.rs`: persisted manifest, events and run status already exist. `last_activity_at` is run activity, NOT a user-idle clock.
- `crates/jcode-app-core/src/tool/evaluate.rs`: native Jev evaluation/cache exists. Do not silently substitute it for the user-requested MCP provider. A fallback needs a separately approved configuration and truthful provenance.
- No root DESIGN.md found. Use existing terminal theme styles, not browser fonts or a new palette.
- A live `Jev_jev_decide` call comparing layouts failed with HTTP 404 on 2026-09-24. No Jev verdict exists for this proposal. MCP availability is an implementation/integration gate, not a reason to block the UI work.

## Alternatives
1. Composer pane (recommended by Jcode): preserves reading context, reuses layout, works at narrow widths. Long content needs scrolling.
2. True overlay: fixes blank background with less layout change, but still hides relevant conversation.
3. Right sidebar: useful on wide terminals, poor at 80 columns and duplicates responsive layout work.

## Before / after wireframe
Before, from screenshot and source:
```
[ conversation absent ]
+ Agent asks --------------------------------------+
| Stored scope (2/2)                               |
| May stored mode expose retention differences?    |
| > [ ] Allow existing variance  Report each        |
| source's real storage kind, retention, freshness |
| [ ] Require uniform history  Do not proceed ...  |
|                                                  |
| Up/Down Space o Enter Esc                        |
|                  unused space                    |
+--------------------------------------------------+
```
After, illustrative pane (Jev status shown without fabricated advice):
```
  Assistant: Here are the source retention differences...
  [ previous messages remain visible and scrollable ]

+ Decision 2 of 2 · Stored scope -------------------+
| May stored mode expose each source's existing    |
| retention differences without expanding history? |
|                                                  |
| > ( ) Allow existing variance                    |
|       Report actual retention, freshness, gaps.  |
|   ( ) Require uniform history                    |
|       Expand or standardize stored history.      |
|                                                  |
| Jev: unavailable · manual answer required        |
| Up/Down move · Space select · o Other             |
| Left back · Enter review · Esc cancel             |
+--------------------------------------------------+
  Overnight · auto-answer on · disable
```

## Presentation and interaction
Single-choice radios, multi-choice checkboxes, bold labels and indented descriptions. Focus is distinct from selection. Keep all descriptions accessible, not hover-only. Pane height follows wrapped content up to half the usable viewport when at least six transcript rows can remain. Otherwise use the available viewport with scrollable body and pinned heading/footer. At tiny sizes show a compact resize/scroll hint rather than panic or draw outside bounds. Existing theme supplies muted text, focus and borders. No animation requirement.

Preserve draft bytes, cursor, selection and transcript scroll anchor. Tab switches focus between transcript and question. Input routed to transcript scroll must not mutate answers. Left goes back only outside Other editing. Existing review/atomic submission remains. A dedicated explicit key accepts Jev's suggestion, never automatic focus/preselection in manual mode. Display exact key in contextual footer. Cancel restores draft, not a generated answer.

## Advisory contract
Use configured MCP dispatch and discovered typed Jev contract, not hard-coded server URL/tool alias. One bounded request per immutable question batch revision, no retries merely to obtain a preferred answer. Supply option IDs/descriptions, bounded relevant context and explicit user constraints. Treat supplied content as data. Exclude credentials and unrelated transcript. Enable only with configured remote-evaluation permission. A missing permission or provider shows advice unavailable, manual choices remain immediate.

Normalized recommendation contains request/revision/context hash, option IDs, probability distribution, model/provider identity, evaluation timestamp and decision status. Support abstention/investigate/ask-user. Do not synthesize a free-form Jev rationale from probabilities. Show confidence as a model estimate, not proof. Any explanation authored by Jcode must be labeled separately. A 15-second advisory deadline exposes timeout state. Discard late/stale responses after mutation or resolution. Multi-select stays manual in v1.

## Overnight policy and clock
New overnight runs enable eligible auto-answering implicitly. No enable flag or separate confirmation is required. Show visible status at launch and in the pane, with an immediate disable control and explicit `--no-auto-answer-questions` launch override. Persist the resolved enabled flag, policy version, thresholds and mission-derived delegation scope. Missing policy fields in old manifests remain disabled; do not retroactively arm existing runs. Only a Running, authenticated matching coordinator run before target_wake_at qualifies. Children do not inherit permission in v1. Overnight status supplies the default only for newly initialized runs; all other eligibility checks still apply.

Delegation scope comes from the user's mission in run setup, not model-provided eligibility text. V1 limits automatic choices to reversible local implementation decisions within that scope. Consent, purchase, outbound communication, destructive actions, permissions, credentials, security/privacy/retention changes and uncertain classification require a human. Existing action-level authorization continues independently. Jev cannot determine consent. Ambiguous safety checks fail closed.

Server owns monotonic deadline: 300 seconds from eligible advice availability or latest authenticated user interaction in that session, whichever is later. Model output, tool events and background activity do not reset it. Keyboard, paste, mouse and answer editing do reset it. Any answer selection/edit switches the batch to manual-only to avoid submitting abandoned partial work. Persisted user activity and run activity are distinct. Countdown reflects server deadline. Disable, cancel, run completion and target wake disarm it. If detached, an already armed request may resolve under explicit policy. Do not broaden headless tool availability.

Automatic submission is atomic only when every question in a batch is single-choice, unanswered, eligible and has current advice meeting p>=0.90 and top-two margin>=0.20. Otherwise entire batch waits and displays why. Jev abstention, invalid distribution or unknown option IDs block. Old clients without activity/countdown capability disable auto-answer while attached. Reattachment broadcasts current state. Process restart marks pending requests interrupted and never catches up an expired timer.

## Prompt opt-out resolution
Resolve the policy before arming any deadline using the authenticated user's launch prompt and later direct instructions. “Do not auto-answer”, “wait for my input”, and “ask me before deciding” disable automatic answers for the run. A clearly question-specific instruction blocks that question; uncertain scope disables the whole run. A semantic resolver may interpret wording but must return a structured disable/no-preference/uncertain result plus an exact supporting span. It must never claim user consent from a probability. Resolver failure, timeout, malformed spans or ambiguity disable automation. No-preference leaves the new-run default enabled.

Only direct user instructions can alter preference. Quoted examples, repository text, fetched pages, tool results and agent-authored messages cannot enable or disable it. Explicit disable controls take precedence. Opt-out is sticky across reconnect: a later vague “continue” cannot re-enable it. A later explicit user re-enable may do so after normal policy checks. An arriving direct user message immediately suspends timers until preference resolution finishes, preventing a deadline from racing a new opt-out. Persist source, supporting span (bounded/redacted), resolved state and timestamp, and show why automation is off. Record default enablement as `overnight_default`, never as explicit consent.

## Exactly-once and audit
Manual submit, cancel and timer share the existing server lock and validation path. First valid resolution wins. Never hold that lock over network evaluation. Before delivering an automatic result durably record its resolution with unique request identity, question revision, run/session/tool IDs, question/option labels, selected IDs, evaluation provenance, confidence, policy, timestamps and reason `overnight_idle_timeout`. If audit write fails, do not answer. Recovery treats a recorded but undelivered resolution as interrupted, not a second response. Escape control sequences in terminal output and HTML-escape review values.

Tool results identify `answered_by: jev_auto` versus `user` and explain delegated scope. Append a deterministic transcript notice. Morning, cancelled and failed-run summaries list each automatic decision, count, reason and links to full records, plus blocked/interrupted questions. Do not rely on the LLM remembering to mention them. Records use existing run retention, no new telemetry. A later user correction is new input, not reversal of already executed work.

## Delivery boundaries
A: pane only. B: advice after A and MCP contract verification. C: overnight policy/audit after B and explicit approval of delegation rules. C does not delay shipping A. All stages approved on 2026-09-24 with the implicit overnight default described above.
