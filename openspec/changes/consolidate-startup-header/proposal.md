---
execution:
  version: 1
  depends_on: []
---
## Why

The startup header spends most of its rows on information that can be found elsewhere (model hint, account router inventory, MCP and skill counts, directory). It does not show the two things that drive work in a repository: the Linear board load and the OpenSpec pipeline.

## What Changes

- Remove these rows: the product row (`jcode self-dev · client · ws:…`), the model row, the account router heading and its rows (including `No auto-routing accounts…`), the `mcp n/m` row, the `skills:` row, and the working-directory/branch row.
- Put the server and client on one row: `🗼 {server version} · 💻 {client version}`. Do not show the session name or its emoji. Keep the version-mismatch highlight. Show the `srv↑`/`cli↑` badges on this row only.
- Add a workspace row: `ws: {world} · {repo}: key:value …`. The values come from the matched `[[workspaces.rules]]` entry (`tracker`, `branch`, plus the existing `!id`/`?id` identity flag). Show it only when a rule matches.
- Add a Linear row: `linear: {org} · {team} · {board}: N {State} | N {State} …`. Org, team and board (project) names are read through the board's allowlisted provider and cached per `BoardScope`. Use the board's own workflow state names, in board order. Leave out states of type `completed` and `canceled`. Show the row only when the verified world has a Linear `[workspaces.rules.board]` binding.
- Add an OpenSpec row: `openspec: N not ready | N ready | N active`. Show it only when `openspec/` exists in the working directory. One background `openspec validate --changes --json` call (about 273 ms on 27 changes) is cached and runs again only when `openspec/changes/` changes on disk.

## Scope

- In scope: the TUI persistent header and startup header (`ui_header.rs`), a background header facts cache, and a read-only Linear name and count query through the existing board provider.
- Out of scope: ADO board rows, the `/board` UI, and writes to Linear or OpenSpec.

## Touched surfaces

- `crates/jcode-tui/src/tui/ui_header.rs` (`build_persistent_header_with_auth`, `build_header_lines_with_auth`, header tests)
- `crates/jcode-base/src/workspace.rs` (structured rule facts for the ws row)
- `crates/jcode-base/src/board/{linear.rs,provider.rs,cache.rs}` (scope names and per-state counts)
- New `crates/jcode-tui/src/tui/header_facts.rs` (background OpenSpec and Linear caches)

## Dependencies

- `openspec` CLI on `PATH` for the OpenSpec row. If it is missing, hide the row.
- Linear MCP server configured as the board's `server`. If it fails, show `linear: unavailable`.
