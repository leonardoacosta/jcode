
# termcast Component API

Source: `remorses/termcast`'s `verceltui` extension (recon: `docs/recon/termcast-verceltui.md`,
2026-07-19). termcast ports the Raycast extension API (`List`, `Detail`, `Form`, `ActionPanel`,
`Action`, `showToast`, `Icon`, `Color`, `getPreferenceValues`, `LocalStorage`) to the terminal via
`@opentui/react` + Bun — you write a TUI as a React component tree using the same declarative
props as a Raycast extension. Import paths below (`termcast`, `@termcast/utils`) are inferred
from the recon (the marker `"termcast"` package-name is confirmed by `stack-detect.sh`'s probe;
the exact import surface was not directly captured upstream) — verify against real termcast
types before relying on them for anything beyond the scaffolded skeleton.

## Declarative manifest (`package.json`)

Commands and preferences are data, not code — the runtime reads the manifest to build the
command list and resolve `getPreferenceValues()` (preferences persist in SQLite, keyed
`preferences.{extensionName}`):

```jsonc
"commands": [{ "name": "search-deployments", "title": "Search Deployments", "mode": "view" }],
"preferences": [{ "name": "accessToken", "type": "password", "required": true, "link": "https://vercel.com/account/tokens" }]
```

## `List` / `List.Item` — accessory-alignment law

**Every `List.Item` in a given `List` must carry the same number of `accessories`, in the same
order, or column alignment breaks across rows.** This is not a stylistic preference — the
renderer lays out accessory columns positionally, so item 3 having 1 accessory while item 4 has 2
shifts every subsequent column. When an accessory is conditionally absent, render a placeholder
(`{ text: "" }` or similar) rather than omitting the entry.

`accessoryTagsLayout` widths map to all accessories **by position** — the same reason: a
mismatched accessory count at any row desyncs the whole column's width computation, not just that
row's rendering.

A `List.Item` composes `accessories`, an optional `detail={<List.Item.Detail metadata={...}/>}`
for split-pane detail views, and `actions={<ActionPanel>...}` for the command palette bound to
that row (verceltui `search-deployments.tsx:73-210`).

## Data fetching: `useCachedPromise` + `revalidate()`

`useCachedPromise` (from `@termcast/utils`) is the standard data-fetching hook — it caches across
renders and exposes `revalidate()` for explicit refresh. **Call `revalidate()` after every
mutation** (an action that writes data) — cached data does not know a mutation happened, and
waiting for the next poll interval leaves the UI stale in the meantime. verceltui also polls on
a `setInterval` (4s in `search-deployments.tsx:63-73`) for live status updates alongside the
cache — polling and `revalidate()` are complementary, not alternatives.

## Cache vs `LocalStorage`

- **Cache** (`useCachedPromise`'s internal cache) — ephemeral, request-shaped, invalidated by
  `revalidate()`. Use for anything derived from a live fetch.
- **`LocalStorage`** — durable, key-value, survives process restarts. Use for user
  preferences/state that must persist across sessions (selected team, last-used filter), not for
  data that's cheap to re-fetch.

## I/O separation: typed API client

Isolate all `fetch` calls and pure helpers (date formatting, truncation) into a dedicated
`api.tsx`/`vercel-api.tsx`-style module, with request/response shapes in a sibling `types.tsx`.
Component files import from the client module; they never call `fetch` directly. This is the
same "types → api → component" layering `t3-code-patterns` enforces for the web stack, applied to
a TUI's typed client instead of a tRPC router.

## Bun explicit-`react`-dep gotcha

Add `react` as an **explicit** dependency in `package.json` — do not rely on Bun resolving it
transitively through `termcast`/`@opentui/react`. A transitively-resolved `react` can produce a
second copy in `node_modules`, and two React instances in one process crash
`useSyncExternalStore` (the hook `useCachedPromise`-style state hooks are commonly built on) with
a context-mismatch error that has nothing to do with your code.

## Agent canon (from upstream `SKILL.md`/`CLAUDE.md`)

- `logger.log`, never `console.log` — stdout is the render surface.
- No `setTimeout` for React state — use the framework's scheduling/poll primitives.
- Minimize `useState` — prefer derived state and `useCachedPromise`.
- `.tsx` for all JSX, never bare `.ts`.
- `ctrl`/`alt`+letter shortcuts only — bare letters collide with text input.
- `showFailureToast` for every action that can fail.
