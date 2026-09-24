## Decision

Use the existing native Jev capability, not an MCP subprocess. The model's function is classification, not tool execution. Reuse the prerequisite service resolver, endpoint/model policy, and credential handling. Do not copy Foreman's transport code or rely on Evaluate's currently inconsistent cache-hit/miss envelope. Expose a typed service result with identical hit/miss shape and strict validation for this caller. Any necessary shared extraction must preserve existing callers' public contracts with tests.

This is a proposed product decision awaiting approval. Mandatory MCP transport would require revising this boundary, including server selection and tool-schema negotiation, before implementation.

## Control and Integration Boundaries

The TUI remains the continuation owner. After capturing a turn-end snapshot, it proceeds with the existing scheduler immediately. Assessment is a nonblocking side operation. The daemon owns hosted calls and a per-session lease for connected clients. Local TUI uses the same assessment service directly. No new model recommendation may enqueue, suppress, or delay a turn.

Client requests carry session ID, turn ID, evidence revision, and a consent generation. Responses echo these identifiers. On new input, session switch, disconnect, cancellation, `/poke off`, or `/poke shadow off`, cancel the request and invalidate its generation. Reconnect starts disabled. Reject late responses even if network cancellation could not recall the billed request. A second attached client cannot acquire a second lease, and only the client holding the lease may receive its result. Lease revocation does not alter poke policy.

## Eligibility and Budgets

Only assess a completed TUI turn with shadow mode on, auto-poke enabled, nonempty todos, and no pending user message, permission wait, guardrail stop, or active known background work. Exclude overnight turns. Deterministically ineligible states produce a skip reason without inference.

At most one request per evidence revision, one in flight per session, and 20 launched requests per consent session. Re-enabling within the same session does not reset the request budget. No automatic retries. An overall two-second deadline includes queueing and provider time. Deadline failure yields an abstention, never changes poke behavior. Cache hits do not count as launched requests but still follow the single-result/revision rule.

## Evidence and Data Boundary

Use an allowlist: bounded current user request, relevant todo IDs/content/status, recent tool names and sanitized exit/result summaries, verification freshness, known background/permission state, and previous recommendation metadata. Exclude full transcripts, raw command arguments, raw stdout/stderr, environment values, file bodies, and credentials. Apply the existing secret redactor to all permitted text, cap the UTF-8-safe serialized payload at 8 KiB, and skip if required context cannot fit safely. Do not claim redaction guarantees anonymity. Opt-in disclosure must state that the remaining task text can still be private.

Treat text as untrusted evidence, never as instructions. Do not feed the agent's confidence labels as independent proof. Store only the latest result in session memory, not payloads or provider bodies in status or diagnostics.

## Result Contract

One choice question returns `continue`, `verify`, `replan`, `wait_for_user`, or `unknown`. This is a recommendation only. Parse only declared labels and finite probabilities in [0,1], with a complete distribution summing to one within 0.01. Missing or malformed answers abstain. Preserve the distribution and provider confidence separately. Provider confidence is concentration, not correctness.

Initially display a non-unknown recommendation only when its top probability is at least 0.80 and exceeds the second probability by at least 0.20. Otherwise return `unknown` with reason `uncertain`. These are display/abstention policy values, not calibrated correctness claims or continuation thresholds. Explicit provider failures use fixed sanitized reason codes. Status displays mode, provider/model, last result or skip reason, result revision, and remaining request budget.

## Alternatives and Risks

Deterministic wait detection is useful regardless of Jev and governs eligibility here. A full LLM supervisor and active intervention add authority not justified by existing evidence. MCP transport adds lifecycle and schema dependencies without an established requirement.

Main risks are evidence leakage, misleading probability interpretation, stale results, duplicate-client billing, and async work affecting scheduling. Scenario tests target each. Actual model quality requires independently labeled, consented samples. Shadow results alone cannot certify completion or justify enabling intervention.
