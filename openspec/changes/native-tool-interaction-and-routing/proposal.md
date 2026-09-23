## Why

Jcode needs structured user decisions in its TUI, a richer public-page reader, and prompt guidance that matches its actual custom tools. Current stdin events do not provide a question UI. Native webfetch misses JS-populated content. Updated browser and computer capabilities are not consistently reflected in tool exposure or prompt routing.

## What Changes

- Add native `ask_user_question` for interactive root TUI sessions, with typed question/answer events, keyboard selection, free text, cancellation and reconnect handling.
- Extend `webfetch` with explicit `backend: direct|firecrawl`; keep direct as the backward-compatible default. Firecrawl is opt-in, public-HTTPS-only and read-only in this change.
- Add capability-aware tool-routing guidance covering every registered custom tool family, including `evaluate`, updated `browser`, and macOS computer use.
- Apply tool guidance after base-prompt replacement so global/project custom prompts still receive accurate capability guidance. Preserve user-owned prompt files and safety policies.
- Reconcile browser schema/implementation drift before advertising `jev_select`; preserve scoped targets and validate selected actions rather than trusting model output.

## Capabilities

### New Capabilities

- `structured-user-questions`: Native TUI question lifecycle and response correlation.
- `webfetch-backends`: Explicit direct and hosted page-reading semantics.
- `custom-tool-routing`: Capability-aware prompt coverage and browser exposure parity.

### Modified Capabilities

None. No baseline specs exist for these capability names in this checkout.

## Impact

Touches tool-core context, native registry, protocol, server session state, TUI input/rendering, webfetch configuration/transport and prompt composition. Browser parity repairs are bounded to currently implemented selection behavior, not the entire archived browser proposal. Existing API schemas must remain compatible when new optional fields are omitted. No automatic hosted requests, installation, login, shared-daemon restart or user prompt overwrite.

## Scope and actors

Actors: root agent, human TUI user, server, provider adapter, hosted Firecrawl, optional browser/Jev services. Include all registered custom tools in an audit inventory, but use concise family guidance rather than duplicate parameter schemas. Computer use remains macOS-only. Existing tools otherwise retain behavior.

## Non-goals

Desktop app UI; interactive stdin; question previews/HTML; nested-worker prompting; durable pending questions across daemon restart; automatic direct-to-cloud fallback; Firecrawl crawl/map/search/monitor/parse/interact; new desktop platforms; broad Jev architecture changes; rewriting personal skills or overlays.

## Dependencies and approval

Status: DRAFT, awaiting user approval. No implementation is authorized.
Proposed choices for approval: canonical question name, root-only availability, explicit Firecrawl backend with direct default, runtime tool guidance after prompt replacement, and bounded browser parity repairs.
The active `unified-jev-service-config` change owns evaluate/Review provider configuration. Do not duplicate or expand its scope here. Coordinate overlapping source edits with `fix-subscribe-lock-order-deadlock`; neither change is imported. `agentmail-ambient-channel` is independent.

## Acceptance evidence

Requirements and scenarios live in specs/. Tasks map every requirement to checks. Acceptance requires real isolated-server TUI/provider workflows as well as unit/protocol tests; source inspection alone does not establish behavior. Platform/provider tests unavailable locally must be explicitly blocked, not passed.
