# jcode-jevy-browser-select — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. Browser select uses the evaluate tool to call Jev.

## 1. DOM snapshot enhancement

- [x] 1.1 Implement indexed DOM element snapshot: In `tool/browser.rs`, add `snapshot_dom_elements()` that queries all visible interactive elements, assigns sequential string indices, captures role/label/value/kind, enumerates select options with sub-indices. Produces `{page, elements, recent_actions}` format. Verify stable indices on known test page.
- [x] 1.2 Implement page fingerprinting for staleness detection: Lightweight fingerprint from hash of (URL + element count + index→role mapping). Compare before/after Jev call to detect page changes. Verify <1ms computation, stable when page unchanged.

## 2. Jev action selection

- [x] 2.1 Implement speculative fan-out request builder: `build_jev_select_request()` takes DOM snapshot + agent goal, constructs Jev request with operation (Choice across 8 operation types) + per-operation target heads (clickable/fillable/selectable elements only). Omits empty target heads. Verify request fits Jev token limits for pages up to ~200 elements.
- [x] 2.2 Implement Jev response resolver: `resolve_jev_select_response()` parses `{operation, target, confidence}`, resolves target index to DOM node, validates fingerprint + occlusion. On staleness: re-snapshot and retry once. On occlusion: scroll and retry once. Verify valid response resolves, stale page retries, missing element returns clear error.

## 3. Action execution and integration

- [x] 3.1 Implement jev_select action in browser tool: Add `action="jev_select"` handling: parse goal → snapshot → build request → evaluate → resolve → execute. Execute CLICK/TYPE_TEXT/SELECT/SCROLL/WAIT/DONE/BLOCKED appropriately. Return structured output. Verify agent can call jev_select in a loop.
- [x] 3.2 Implement text generation for TYPE_TEXT operations: When Jev selects TYPE_TEXT, route value generation to existing LLM with field context. Handle null/empty return as `requires_manual_value`. Verify appropriate text generation and graceful null handling.
- [x] 3.3 Implement fallback and error handling: Jev unavailable → clear error directing agent to manual actions. No interactable elements → Jev should return BLOCKED. Element index out of range → diagnostic error. Stale element after retry → error with fresh state. Tool description teaches loop pattern.
- [x] 3.4 Register and document jev_select: Add to browser tool action schema with goal field. Write tool description covering usage pattern, goal phrasing, operation types, loop pattern. Verify agent calls jev_select with natural-language goals on first attempt.

## Dependency graph

```
Task 1.1 → 1.2 → 2.1 → 2.2 → 3.1 → 3.2 → 3.3 → 3.4
```

Phase 1 (DOM snapshots) must complete before Phase 2 (Jev selection). Phase 3 (execution) depends on Phase 2.