# Native tool interaction and routing

Status: proposal drafted and structurally validated, awaiting user approval. No implementation performed.

Read proposal.md for scope and approval choices, design.md for evidence and contracts, specs/ for observable requirements, and tasks.md for dependency-ordered implementation/verification.

## Definition validation (2026-09-23)

- `openspec validate native-tool-interaction-and-routing --strict --no-interactive`: passed.
- Local artifact audit: 12 unique requirements, 24 success/edge scenarios, 12 dependency-ordered tasks, referenced source/test paths and approval gate: passed.
- This checks the definition, not implementation. No TUI, provider, macOS, packaging or new Firecrawl integration acceptance has passed for this proposed change.

## Approval choices

1. Native `ask_user_question`, root interactive TUI only, no HTML previews or durable restart recovery in this change.
2. Explicit Firecrawl backend within webfetch, direct remains default; hosted mode is opt-in and allowlisted, read-only markdown, using optional installed CLI initially.
3. Capability-aware routing guidance for every custom tool family survives base-prompt replacement while preserving user files and stricter policies.
4. Repair bounded browser selection schema/scoping/validation before advertising it; document existing macOS computer behavior without adding platforms.

## Dependencies

Do not implement before approval. Coordinate Jev configuration guidance with active unified-jev-service-config and server-file overlap with fix-subscribe-lock-order-deadlock. Archived Jev proposals are historical intent, not proof of current behavior.
