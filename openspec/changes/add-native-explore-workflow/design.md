## Context

Claude contributes prior-art retrieval, decision-map mode, ranked output, and explicit feature handoff. Codex contributes uncertainty classification, source freshness, revision stamps, surface inventory, and single-default routing. Jcode provides native slash skills, `todo`, `memory`, session search, `swarm`, `initiative`, `side_panel`, structured repository tools, and telemetry.

The workflow must not require OpenSpec or Beads, but it must offer a one-time initialization path when either is absent. It must avoid spending model tokens on shell work that Jcode tools or structured CLI output can perform directly.

## Goals / Non-Goals

**Goals:**

- Create a native `/explore` skill with a stable phase and output contract.
- Use Jcode state and coordination primitives rather than importing another harness's orchestration.
- Detect and optionally initialize repository planning integrations with explicit consent.
- Preserve provenance for `/feature` without a duplicate durable ledger.
- Make telemetry and execution efficiency observable behavior.

**Non-Goals:**

- Implement product changes during exploration.
- Require OpenSpec, Beads, Recon, swarm, or telemetry for local-only exploration.
- Copy Claude preprocessors, Codex verification scripts, or repository-specific ceremony.
- Use shell commands when a purpose-built Jcode tool supplies the same result.

## Decisions

### 1. Ship a native skill named `explore`

The existing registry maps skill names to slash commands, so no new Rust slash-command variant is needed. Rejected: aliasing `codex-explore`, which preserves Codex ownership instead of establishing a Jcode contract.

### 2. Use a shared repository preflight

Resolve repository identity and check OpenSpec, Beads, and telemetry. Missing integrations produce one consent question. Repository-scoped Jcode state records accepted, declined, or unavailable status. Acceptance runs the canonical non-interactive initializer and rechecks readiness. Decline continues without the integration. The prompt repeats only after reset, repository change, or explicit setup request.

### 3. Use a fixed native phase sequence

1. Intent and success criteria.
2. Integration and telemetry preflight.
3. Session plan with `todo`.
4. Prior context from memory, sessions, repository guidance, active work, initiatives, and Recon.
5. Scoped evidence using native tools and optional read-only swarm.
6. Synthesis of facts, assumptions, conflicts, options, and decision frontier.
7. Ranked queue and one default route.
8. Structured `/feature` handoff or durable initiative checkpoint.

### 4. Keep durable state in authoritative homes

Session work stays in `todo`; cross-session decisions use `initiative`; repository decisions use the accepted planning integration; external research uses Recon. The side panel is a view, not authority.

### 5. Detect telemetry every run

Check the harness telemetry capability directly. When available, emit start, phase outcome, integration status, route, shell count, structured-tool count, and completion. Missing or failing telemetry never blocks exploration.

### 6. Apply a token-efficient execution ladder

Use already-injected context, memory/session search, `agentgrep`/`read`/`ls`, structured first-party CLI JSON, one batched bounded shell call, then optional focused swarm. Shell calls use direct argv where possible, explicit timeouts, narrow paths, source-side filters, and output caps. Broad scans, recursive dumps, repeated polling, and shell parsing when a typed tool exists are prohibited.

### 7. Produce a structured handoff

Include destination, success criteria, provenance, assumptions, alternatives, scope, decisions, surface inventory, confirmed revisions, dependencies, conflicts, edge cases, done means, remaining questions, and recommended action. `/feature` freshness-checks references rather than rediscovering them.

## Risks / Trade-offs

- **[Risk] Initialization prompts become nagging** → persist repository-scoped decisions and provide reset.
- **[Risk] Degraded mode hides missing ceremony** → include integration status and limitations.
- **[Risk] Native tools produce large output** → require scoped queries and source-side limits.
- **[Risk] Parallel exploration fragments context** → coordinator owns synthesis and workers return typed evidence.
- **[Risk] Telemetry changes behavior** → emission is best-effort and cannot control routing.
- **[Risk] Stale handoff reaches `/feature`** → include revisions and freshness-check on consumption.

## Migration Plan

1. Land the shared preflight and tests.
2. Add native `explore` and public slash acceptance.
3. Add decision-map, side-panel, and handoff behavior.
4. Add telemetry and shell-efficiency assertions.
5. Enable companion `/feature` consumption.
6. Roll back by disabling the skill; durable artifacts remain readable.

## Open Questions

None.
