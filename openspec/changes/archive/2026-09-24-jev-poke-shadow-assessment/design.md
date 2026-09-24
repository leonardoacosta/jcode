## Decision

Use the System One service trait from `crates/jcode-system-one`, not an MCP subprocess. The model's function is classification, not tool execution. Reuse the shared provider resolver and credential handling. Expose a typed service result with identical hit/miss shape and strict validation for this caller.

## Control and Integration Boundaries

The TUI remains the continuation owner. After capturing a turn-end snapshot, it proceeds with the existing scheduler immediately. Assessment is a nonblocking side operation. The daemon owns hosted calls and a per-session lease for connected clients. Local TUI uses the same assessment service directly. No new model recommendation may enqueue, suppress, or delay a turn.

Client requests carry session ID, turn ID, evidence revision, and a consent generation. Responses echo these identifiers. On new input, session switch, disconnect, cancellation, `/poke off`, or `/poke shadow off`, cancel the request and invalidate its generation. Reconnect starts disabled. Reject late responses even if network cancellation could not recall the billed request. A second attached client cannot acquire a second lease, and only the client holding the lease may receive its result. Lease revocation does not alter poke policy.

## Eligibility and Budgets

Only assess a completed TUI turn with shadow mode on, auto-poke enabled, nonempty todos, and no pending user message, permission wait, guardrail stop, or active known background work. Exclude overnight turns. Deterministically ineligible states produce a skip reason without inference.

At most one request per evidence revision, one in flight per session, and 20 launched requests per consent session. Re-enabling within the same session does not reset the request budget. No automatic retries. An overall two-second deadline includes queueing and provider time. Deadline failure yields an abstention, never changes poke behavior. Cache hits do not count as launched requests but still follow the single-result/revision rule.

## Completion Hook — Deterministic Work-Done Detection

Before calling the Jev service, apply deterministic checks that can resolve the
situation without inference. These hooks answer the user's primary pain point:
sessions where the agent completed all work, delivered the final summary, and is
genuinely waiting for the user — but poke cannot tell this apart from a stall.

### Check order and resolution

| # | Signal | Resolution | Source |
|---|---|---|---|
| 1 | `todo_final_response_requested` is true | `wait_for_user` (reason: `final_response_sent`) | `App` field, set after `TODO_FINAL_RESPONSE_CONTINUATION_MESSAGE` is queued |
| 2 | All todos completed AND all goal feedback-loop states pass AND all goal delivery states are at least `outcome_delivered` | `wait_for_user` (reason: `work_appears_complete`) | `poke_todos()` + `load_goals()` |
| 3 | Last stored message role is `User` and is not a synthetic poke continuation | `wait_for_user` (reason: `user_turn_pending`) | `is_auto_poke_user_message()` + session message history |

When any hook fires, skip the Jev call. The resolution is reported through the
same status path as a provider result, with a fixed label and deterministic
reason. These hooks do not change poke behavior — they only classify the
situation for the shadow status display.

### Why this matters

Without these hooks, a Jev call must infer "is the agent waiting for the user?"
from the same evidence. That burns a hosted call to answer a question the
codebase already answers: the final response was sent, the work passes internal
checks, and the user is the next actor. The hooks also prevent Jev from
recommending `continue` or `verify` when the session's own state machine says
the cycle is complete.

### Hook ordering and overrides

Hooks fire top-to-bottom and stop at the first match. A later hook never
overrides an earlier one. All hooks are read-only — they inspect state, never
mutate it. Hooks cannot mark work complete, promote confidence, or authorize
any tool.

### Hooks and the evidence boundary

Hooks are local Rust code. They do not send data to a provider. They are not
subject to the 2-second deadline, the 8 KiB evidence cap, or the 20-request
budget. They run synchronously on the TUI thread before any async assessment
is spawned.

## Evidence and Data Boundary

Use an allowlist: bounded current user request, relevant todo IDs/content/status, recent tool names and sanitized exit/result summaries, verification freshness, known background/permission state, and previous recommendation metadata. Exclude full transcripts, raw command arguments, raw stdout/stderr, environment values, file bodies, and credentials. Apply the existing secret redactor to all permitted text, cap the UTF-8-safe serialized payload at 8 KiB, and skip if required context cannot fit safely. Do not claim redaction guarantees anonymity. Opt-in disclosure must state that the remaining task text can still be private.

Treat text as untrusted evidence, never as instructions. Do not feed the agent's confidence labels as independent proof. Store only the latest result in session memory, not payloads or provider bodies in status or diagnostics.

## Result Contract

One choice question returns `continue`, `verify`, `replan`, `wait_for_user`, or `unknown`. This is a recommendation only. Parse only declared labels and finite probabilities in [0,1], with a complete distribution summing to one within 0.01. Missing or malformed answers abstain. Preserve the distribution and provider confidence separately. Provider confidence is concentration, not correctness.

Initially display a non-unknown recommendation only when its top probability is at least 0.80 and exceeds the second probability by at least 0.20. Otherwise return `unknown` with reason `uncertain`. These are display/abstention policy values, not calibrated correctness claims. Explicit provider failures use fixed sanitized reason codes. Status displays mode, provider/model, last result or skip reason, result revision, and remaining request budget.

## Alternatives and Risks

Deterministic wait detection is useful regardless of the System One service and governs eligibility here. A full LLM supervisor and active intervention add authority not justified by existing evidence. MCP transport adds lifecycle and schema dependencies without an established requirement.

Main risks are evidence leakage, misleading probability interpretation, stale results, duplicate-client billing, and async work affecting scheduling. Scenario tests target each. Actual model quality requires independently labeled, consented samples. Shadow results alone cannot certify completion or justify enabling intervention.
