---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-browser-select

## Summary

Add `action="jev_select"` to Jcode's browser tool. Given a natural-language goal, the browser snapshots the DOM into an indexed element table, sends it to TypeSafe Jev in one request for both operation selection (CLICK, TYPE_TEXT, SELECT, etc.) and target element selection, then executes the chosen action on the resolved DOM node. The architecture is adapted from `browser-use/jev-ultrafast` (7.9k stars, MIT).

## Motivation

Jcode's browser tool (`tool/browser.rs`) requires the agent to manually select elements by CSS selector, coordinates, or accessibility snapshots. This creates multi-turn "find the right element" loops: the agent observes the page, guesses a selector, clicks, observes again, corrects, etc. jev-ultrafast proves that a single Jev call can replace an entire observation→selection→execution cycle: 7.1s for a real Google Flights search with 101 browser protocol calls (down from 1,092). The architecture reduces browser round trips by an order of magnitude.

## Actors

- **Jcode agent**: calls `browser(action="jev_select", goal="...")` with a natural-language instruction
- **Browser bridge** (Firefox/Chrome): produces DOM snapshots, executes actions
- **TypeSafe Jev**: selects operation + target element from structured state
- **Text LLM** (existing): generates text only for TYPE_TEXT operations (Jev selects the field, LLM writes the value)

## Scope

- New `action="jev_select"` in `tool/browser.rs`
- DOM snapshot enhancement: produce indexed element table with role, label, value, kind
- Jev integration: construct speculative fan-out request (operation head + per-operation target heads)
- Action execution: resolve Jev's choice to a DOM node, validate freshness, perform action
- Text generation: for TYPE_TEXT, route to existing LLM for value generation
- Fallback: if Jev is unavailable, `jev_select` returns error; agent uses existing manual actions
- Loop support: `jev_select` can be called repeatedly in a task loop (observe → choose → act → observe)

## Non-scope

- Not replacing existing `click`, `type`, `snapshot`, `screenshot` actions
- Not implementing full browser-use agent — `jev_select` is one action, agent orchestrates the loop
- Not adding screenshot-based selection (Jev consumes structured state, not pixels)
- Not handling canvas, shadow roots, frames, or file uploads in initial scope
- Not implementing the full jev-ultrafast agent loop — just the action primitive

## Behavior

### Happy path

1. Agent calls `browser(action="jev_select", url="...", goal="...")` or on an already-open page
2. Browser takes an atomic DOM snapshot: reads all visible controls, produces indexed table
3. Snapshot is converted to Jev-compatible state: `{page: {url, title, text}, elements: [{index, role, label, value, kind}], recent_actions: [...]}`
4. Jev request is constructed with speculative fan-out:
   - `operation`: Choice across [CLICK, TYPE_TEXT, SELECT, SCROLL_UP, SCROLL_DOWN, WAIT, DONE, BLOCKED]
   - `click_target`: Choice across clickable elements only
   - `type_text_target`: Choice across fillable elements only
   - `select_target`: Choice across selectable elements only
5. Jev returns `{operation: "CLICK", click_target: "3", confidence: 0.94}`
6. Tool resolves target "3" to the DOM node with that index
7. Tool validates: page hasn't changed (fingerprint match), target is not occluded
8. Tool executes: performs click on resolved node
9. Tool returns: `{operation, target, confidence, new_page_state}`

### Edge cases

- **Page changed between snapshot and execution**: re-observe, retry once, then return error
- **Jev selects DONE**: return completion state, agent verifies independently
- **Jev selects BLOCKED**: return blocked state with reason
- **TYPE_TEXT with missing value**: LLM returns null, agent supplies value manually
- **No interactable elements**: return empty element table, Jev should select BLOCKED
- **Occluded target**: scroll to element, retry once, then return error
- **Stale element reference**: re-resolve from index, retry once

## Design

### DOM snapshot format

```json
{
  "page": {
    "url": "https://www.google.com/travel/flights",
    "title": "Google Flights",
    "text": "Find cheap flights..."
  },
  "elements": [
    {"index": "1", "role": "combobox", "label": "Where from?", "value": "San Francisco", "kind": "fill"},
    {"index": "2", "role": "combobox", "label": "Where to?", "value": "", "kind": "fill"},
    {"index": "3", "role": "button", "label": "Search", "kind": "click"},
    {"index": "4", "role": "combobox", "label": "Sort by", "value": "Price", "kind": "select", "options": [{"index": "4:1", "label": "Price"}, {"index": "4:2", "label": "Duration"}]}
  ],
  "recent_actions": [
    {"operation": "CLICK", "target": "1", "fingerprint": "abc123"}
  ]
}
```

### Jev request structure

```json
{
  "model": "jev-latest",
  "state": { /* page + elements + recent_actions */ },
  "questions": {
    "operation": {"type": "choice", "criteria": {"CLICK": "Click an element...", "TYPE_TEXT": "Enter text...", ...}},
    "click_target": {"type": "choice", "criteria": {"1": {"element": "Where from?", "role": "combobox", "value": "San Francisco"}, ...}},
    "type_text_target": {"type": "choice", "criteria": {"2": {"element": "Where to?", "role": "combobox", "value": ""}, ...}},
    "select_target": {"type": "choice", "criteria": {"4:1": {"element": "Sort by", "option": "Price"}, ...}}
  }
}
```

Only the target head matching the selected operation is consumed. Unused heads are ignored.

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` — browser select uses the evaluate tool
- **Prior art**: `https://github.com/browser-use/jev-ultrafast` (7.9k stars, MIT)
- **Source reference**: `jev_ultrafast/model.py` (choose function), `jev_ultrafast/snapshot.js` (DOM snapshot)

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `tool/browser.rs` | Add `jev_select` action, DOM snapshot enhancement |
| `tool/browser.rs` | New `BrowserInput` variant: `goal` field |
| `tool/evaluate.rs` | Used for Jev API calls |

## Acceptance

1. Agent calls `browser(action="jev_select", goal="Search for flights to London")` → browser performs click on search button
2. Agent calls `browser(action="jev_select", goal="Type 'London' into the destination field")` → browser types "London"
3. Page changes between snapshot and execution → retries once, returns fresh page state
4. Jev selects DONE → returns completion, agent independently verifies
5. Jev unavailable → returns clear error, agent uses existing `click`/`type` actions
6. No interactable elements → Jev selects BLOCKED, agent adapts
7. Multi-step task via loop → each call observes fresh state, picks next action