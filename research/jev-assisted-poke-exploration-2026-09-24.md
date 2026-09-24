# Jev-assisted poke exploration

Date: 2026-09-24. Status: recommendation only, not an approved implementation change.

## Intent and scope

Assess whether Jev MCP can improve Jcode's TUI `/poke` and automatic continuation. Desired outcome: fewer useless continuation turns, fewer premature stops, and better recognition of user blockers. Do not replace evidence-based verification with a model verdict.

Interpretation: “jev-mcp” refers provisionally to TypeSafe's Jev exposed through `itsmostafa/typesafe-mcp` (`evaluate`), the MCP alternative named in this repository's prior work. No Jev MCP tool is exposed in this session. The exact server intended by the user remains unconfirmed.

## Current evidence

- `crates/jcode-tui/src/tui/app/input.rs:1541`: turn-end scheduling runs the guardrail circuit breaker before normal poke and overnight continuation.
- `input.rs:1596`: normal poke guards pending turns and queued work, loads todos, and performs long-session review, deferred gate digest, delivery ownership, and completion-confidence checks.
- `input.rs:1746`: incomplete-todo deduplication fingerprints serialized todo records. Identical records suppress another poke, even if actual tool activity changed. Changes to records do not necessarily demonstrate real progress.
- `crates/jcode-base/src/todo.rs:638`: incomplete-work prompt is count-based: “You have N incomplete todos. Continue working, or update the todo tool.” Completion prompts already name relevant todos, so targeted validation is not a wholly new capability.
- `crates/jcode-tui/src/tui/app/commands.rs:2699`: completion confidence comes from todo fields and their history, not independent inspection of test results.
- Existing regression coverage includes unchanged-todo suppression, rearming, queue behavior, remote-session goal lookup, reload, confidence spikes, and bounded completion-gate retries. See `tests/remote_events_reload_05.rs` and `tests/state_model_poke_03.rs`. These tests were inspected, not executed during exploration.
- `crates/jcode-app-core/src/tool/evaluate.rs`: native TypeSafe/OpenRouter evaluator already exists with typed noul/choice/score questions and caching. It uses a 10-second request timeout and retries. This is too long to assume suitable for a turn-end UI operation without an overall deadline.
- `crates/jcode-app-core/src/agent/foreman.rs`: related passive worker-observer implementation asks about progress, looping, verification, and human intervention. It is disabled by default and explicitly does not intervene. Runtime activation was not established in this exploration.
- No `auto_poke` or `build_auto_poke_message` matches were found in app-core source. The identified scheduling owner is the TUI, including its remote-client flow. Do not assume headless daemon ownership or equivalent headless behavior.

Working tree already contains extensive unrelated changes, including relevant evaluator, foreman, and command files. Findings describe that working tree, not a clean release. No application source was edited.

## Prior art and memory

The archived `add-jevsdk-evaluate-tool` change records native and MCP alternatives. Archived Jev foreman work and `research/jev-ecosystem-deep-research.md` provide related ideas, not proof of runtime behavior or measured performance.

Active `openspec/changes/unified-jev-service-config/` proposes a common provider/model/credential resolver for Evaluate and Review screening. Its task list is unchecked, and no `JevService`/`jev_service` implementation was located. A poke integration should coordinate with this work rather than create another resolver. Adding poke as a consumer requires explicit scope/dependency treatment, not quietly expanding the current proposal.

Memory search for poke/Jev found no relevant entries. Session-history retrieval was withheld for excessive output and was not used as evidence.

## External documentation and Context7

Context7 resolution for `jev-mcp` returned Jev Feels, Jev Ultrafast, and unrelated MCP libraries, not the requested server. No exact version-relevant Context7 documentation was available. Do not substitute a browser agent's contract for this server's contract.

Read upstream documentation directly on 2026-09-24:

- https://github.com/itsmostafa/typesafe-mcp
- https://raw.githubusercontent.com/itsmostafa/typesafe-mcp/main/README.md
- https://raw.githubusercontent.com/itsmostafa/typesafe-mcp/main/docs/tool-reference.md

The MCP exposes `evaluate`, which accepts evidence and narrow typed questions. `noul` returns yes/no probability. `choice` returns an option, probabilities, and a confidence statistic. Confidence measures concentration of the distribution, not correctness or the winning option's probability. `min_confidence` is MCP-side abstention behavior, not an API feature. Native Jcode's inspected schema does not expose this option, so an internal caller must implement its own explicit abstention policy.

Upstream documents a 60-second timeout and retries. Its typical sub-half-second latency is a vendor claim, not a measurement here. No inference calls, credential checks, installs, or session-data uploads were performed. Documentation was read from mutable main, not a pinned release.

## Recommendation

Use Jev as an optional continuation adviser beneath deterministic safety rules. Keep policy and prompt wording in Rust. Prefer the existing native Jev capability through a shared service over adding an MCP subprocess solely for poke. If MCP transport is an explicit requirement, adapt the configured MCP evaluator behind the same policy boundary.

Start with opt-in shadow mode: evaluate snapshots and record what would have happened while preserving current behavior. Only enable interventions after labeled replay establishes useful accuracy and acceptable latency/cost.

Candidate classifications and eventual bounded actions:

