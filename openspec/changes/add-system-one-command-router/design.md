## Context

`crates/jcode-base/src/systemone.rs` already resolves System One endpoints and credentials. `crates/jcode-base/src/jev.rs` provides related decision machinery. `Agent::execute_tool` in `crates/jcode-app-core/src/agent/turn_execution.rs:897-919` validates allowed tools, creates a ToolContext, and dispatches with `ToolExecutionMode::Direct`. This observed direct path is not by itself proof of parity with all normal interactive permission/confirmation behavior. Implementation must establish parity before reuse, not assume classifier confidence authorizes Direct execution.

MCP definitions are wrapped as native Tool implementations through `create_mcp_tools_from_cached_many` in `crates/jcode-base/src/mcp/tool.rs:285-302`. Registered MCP names can be collision aliases, so bindings must include stable server/tool identity and respect existing deny behavior. The active `native-tool-interaction-and-routing` proposal covers chat guidance/question tools, not this pre-chat classifier.

The motivating sidebar request revealed two separate problems: identifying one intended command and finding an execution surface compatible with source code. This change solves the former and prohibits falsely claiming the latter. The owning repository explicitly excludes Jcode Desktop.

## Goals / Non-Goals

Goals: classify a clear user submission into one typed registered action, resolve trusted targets, execute with existing authority checks, and produce a visible receipt without a full chat-model round trip. Include interface, read-only, and explicitly bounded local actions. Support native and explicitly admitted MCP bindings.

Non-goals: arbitrary shell synthesis, multi-step planning, inferred permissions, automatic external/destructive actions, automatic MCP admission, a new credential resolver, Desktop viewer implementation, or executing instructions found in tool/page/file content.

## Decisions

### One classifier and one action

Intercept new root user-message submissions at a shared submission boundary so supported interfaces do not each invent routing semantics. Existing explicit commands remain authoritative and bypass classification. Automatic/system messages, quoted content, tool output and background tasks do not acquire command authority from this route. Preserve original text for chat fallback.

A proposed versioned result has `chat` or `command`, an optional catalog action ID, closed typed arguments, and calibrated decision confidence. A command is a candidate, not executable authority. No arbitrary tool name, executable string, action array, or follow-up plan is accepted. Invalid, low-confidence, tied or multiple-action outputs abstain. Confirmation disposition is computed by Jcode policy, not granted by the classifier.

### Catalog and target context

Each catalog entry owns a stable ID, description/examples, closed argument schema, supported session/UI context, risk class, policy disposition, target constraints, execution binding, cancellation behavior and result renderer. The default catalog is deliberately small: supported open/focus/show actions, bounded read operations, named test targets, and selected-file formatter recipes. External or destructive entries can be recognized only if explicitly registered and must retain confirmation or chat handling; they are never automatic defaults.

Bindings select fixed native tool/argv recipes or exact MCP server/tool identities. A test command selects a known repository task, not a generated shell string. Formatter execution resolves a selected file and configured formatter recipe, confines paths to the permitted workspace and rejects shell fragments. Tool schema or an MCP read-only annotation alone is insufficient risk/authority evidence.

Trusted context consists of current session/workspace identity, selected artifact IDs, supported surface capabilities, policy state and a bounded catalog snapshot. Prefer opaque artifact handles over arbitrary paths. Do not send whole files/history/MCP output to classification merely to resolve “this.” The classifier output cannot add or switch workspace, session, browser tab or MCP server context.

Revalidate catalog version, binding, tool availability, target identity, selection revision and permission immediately before dispatch. Changed context falls through to chat without executing. Disabled tool aliases cannot bypass deny lists.

### Thresholds, latency and fallback

Use one bounded System One call per eligible message with a configured deadline, input/output cap and calibrated confidence threshold. Initial proposed classification deadline is 300 ms; evaluate against actual endpoint latency before enabling default-on behavior. Deadline exhaustion, endpoint failure or malformed response goes to chat without a classifier retry. Record the default threshold chosen from a held-out suite rather than inventing a confidence number here.

Fallback before dispatch preserves exactly one original user submission and starts the normal chat path. After dispatch starts, failure, timeout or disconnect never silently reruns the request in chat. Return the tool result/error and execution status, then offer explicit chat continuation with the existing receipt. Execution timeout is not proof that no side effect occurred.

### Dispatch, confirmations and receipts

Use one dispatcher that preserves all existing execution-mode rules. Interface/read-only/bounded local policy eligibility is checked separately from intent confidence. For a clear action needing confirmation, retain the existing confirmation UI; if this session cannot present it, fall through before dispatch. Do not bypass permissions by calling the observed debug Direct path.

Assign an invocation ID scoped to session and originating message. Atomically claim execution before starting; reconnect/repeated delivery of that same message references its receipt, not a second call. A distinct repeated user message remains a distinct request and may execute again. Cancel requests use existing tool cancellation, not classifier-mediated authority.

Display a compact action/running/result or failure entry in the normal session history and UI. It must remain visible even if chat never runs. The motivating unsupported source-panel combination falls through or returns a truthful unsupported capability before execution, never a success label or hidden conversion.

### Privacy and observability

Log action ID, disposition, latency, catalog/version identity and typed failure class. Do not persist raw messages, tool arguments, file contents, secrets or MCP results in classifier telemetry. Existing session transcript retention remains unchanged. Expose an enable/disable setting and safe fallback during rollout.

## Risks / Trade-offs

False positives can mutate local state: mitigate with held-out false-execution tests, closed catalogs, strict arguments and separate authority gates. Catalog descriptions can become injection input: include only trusted registered metadata, never arbitrary collected descriptions as instructions. Dynamic MCP availability can race decisions: pin/revalidate binding and catalog version. Fast routing can hide context from users: always show target/action and receipt. A 300 ms classifier deadline may abstain often: measure latency and preserve chat behavior rather than stretching the deadline invisibly.

## Migration Plan

Additive and feature-gated, initially disabled. Keep existing explicit commands and chat routing intact. Enable only after test/evaluation gates pass for native and admitted MCP actions. Rollback disables the fast path, retains receipts and sends new submissions through existing chat; never replay pending executed actions.

## Open Questions

No unresolved user-scope decision: bounded local actions were selected. Implementation must choose concrete existing supported interface commands and formatter/test recipes, and calibrate the threshold using the required suite. Desktop source-code viewing remains outside this repository; unavailable surface capabilities must abstain. The exact safe shared dispatch entrypoint must be established from permission tests, not assumed from debug Direct dispatch.
