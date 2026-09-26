## Why

Jcode has several ways to find relevant repository context, but no reproducible local evidence shows when the installed Graft CLI's graph ranking helps over Jcode's existing `agentgrep` modes or plain lexical search. A bounded retrieval benchmark will measure relevance, context cost, and latency before any navigation integration is proposed.

## What Changes

- Add a reproducible, local-only evaluation protocol and runner for fixed repository questions.
- Compare `graft ask --source` with graph ranking enabled and disabled, plus `graft grep` and Jcode `agentgrep` where runnable.
- Record relevance, output size, latency, command failures, and graph freshness as machine-readable results with a concise summary.
- Keep benchmark fixtures and generated graph/results outside tracked source by default; document an explicit opt-in for refreshing or building a graph.

## Capabilities

### New Capabilities
- `repository-retrieval-evaluation`: Reproducible comparison of local repository-context retrieval strategies.

### Modified Capabilities

None. This adds evaluation tooling, not product retrieval behavior.

## Impact

Evaluation scripts, question/answer fixtures, and usage documentation only. Uses installed `graft`, `rg`, and the repository's Jcode tool-test or runner surface if available. No new runtime dependency, MCP server, AST/LSP implementation, model call, or tracked code-index output is in scope.
