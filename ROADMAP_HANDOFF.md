# jcode Port and Orchestration Roadmap Handoff

## Status and scope

This is the planning baseline for turning the installed jcode runtime into a Zentui-style, session-safe orchestration client. It is intentionally a handoff, not an implementation proposal. Each numbered block should become a separately reviewable OpenSpec proposal under `openspec/changes/` after its dependencies and acceptance evidence are confirmed.

Current machine state:

- jcode `v0.68.0 (fcf53909)` is installed at [`/home/nyaptor/dev/jcode`](/home/nyaptor/dev/jcode).
- [`/home/nyaptor/.jcode`](/home/nyaptor/.jcode) is a symlink to that directory.
- OpenSpec `1.4.1` was initialized in `/home/nyaptor/dev/jcode` with no tool adapters selected.
- No Pi system-prompt files were copied or modified.

## Reference points

Use these as read-only compatibility references while designing jcode equivalents:

- Pi runtime prompt builder: [`dist/core/system-prompt.d.ts`](/home/nyaptor/.local/share/pnpm/global/5/.pnpm/@earendil-works+pi-coding-agent@0.84.1_@modelcontextprotocol+sdk@1.29.0_zod@4.3.6__ws@8.18.3_zod@4.3.6/node_modules/@earendil-works/pi-coding-agent/dist/core/system-prompt.d.ts). It exposes a base prompt, replacement prompt, appended prompt, tool snippets, guidelines, and context inputs.
- Pi resource layering: [`dist/core/resource-loader.d.ts`](/home/nyaptor/.local/share/pnpm/global/5/.pnpm/@earendil-works+pi-coding-agent@0.84.1_@modelcontextprotocol+sdk@1.29.0_zod@4.3.6__ws@8.18.3_zod@4.3.6/node_modules/@earendil-works/pi-coding-agent/dist/core/resource-loader.d.ts), including `systemPrompt`, `appendSystemPrompt`, and override/source metadata.
- Pi hook contract: [`pi-agent-surface-registry.test.ts`](/home/nyaptor/dev/pi/harnesses/test/pi-agent-surface-registry.test.ts), especially `before_agent_start` prompt mutation.
- Pi agent append/replace semantics: [`pi-subagents README`](/home/nyaptor/dev/pi/agent/packages/pi-subagents/README.md) sections on `systemPromptMode: append|replace`.
- jcode prompt composition: [jcode `SYSTEM_PROMPT_CONFIG.md`](https://github.com/1jehuang/jcode/blob/master/docs/SYSTEM_PROMPT_CONFIG.md). Its documented layers include the base prompt, capability modules, self-development guidance, `AGENTS.md`, global/project overlays, tools, memory, and active skills. New sessions capture the prompt; existing sessions do not retroactively change.

The Pi references are comparison material only. Do not vendor, copy, or edit Pi prompt sources as part of this roadmap.

## Non-negotiable design principles

1. **Session isolation from day one.** Every session owns an immutable runtime profile snapshot: provider, model/deployment, effort/reasoning level, credentials/auth mode, system/append prompt layers, tool policy, MCP/extensions, cwd/worktree, environment, execution mode, timeouts, retry policy, and telemetry correlation IDs. A later settings change creates a new profile/version; it must not mutate a running session.
2. **Local, remote, and hybrid are explicit modes.** Local runs tools and agents on the workstation. Remote runs them in a provisioned cloud/CI environment. Hybrid keeps the UI/control plane local while execution, repositories, or selected tools run remotely. The mode and environment identity are displayed and recorded per session.
3. **Control plane before scale.** Establish one reliable task lifecycle and evidence model before dispatching many agents.
4. **Herdr hooks, not Shepherd.** Hook names, payloads, ordering, failure policy, and redaction must be a stable event contract.
5. **Every phase earns the next phase.** Advance only after the confidence gate for the preceding block is met.

## Proposal building blocks and order

### P0 — Provider and session runtime foundation

Define config schema and resolution for generic OpenAI-compatible providers, Azure AI Foundry v1 endpoints, GitHub Copilot-compatible access where available, Anthropic/Claude Code, and Codex. Add explicit model/deployment and effort mappings. Build a session profile snapshot and redacted diagnostics. Prove two concurrent sessions can use different providers, models, effort levels, prompts, tools, workspaces, and env vars without cross-talk.

**Gate:** deterministic profile digests; no credential leakage; concurrent isolation tests pass; provider/model/effort are visible in session status.

### P1 — Prompt compatibility layer

Map jcode’s base prompt, capability modules, `AGENTS.md`, global/project overlays, skills, memory, replacement prompts, and append prompts into a versioned prompt assembly contract. Add source attribution and prompt digest to the session profile. Preserve Pi’s append/replace behavior as a compatibility target without copying Pi files.

**Gate:** golden prompt snapshots for base, append, replace, project overlay, and hook augmentation; prompt changes affect only new sessions.

**Status (2026-08-09): CLOSED.** Proposal `add-prompt-assembly-contract` (commit `e98c4b0`) applied as commit `d990653`: versioned named-layer assembly (`PROMPT_ASSEMBLY_VERSION = 1`, fixed static order base → capability modules → selfdev → agents-md → prompt-overlay → preferred-tools → skills-list), SHA-256 digest `prompt:<version>:<hex16>` over version+layer ids+source kinds+contents (paths excluded), session freeze of static layers in both the TUI `App` and app-core `Agent` (enforcing the documented new-session semantics; dynamic layers memory/active-skill/reminder/swarm-directive rebuild per turn), attribution + digest via `ContextInfo` into `/context` and the server sessions debug payload, Pi append/replace mapping documented in `docs/PROMPT_ASSEMBLY.md` without vendoring. Gate evidence: 9 new contract goldens + 2 agent freeze tests, 43/43 jcode-base prompt suite (34 pre-existing unmodified = byte-identity proof), strict OpenSpec validation, full-suite parity vs clean-tree baselines (jcode-base 5=5, jcode-app-core 5=5, jcode-tui 16≤17 zero new). Archived as `2026-08-09-add-prompt-assembly-contract` (archive commit `c0db1a1`; specs merged into `openspec/specs/prompt-assembly/spec.md`, 6 requirements; `openspec validate --specs --strict` = 7 passed). **P1 CLOSED** (applied `d990653` + archived `c0db1a1` + handoff updated). P0's full session profile remains future work; this delivered its prompt portion.

### P2 — Basic Zentui port

Port the visual language first: layout primitives, panels, status bars, command palette, model/provider/session selectors, tool cards, diff/log views, and keyboard navigation. Keep rendering state separate from agent state. Do not introduce animation complexity until static states are correct and accessible at small terminal sizes.

**Gate:** deterministic snapshots, resize behavior, degraded-terminal behavior, and no event-loop stalls during streamed output.

**Status (2026-08-09): CLOSED.** Decomposed into three narrow, independently reviewable OpenSpec changes in `source/jcode/openspec/changes/`: `add-status-footer` (persistent starship-style bottom row), `add-composer-frame` (opencode-style accent rail + metadata row), and `add-user-message-framing` (framed/compact/labeled transcript prompt styles). All three applied with per-change gate evidence (deterministic snapshots at widths 60-160, packed + scrolling byte identity, ASCII variants, streaming benchmarks, full-suite baseline parity) and archived: footer `70319c0`, composer frame `1796428`, user-message framing `617eb15`, archive commit `ff3d9c5`. Other P2 surfaces (panels, pickers, tool cards, diff/log views, keyboard navigation) already exist in `crates/jcode-tui` and are covered by their existing test suites. P3 (motion and frame scheduler) is unblocked.

### P3 — Motion and frame scheduler

Introduce a centralized frame clock/scheduler for spinners, streamed token reveal, progress transitions, notifications, and shader-like effects that degrade to ANSI-safe motion. Animation must be pausable, bounded, and independent of provider latency. Record frame/render timings for telemetry.

**Gate:** stable frame budget under streaming/tool load; reduced-motion and non-color fallbacks; no starvation of input or agent events.

**Status (2026-08-09): CLOSED** as one OpenSpec change, `add-frame-clock`, applied and archived (`source/jcode/openspec/changes/archive/2026-08-09-add-frame-clock`, spec merged to `openspec/specs/frame-clock/spec.md`; implementation `9dcd285`, archive `71354a8`, viewport-clock adoption follow-up `8a07eab`). Delivered: `FrameClock` central time authority (epoch = app start, exact pause/resume exclusion, bounded saturating frame math; a real backwards-drift-during-pause bug was caught by the exact-freeze unit test and fixed before commit), all animation consumers (`animation_elapsed`, spinner frames, workspace ticks) ride the clock with zero call-site churn, focus loss/gain pauses/resumes the clock (no terminal-suspend path exists; focus events are the effective suspend signal), `FrameTimingRecorder` bounded ring exposed as `frame_timing` percentiles in the debug state JSON, and two new test gates: a frame-budget gate (streaming bursts + tool cycles through real `ui::draw`, p95 <= 250ms / max <= 1000ms absolute ceilings) and missed-tick starvation gates pinning `MissedTickBehavior::Skip` semantics on the redraw timer. Defaults preserve behavior byte-for-byte: the clock never pauses unless focus events arrive, and full-suite parity is 15 <= 17 baseline failures with zero new. Gate note: reduced-motion/non-color fallbacks are explicitly deferred to a follow-up change built on this clock (recorded as a non-goal in the design); the budget and no-starvation gates ship here. Docs: `docs/FRAME_CLOCK.md`.

### P4 — Basic tools and execution safety

Port the minimum tool surface: read, write, edit, grep/find, list, shell, diff, and cancellation. Add per-session tool allowlists, approval policy, command timeout, output truncation, and structured tool lifecycle events. Keep remote execution behind the same tool contract.

**Gate:** replayable tool traces, cancellation correctness, approval auditability, and safe cleanup after timeout/error.

### P5 — Herdr hooks and telemetry

Define Herdr lifecycle hooks for session, turn, provider request, tool call, dispatch, environment, worktree, artifact, PR, and workflow events. Add redaction, sampling, correlation IDs, and durable local event buffering. Telemetry must be session-scoped and must not include secrets or raw prompts by default.

**Gate:** event schema versioning, hook failure isolation, complete lifecycle traces, and useful latency/error/cost dashboards.

### P6 — Basic orchestration control plane

Implement one-task dispatch with a durable task state machine: queued, claimed, running, waiting, succeeded, failed, canceled, and orphaned. Add idempotency keys, retries, cancellation, heartbeats, artifact capture, and operator-visible evidence. Start with one local worker and one session profile.

**Gate:** crash/restart recovery, no duplicate side effects under retry, orphan detection, and measured completion/cleanup rates.

### P7 — Claude Code agent adapter

Dispatch Claude Code agents through a provider-neutral adapter while preserving the session profile, prompt contract, tool policy, Herdr events, and evidence model. Support append-system-prompt versus replacement semantics where the adapter allows it. Keep adapter-specific flags out of the core state machine.

**Gate:** repeatable task runs, correct prompt/profile capture, cancellation, exit classification, and artifact handoff.

### P8 — Cursor Cloud Agent adapter

Add remote Cursor Cloud Agent dispatch as a second adapter. Model remote job IDs, asynchronous status polling/webhooks, remote logs, environment identity, branch/artifact return, and failure recovery. The UI must make local versus remote execution obvious.

**Gate:** remote job recovery, duplicate-safe polling, environment cleanup, branch/artifact correctness, and parity with local task evidence.

### P9 — Worktree and workspace manager

Create isolated worktrees per task/agent with naming, locking, cleanup, base revision capture, patch/artifact export, and conflict diagnostics. Make worktree identity part of the immutable session profile and task record.

**Gate:** parallel tasks do not collide; cleanup is idempotent; failed runs retain enough evidence to reproduce.

### P10 — Remote environment manager

Provision and tear down remote execution environments with explicit images, secrets references, network policy, tool versions, and TTLs. Support local, remote, and hybrid profiles. Capture environment manifests and cost/latency data.

**Gate:** reproducible environment manifests, secret isolation, TTL cleanup, and bounded orphan cost.

### P11 — GitHub and Azure DevOps integrations

Add provider-neutral repository and PR abstractions, then GitHub and ADO implementations for branch creation, commits, push, PR creation/update, review status, checks/pipelines, comments, and merge policy. Keep credentials and PR metadata session/task scoped.

**Gate:** idempotent PR updates, correct repository/branch targeting, check-status reconciliation, and safe handling of permissions/rate limits.

### P12 — Basic workflow integration

Wire OpenSpec and Beads into the control plane and command surfaces: `explore`, `feature`, and `apply`. Each command should create/resolve a durable task record, preserve session profile and prompt evidence, and expose approval/validation gates. Use OpenSpec strict validation before handoff/archive.

**Gate:** one end-to-end local workflow with evidence, resumability, and no manual database edits.

### P13 — Advanced orchestration

Add multi-agent plans, dependency-aware waves, specialist routing, bounded fan-out, dynamic retries, human checkpoints, remote/local placement, cost budgets, and policy-aware scheduling. Claude Code and Cursor agents become interchangeable workers behind the same task contract.

**Gate:** comparative runs show improved throughput without regressions in correctness, isolation, orphan rate, or cost.

### P14 — Advanced workflow integration

Add `apply:all`, queue planning, dependency graphs, wave execution, selective re-runs, cross-task artifacts, PR batching, and final OpenSpec/Beads disposition. Require explicit dry-run and approval modes before mutating repositories or PRs.

**Gate:** full queue replay, partial failure recovery, audit-complete telemetry, and operator-visible final disposition.

## Metrics and promotion gates

Track at minimum: session-profile isolation violations (target zero), provider request success, time to first event, task completion, retry recovery, cancellation latency, tool approval/error rates, worktree collision/orphan rates, environment cleanup and cost, artifact/branch correctness, PR/check reconciliation, prompt digest reproducibility, Herdr event completeness, UI frame latency, and end-to-end operator recovery time.

Every proposal should define its metric baseline, target, rollback signal, fixtures/replays, and the evidence required to unlock the next proposal. No advanced orchestration or `apply:all` work should begin on anecdotal success alone.

## Proposal authoring checklist

For each block, create one OpenSpec change with:

- a narrow capability and explicit dependency IDs;
- session-isolation invariants and security/redaction rules;
- local/remote/hybrid behavior where applicable;
- Herdr events and telemetry fields;
- UI states, keyboard/error states, and reduced-motion behavior where applicable;
- acceptance scenarios for success, cancellation, retry, crash, and partial failure;
- strict validation: `openspec validate <change-id> --strict --no-interactive`;
- a measurable promotion gate and rollback plan.

The first proposals should be P0, P1, and P2 in that order. P3–P5 can then proceed in parallel only after the session profile and prompt contract are stable. P6 is the orchestration confidence point; P7–P11 build the agent, workspace, environment, and PR surface on top of it; P12–P14 connect and scale the OpenSpec/Beads workflow.
