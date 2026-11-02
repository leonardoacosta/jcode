# Sandcastle mining by roadmap domain

Research date: 2026-11-02. Two agents were assigned per P01-P10 using `omni:gpt-6.1-sol`: A mined mechanisms; B challenged fit, failure modes and adoption assumptions. This is a source-backed candidate inventory, not implementation, acceptance or permission to run Sandcastle.

The first assignments used an incorrect route prefix and encountered provider errors; those were stopped. Named-provider routing was checked against `provider_key_for_spawn_model`, and the replacements used the `omni` provider. Some replacements also encountered transient Responses failures and required bounded retries. Report-file presence, successful tool calls and model labels do not establish runtime acceptance of the mechanisms described.

## Domain findings and existing owners

| Phase | Mechanisms worth carrying forward | Counterexample / guardrail | Existing proposal owners |
| --- | --- | --- | --- |
| P01 · Core/evidence | Typed plan validation; setup as a separately checked prerequisite; preserve per-worker failures | A commit, empty plan or completed review call is not acceptance evidence | `restrict-plan-command-execution`, `bind-acceptance-evidence-to-runtime`, `admit-judge-feedback-with-bounds` |
| P02 · Discovery | Canonical skill ownership with explicit harness projections and aliases; validate links and target content separately | Installed names and symlinks do not prove discoverability, compatible transport or trustworthy execution | `add-typed-skill-selector-metadata`, `show-skill-and-mcp-provenance`, `discover-native-and-mcp-capabilities` |
| P03 · Continuation | Validate structured work selection; carry reviewer context and individual attempt outcomes; account for nested attempts | Planner emptiness, commits and local loop limits cannot authorize continuation or prove completion | `add-structured-continuation-evidence`, `route-jev-supervision-explicitly`, `admit-semantic-poke-in-shadow` |
| P04 · Swarms | Explicit worker assignment, bounded concurrency and durable ownership; preserve disposal/retry evidence separately | A branch name is not ownership; a process-local semaphore is not durable lifecycle state | `add-swarm-handoff-contracts`, `add-swarm-file-claims`, `surface-worker-stall-and-block-reasons` |
| P05 · Durable work | Admission before dispatch; owned-worktree reuse checks; cancellation and resource accounting across attempts | Do not silently replace a missing cwd, refresh a dirty branch or infer authority from typed planner output | `configure-task-and-worktree-policy`, `admit-overnight-tasks-on-evidence`, `enforce-overnight-cancellation-and-limits`, `reconcile-overnight-runs-after-restart` |
| P06 · Providers/budgets | Explicit credential/environment precedence; supported/unsupported telemetry; per-attempt accounting | A context snapshot or best-effort stream log is not total billed usage; time/iteration caps are not spending caps | `classify-provider-auth-failures`, `reconcile-usage-budget-accounting`, `report-provider-doctor-coverage` |
| P07 · Memory/context | Treat session metadata and transcripts as distinct artifacts; typed handoffs; stable ownership through path remapping | No shell expansion from recalled context, automatic transcript export or assumed privacy merely because a container is used | `workspace-aware-context`, `add-attributable-memory-recall`, `add-compaction-quality-evidence`, `add-memory-retention-controls` |
| P08 · Integration | Structured result validation; cancellation distinct from disposal; observer failures distinct from policy-hook failures | Telemetry callback isolation must not suppress authority checks; SDK types alone do not prove protocol parity | `add-harness-sdk-contract-tests`, `add-structured-headless-review`, `add-lifecycle-hooks`, `add-browser-voice-boundary-acceptance` |
| P09 · TUI | Separate readiness from bootstrap; expose route identity and honest status; preserve context on resume | Setup is not permission to start work; no silent Git mutation on resume or hardcoded workflow roles as a picker design | `clarify-first-run-provider-setup`, `restore-resume-picker-redesign`, `improve-model-picker-browsing` |
| P10 · Distribution/setup | Explicit sandbox/setup boundaries, cancellation-aware cleanup, safe worktree reuse and attributable diagnostics | Do not copy another tool's worktree root, installer, environment propagation or refresh defaults without policy checks | `configure-task-and-worktree-policy`, `report-install-channel-recovery`, `record-resource-diagnostics`, `verify-selfdev-reload-identity` |

These are refinements to existing ownership, not 40 new feature commitments. No additional proposals are created solely because two miners described the same mechanism. A future implementation must add the relevant failure scenarios to its own authoritative `tasks.md` and specs rather than treating this research index as another execution ledger.

## Pair evidence

| Phase | Mechanism mining | Counterexample/fit mining |
| --- | --- | --- |
| P01 | [A](P01-A.md) | [B](P01-B.md) |
| P02 | [A](P02-A.md) | [B](P02-B.md) |
| P03 | [A](P03-A.md) | [B](P03-B.md) |
| P04 | [A](P04-A.md) | [B](P04-B.md) |
| P05 | [A](P05-A.md) | [B](P05-B.md) |
| P06 | [A](P06-A.md) | [B](P06-B.md) |
| P07 | [A](P07-A.md) | [B](P07-B.md) |
| P08 | [A](P08-A.md) | [B](P08-B.md) |
| P09 | [A](P09-A.md) | [B](P09-B.md) |
| P10 | [A](P10-A.md) | [B](P10-B.md) |

## Verification boundary

Reports identify public sources and local code/proposal references, distinguish older cached README evidence from newer fetched pages, and contain no raw private transcripts or credentials. The coordinator checked report structure, coverage, references and a representative set of claims against source. This is not an independent runtime verification of every cited mechanism or line range. The shared Jcode working tree changed during mining, so references are observations at their recorded snapshot rather than guarantees about future revisions.

For global policy, comparable tools and the distinction between a prior Sandcastle fetch and original-list admission, see [the config/triage research](../sandcastle-task-worktree-config-2026-11-02.md).
