## Context

Jcode's browser tool (`tool/browser.rs`) requires agents to manually select elements by CSS selector, coordinates, or accessibility snapshots. This creates multi-turn observation→selection→execution loops. `browser-use/jev-ultrafast` (7.9k stars, MIT) proves a single Jev call can replace an entire cycle: 7.1s for Google Flights search with 101 browser protocol calls (down from 1,092).

The evaluate tool (`add-jevsdk-evaluate-tool`) provides the Jev API integration layer this proposal depends on. The browser bridge already supports DOM access, element interaction, and page state queries.

## Goals / Non-Goals

**Goals:**
- Add `action="jev_select"` to browser tool with natural-language goal input
- DOM snapshot: indexed element table with role, label, value, kind
- Speculative fan-out Jev request: operation selection + per-operation target heads
- Execute resolved action on DOM node (CLICK, TYPE_TEXT, SELECT, SCROLL, etc.)
- Loop support: observe → choose → act → observe pattern
- Fallback: Jev unavailable → error, agent uses manual actions

**Non-Goals:**
- Not replacing existing click/type/snapshot/screenshot actions
- Not implementing full browser-use agent (jev_select is one action, agent orchestrates the loop)
- Not handling canvas, shadow roots, frames, or file uploads in initial scope
- Not implementing screenshot-based selection (Jev consumes structured state, not pixels)

## Decisions

**Speculative fan-out:** Instead of running two sequential Jev calls (select operation + select target), send both questions in one request. Jev sees all elements categorized by interaction kind. Only the target head matching the selected operation is consumed; unused heads are ignored. This eliminates one round-trip.

**Indexed element table:** Assign sequential string indices ("1", "2", ...) to DOM elements rather than using CSS selectors or XPaths. Indices are stable within a snapshot, simpler for Jev to reason about, and avoid selector fragility. Select options use sub-indices ("4:1", "4:2").

**Staleness detection:** Lightweight page fingerprint (hash of URL + element count + index→role mapping) computed before Jev call. Compare after. Stale → re-snapshot + retry once. Occluded element → scroll + retry once.

**TYPE_TEXT value generation:** Jev selects the field (which is fillable). Text LLM generates the value. Jev doesn't produce text; it only selects targets. This separation keeps Jev's responsibility narrow (selection) and LLM's broad (generation).

**Error model:** All failures return structured errors with enough context for the agent to recover. Staleness includes fresh page state. Occlusion includes element position. Agent can call jev_select again in a loop without manual recovery.

## Risks / Trade-offs

- **Risk:** DOM snapshot may not capture all interactable UI (shadow roots, canvas, dynamic content). Mitigation: documented as initial scope limitation; agent falls back to manual actions.
- **Risk:** Page changes between snapshot and execution cause stale-element errors. Mitigation: fingerprinting + single retry.
- **Risk:** Jev may hallucinate element indices. Mitigation: validate resolved node exists before acting.
- **Trade-off:** Fan-out request uses more Jev tokens than sequential calls. Justification: one round-trip vs two saves ~300ms latency, which matters more than marginal token cost.