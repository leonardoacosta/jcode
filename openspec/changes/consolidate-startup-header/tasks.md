## 1. Remove and consolidate rows
- [ ] 1.1 Remove the product, model, account router, mcp, skills, and directory rows from `ui_header.rs`, and delete helpers that no longer have callers
- [ ] 1.2 Render `🗼 {ver} · 💻 {ver}` plus `srv↑`/`cli↑`, keeping the mismatch style
- [ ] 1.3 Update the header tests and snapshots (remote, standalone, mismatch, narrow width)

## 2. Workspace row
- [ ] 2.1 Return structured rule facts (world, repo, tracker, branch, identity flag) from the `workspace.rs` cached probe
- [ ] 2.2 Render the `ws:` row, hidden when no rule matches. Add tests

## 3. Linear row
- [ ] 3.1 Add read-only scope names (org, team, project) and per-state counts to the Linear board provider, with a fixture test for an arbitrary state name
- [ ] 3.2 Add a background cache in `header_facts.rs`, keyed by `BoardScope`, that never runs on the render thread
- [ ] 3.3 Render the row, excluding `completed`/`canceled`. Add tests for no binding, Brown, and failure (`unavailable`)

## 4. OpenSpec row
- [ ] 4.1 Run `openspec list --json` and `openspec validate --changes --json` in the background, and classify not ready, ready, and active
- [ ] 4.2 Invalidate the cache on `openspec/changes/` mtime. Hide the row when the dir or CLI is missing. Add tests

## 5. Verify
- [ ] 5.1 `cargo test -p jcode-tui ui_header` and `cargo test -p jcode-base board`
- [ ] 5.2 `selfdev build-reload`, then a debug_socket frame showing the new rows at 120 and 40 columns