| Observed situation | Candidate action |
| --- | --- |
| Unfinished work with a supported next step | Existing continuation, optionally targeted to a todo |
| Completion claim lacks observed verification | Targeted verification reminder, without inventing a command |
| Repeating failure without new evidence | At most one diagnostic/replan reminder, then pause |
| Awaiting user decision, permission, or credentials | Pause visibly, preserving incomplete todos |
| Background task or worker still running | Wait for the existing completion event, not another poke |
| Evidence is ambiguous or unavailable | Abstain and retain the existing guarded policy |

Do not let Jev mark todos complete, promote confidence, authorize tools, override `/poke off`, reset budgets, or bypass a refusal. “Stop poking” is not “task succeeded.” Initially exclude no-todo turns, overnight policy changes, and automatic completion verdicts.

## Evidence snapshot and ownership

A bounded snapshot should contain the current user request/scope, relevant todo IDs and statuses, recent actual tool outcomes, verification results with freshness, background-task state, explicit user-input waits, and previous continuation outcomes. Avoid feeding only the agent's summary or its confidence labels back to a second model.

Use deterministic known facts before inference: explicit cancellation, queued user work, active background work, provider guardrail stops, and permission waits. Jev helps only with remaining semantic ambiguity.

Keep one continuation owner. Since current scheduling is synchronous TUI code and evidence/tool services live in app-core, define an asynchronous assessment request/result contract rather than making a blocking HTTP call from the scheduler. Preserve TUI ownership initially. Any later daemon-owned controller is a separate migration.

Tag assessments with session ID, turn ID, and evidence revision. Discard results after new input, `/poke off`, cancellation, session switch, or revision change. Coalesce repeated assessments of the same snapshot. Do not run native and MCP evaluation for the same decision.

## Adversarial findings

1. **Confidently wrong classifications:** model confidence is not proof. Use abstention and measured thresholds, not an arbitrary score promoted to a completion gate.
2. **Circular assessment:** self-reported “verified” is not independent evidence. Supply observed results and preserve deterministic validation requirements.
3. **Privacy and instruction injection:** session/tool output can contain secrets or hostile instructions. Minimize and redact before hosted evaluation, treat supplied content as data, and require explicit opt-in for this new automatic data flow.
4. **UI latency and provider outage:** impose a short overall deadline, bounded request budget, cancellation, and sanitized fallback reason. Invalid, missing, or out-of-range output is not a valid vote.
5. **Stale results and duplicate clients:** one owner and revision checks must prevent a late result from scheduling an extra turn. Reconnect and reload must not multiply nudges.
6. **False progress:** todo text/confidence churn should not reset a semantic stall budget. Evidence changes, not formatting changes, should distinguish fresh attempts.
7. **Existing integration hazards:** native Evaluate returns the full response on a miss but stores/returns only the answers map on a hit. Normalize that contract before depending on it. Foreman also selects an OpenRouter credential but always posts to TypeSafe, so copy its question concepts, not its transport code. These are source-inspection findings, not reproduced failures.
8. **Overlapping gates:** ownership, confidence, deferred checks, and final-response reminders already exist. An adviser must not create another independent nudge stream or turn uncertainty into repeated expensive checks.

## Alternatives

- **Deterministic-only improvements:** recognize explicit waits/background tasks and improve progress fingerprints. Lowest operational cost and essential regardless of Jev. Cannot resolve all semantic ambiguity.
- **Native Jev adviser (recommended):** reuse existing integration and planned resolver. Adds hosted judgment cost and requires new async lifecycle/evidence plumbing.
- **MCP-backed adviser:** useful for an existing configured server or transport standardization. Adds process/tool availability, schema negotiation, and timeout concerns. Not automatically an improvement over native calls.
- **Full LLM supervisor:** can produce richer diagnoses but duplicates more work and generally costs more. Not justified as the first experiment.

## User-only actions, decisions, and open questions

- Confirm the intended server if it is not `itsmostafa/typesafe-mcp`, and whether MCP transport is required or Jev-backed behavior is the actual goal.
- Decide whether the primary pain is premature stopping, redundant nudges, or failure to notice blockers. Default recommendation prioritizes blockers and redundant nudges.
- Before live shadow evaluation, approve the provider/data boundary and cost budget. Provision credentials only if necessary, never print them or write them into repository files. No credential setup is needed for fixture-based development.
- Decide how existing manual `/poke` should interact with advisory pauses. Explicit user actions should not silently become model vetoes.
- Poke's headless behavior and multiple attached-client semantics need further tracing if daemon-wide operation is desired.

## Proposed next scope and verification

Route to feature authoring only after accepting the recommendation. First bounded feature: opt-in shadow assessment at eligible TUI turn boundaries, a normalized evaluator adapter, minimized evidence snapshot, local decision metadata, and labeled replay fixtures. No automatic intervention in that first feature.

Test categories: useful progress with unchanged todos, todo churn without progress, missing verification, explicit user blockers, active workers, provider refusals, absent credentials, malformed answers, deadline expiry, low confidence, stale response races, `/poke off`, reload/reconnect, and native cache hit/miss parity.

Compare against current behavior using unnecessary-poke rate, missed useful-continuation rate, missed blocker rate, added p50/p95 latency, request cost, and budget adherence. Establish thresholds from labeled results before enabling active decisions. Unit fixtures prove mechanics, not model quality. Live opt-in shadow evidence is needed for that claim.

For eventual runtime verification, use an isolated tester/socket running the changed binary. A successful build alone does not update the shared daemon. This exploration made no runtime improvement claim and ran no application tests.
