# Jcode AST/LSP tooling checkpoint

**Checkpoint date:** 2026-09-20
**Resume target:** 2026-09-21
**Scope:** Improve Jcode as a whole. Jev is evidence and an integration target, not the sole product scope.

## Current decision

- **BUY now:** ast-grep MCP as the recommended structural-query server. Jcode already speaks stdio MCP natively, so do **not** buy Supergateway or mcp-proxy for Jcode. Start with documentation and a usage measurement. Consider bundling only if demand is demonstrated.
- **EXTEND second:** Graft's deterministic tier-1 graph behind a thin, version-pinned `jcode-code-graph` adapter. Feed repository map and call/blast-radius output into turn-1 system-prompt/catchup context. Keep Graft's `--deep` LLM tier off initially.
- **BUILD last:** diagnostics-on-edit as the native form of upstream issue #471. Run the workspace's own scoped typecheck/lint after edits, compute diagnostic deltas, and surface them to the next turn. Escalate to native LSP only if this cheap path is measured insufficient.
- **DO NOT:** fork candidates, build a Jev-side AST engine, adopt isaacphi LSP first, or add XRAY. isaacphi's fd leak, sleeps, handshake race, and child lifecycle issues are especially relevant because Jcode already has recent child-reaping fixes.

## Why this differs from Jev

Jev only exposes streamable HTTP in its MCP inventory, so Jev needs a stdio bridge. Jcode's MCP path is already stdio-native. Therefore Supergateway is a Jev dependency but stays out of the Jcode plan.

## Evidence already retained

- Jev primary-source research and snapshots:
  `~/dev/personal/jev/docs/research/2026-09-20-jev-ast-lsp-candidates.md`
  and `~/dev/personal/jev/docs/research/evidence/2026-09-20-jev-ast-lsp/`
- Jev diagram: `~/dev/personal/jev/docs/diagrams/jev-buy-extend-build.html`
- Jcode-scoped research:
  `~/dev/jcode/research/ast-lsp-buy-extend-build.md`
- Jcode-scoped diagram:
  `~/dev/jcode/docs/diagrams/ast-lsp-buy-extend-build.html`
- Jcode research commit: `87ad757b6`
- Jev diagram commit: `a887db3`

## Verified Jcode facts

- Source checkout: `~/dev/jcode/source/jcode`
- Branch: `local/v0.84.0-homelab`
- Local fork state observed: 17 commits ahead of its recorded upstream position and 2052 commits behind `upstream/master` at checkpoint time. Re-check before implementation.
- `agentgrep` is approximately 1,835 lines across `crates/jcode-app-core/src/tool/agentgrep.rs` and `agentgrep/`, with no tree-sitter dependency found in the checked manifests.
- Existing Jev-related OpenSpec proposals include `add-jevsdk-evaluate-tool`, `jcode-jevy-compaction`, and `jcode-jevy-foreman-supervision`.

## Tomorrow's execution order

1. Re-read this checkpoint and inspect current branch/upstream state.
2. Inspect Jcode's MCP registration and bundled-server conventions. Decide whether the ast-grep work should be docs-only first or a bundled-server proposal.
3. Inspect the existing compaction/catchup context contracts before designing `jcode-code-graph`.
4. Inspect issue #471 and the post-edit lifecycle before writing diagnostics-on-edit code.
5. Turn the accepted slice into a bounded OpenSpec proposal, then implement with tests.
6. Measure before/after using: tokens per structural query, tool calls to converge, review/test outcomes, and supervision decision quality.

## Resume command

```bash
cd ~/.jcode
cat plans/2026-09-21-jcode-ast-lsp-resume.md
cd ~/dev/jcode/source/jcode
```
