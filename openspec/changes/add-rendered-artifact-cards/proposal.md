## Why

Explicit render-tool artifacts currently blend into ordinary tool actions and reasoning in the transcript, making structured documents, human-facing messages, and code snippets harder to find and scan. Jcode needs a semantic artifact contract and distinct card hierarchy without changing ordinary assistant, reasoning, or untagged tool output.

## What Changes

- Add semantic metadata for explicit rendered artifacts with `markdown`, `message`, and `code` kinds plus optional title and code language.
- Preserve artifact metadata through live tool streaming, persisted session history, reconnect, and replay.
- Render each artifact kind with a dedicated, width-aware TUI card while preserving Markdown structure, code highlighting, and code copy targets.
- Keep ordinary assistant text, reasoning, tool actions, and legacy or untagged tool output behavior unchanged.
- Fall back safely to ordinary tool rendering for missing or unknown artifact metadata.

## Capabilities

### New Capabilities

- `rendered-artifact-cards`: Semantic render-tool artifacts, their persistence contract, and distinct Markdown, Message, and Code card presentation.

### Modified Capabilities

None.

## Impact

- Tool output and message/session types gain backward-compatible optional artifact metadata.
- Agent streaming and session rendering preserve that metadata across live and restored transcripts.
- TUI display messages and render dispatch gain three artifact-card paths and focused snapshot/copy tests.
- No persisted data migration is required because the metadata is optional and older sessions retain the current fallback path.
