
# termcast PTY inline-snapshot e2e testing (tuistory)

Source: `verceltui/src/search-deployments.vitest.tsx` + `search-deployments-demo.tsx` (recon:
`docs/recon/termcast-verceltui.md`, 2026-07-19). Golden-master testing for a terminal UI — the
test drives a real pseudo-terminal running the app and asserts on the literal rendered frame.

## 1. The mock-data `*-demo.tsx` variant

Never point an e2e test at the live command — it needs a real API token and network access, and
its data is non-deterministic. Instead, write a sibling `<command>-demo.tsx` that mirrors the
real command 1:1 except for the data source: hardcoded `MOCK_*` data in place of the typed API
client call. This decouples the test from the live API entirely.

The demo file calls `renderWithProviders(<Demo/>)` **at module scope** (not inside a `main()` or
an exported function the test calls) — `tuistory`'s `launchTerminal` spawns `bun` directly on
this file, so the component tree must already be mounted the instant the PTY session boots; there
is no separate "call the test's entry point" step.

```tsx
// src/list-items-demo.tsx
export const MOCK_ITEMS: Item[] = [ /* ... */ ];

function ListItemsCommandDemo() { /* same JSX as the real command, reading MOCK_ITEMS */ }

renderWithProviders(<ListItemsCommandDemo />); // module scope — not inside a function
```

## 2. Launching the PTY session

`launchTerminal` boots a real pseudo-terminal running the demo under Bun:

```tsx
import { launchTerminal } from "tuistory";

const session = await launchTerminal({
  command: "bun",
  args: ["src/list-items-demo.tsx"],
  cols: 140,
  rows: 30,
});
```

`cols`/`rows` matter — a narrower terminal wraps/truncates differently, which changes the
snapshot. Pick a size wide enough that the real UI's columns (title + all accessories) don't
wrap, and keep it stable across the test file so snapshots stay comparable.

## 3. Driving the session

- `session.press('down')` / `session.press('up')` / `session.press('enter')` — navigate the list.
- `session.type('optimize')` — type into a search/filter field.
- `session.text({ waitFor: (t) => t.includes('dark mode') })` — block until the rendered frame
  contains an expected substring. **Always `waitFor` before asserting** — rendering (and any
  fetch/cache resolution inside the demo) is async; asserting immediately after `press`/`type`
  races the render.

## 4. Asserting with golden frames

```tsx
expect(await session.text()).toMatchInlineSnapshot();
```

`toMatchInlineSnapshot` captures the literal rendered terminal frame (all visible text,
positionally laid out) as the source of truth. A regression in spacing, accessory alignment, or
copy shows up as a snapshot diff — this is why the accessory-alignment law in
`references/component-api.md` matters for tests, not just visual polish: an alignment bug fails
the snapshot assertion directly.

**Update flow**: `pnpm e2e -u` re-records every inline snapshot in the run. Review the diff before
committing — an updated snapshot is either confirming an intentional UI change or silently
baking in a regression.

## 5. CI-skip when the extension folder is absent

termcast's own test suite (`TESTING_RAYCAST_EXTENSIONS.md`) skips PTY e2e tests when the
extension folder isn't present in the CI checkout (e.g. a partial/sparse checkout, or a CI job
that doesn't need the TUI layer). Mirror this convention in scaffolded projects: guard the
`describe`/`it` block (or the whole file) with an existence check on the extension's expected
root, and skip rather than fail when it's absent — a missing directory is an environment
difference, not a test failure.

```tsx
import { existsSync } from "node:fs";

const hasExtension = existsSync("src");
(hasExtension ? describe : describe.skip)("list-items", () => {
  // ...
});
```

## Full command reference

- `pnpm e2e` — run all PTY snapshot tests.
- `pnpm e2e -u` — run and update snapshots.
- `pnpm tsc && pnpm e2e` — full quality gate (see `templates/workflow/tui-app.CLAUDE.md.tmpl`).
