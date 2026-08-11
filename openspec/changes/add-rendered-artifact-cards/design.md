## Context

Jcode currently represents completed tool output primarily as text. `ToolOutput` can carry generic JSON metadata, but the agent converts the output into a `ContentBlock::ToolResult` containing only `tool_use_id`, `content`, and `is_error`; session rendering then produces a `RenderedMessage`, and the TUI produces a `DisplayMessage` whose `tool` role is dispatched to the generic tool renderer. Consequently, an explicit user-facing render artifact cannot retain a stable semantic identity across streaming, save, reconnect, and replay.

The TUI already has reusable rounded-box geometry, Markdown rendering, syntax highlighting, semantic code-block copy targets, narrow-width handling, and ASCII-compatible border utilities. The design must compose those primitives rather than create a parallel rendering stack.

The user selected semantic metadata over tool-name detection or content heuristics and limited scope to explicit render-tool artifacts only.

## Goals / Non-Goals

**Goals:**

- Define one backward-compatible semantic descriptor for Markdown, Message, and Code artifacts.
- Preserve the descriptor from tool completion through persisted history and restored TUI messages.
- Give each kind a visually distinct, accessible card while preserving content semantics and copy behavior.
- Keep untagged and unknown tool output on the current generic tool path.
- Make the contract reusable by built-in and MCP tools without depending on tool names.

**Non-Goals:**

- Boxing every assistant response or every fenced code block.
- Restyling reasoning, generic tool actions, user-message frames, plans, or existing system cards.
- Inferring artifact kinds from output content or tool names.
- Adding background fills, terminal-specific image protocols, or a persisted-state migration.

## Decisions

### 1. Use typed semantic artifact metadata

Introduce a shared serializable `RenderedArtifact` descriptor and `RenderedArtifactKind` enum with `Markdown`, `Message`, and `Code` variants. The descriptor carries optional `title` and `language`; artifact body content remains the tool output string so there is one source of truth.

`ToolOutput` gains an optional typed artifact field and ergonomic builders. The existing generic metadata field remains independent. This makes intent explicit at the producing tool and prevents display behavior from being coupled to tool registration names.

**Alternatives rejected:**

- Tool-name detection is brittle under aliases, MCP namespaces, and future tools.
- Content heuristics can misclassify ordinary output and violate the explicit-only scope.
- Encoding a hidden sentinel in output text pollutes model-visible history and copy behavior.

### 2. Persist metadata on the matching tool result

`ContentBlock::ToolResult` gains an optional, serde-defaulted artifact field. Agent tool-result construction copies `ToolOutput.artifact` into that field in every synchronous and streaming execution path. Provider adapters ignore the display-only field when serializing model-facing tool results.

The session renderer copies the descriptor into `RenderedMessage`, and TUI conversion copies it into `DisplayMessage`. Live and restored transcripts therefore use the same render dispatch. Older serialized sessions omit the field and deserialize as `None`.

**Alternative rejected:** retaining metadata only in transient stream events would make reconnect and replay visually inconsistent.

### 3. Dispatch artifact cards before generic tool rendering

For a `tool` message with recognized artifact metadata, `ui_prepare` calls the artifact-card renderer and bypasses generic tool chrome for the artifact body. Unknown future kinds or invalid metadata fall back to the generic tool renderer instead of dropping content.

Card identities:

- **Markdown:** document-blue border and `▤ Markdown` default title. Content uses the normal Markdown renderer, including headings, lists, tables, quotes, math, and diagrams.
- **Message:** warm neutral border and `● Message` default title. Content is rendered as readable Markdown-capable prose so intentionally formatted human-facing messages remain expressive.
- **Code:** terminal-green border and `<> Code` title with optional ` · <language>`. Content uses the existing fenced-code renderer and syntax-highlighting path.

All cards use existing rounded-box geometry, width-aware wrapping, centered-mode behavior, and ASCII glyph fallback. Titles are truncated safely at narrow widths. No background fill is required.

### 4. Preserve semantic copy behavior

Card borders, titles, and gutters are display chrome and must not enter copied artifact content. Markdown and Message cards map selectable text to the original artifact body. Code cards register one `CopyTargetKind::CodeBlock` target containing the exact source and language, preserving the existing copy badge and selection behavior.

### 5. Keep defaults and compatibility additive

Artifact rendering activates only when a producer explicitly sets a recognized descriptor. There is no global style toggle in the first version because the legacy behavior remains available by omitting metadata. No dependency or schema migration is introduced.

## Risks / Trade-offs

- **[Risk] Cross-crate propagation misses an execution path** → Centralize tool-result construction where possible and add live, persisted, and restored round-trip tests.
- **[Risk] Nested Markdown code frames inside a card become visually busy** → Keep the outer card border restrained and reuse current inner block rendering rather than adding another fill or heavy rail.
- **[Risk] Wide Markdown structures exceed the card** → Derive inner width before rendering and reuse current structured-block recentering and truncation rules.
- **[Risk] Typed metadata changes provider payloads accidentally** → Keep it display-only and add provider/session serialization tests proving model-facing text remains unchanged.
- **[Risk] New enum variants from newer clients fail older readers** → Treat recognized values strictly at the TUI boundary and retain generic fallback for absent or unsupported metadata representations.

## Migration Plan

1. Land the optional shared types and serde-compatible tool-result field.
2. Propagate metadata through agent, session, protocol, and TUI display conversions.
3. Add card rendering and copy maps.
4. Opt the explicit render tools into the new builders.
5. Run focused cross-crate tests, TUI snapshots, and a runtime render against an isolated socket.

Rollback removes producer opt-in first, immediately restoring generic tool rendering. The optional persisted field can remain safely ignored or be removed later without rewriting existing sessions.

## Open Questions

None. Artifact scope, semantic metadata, visual hierarchy, fallback behavior, and verification expectations were approved by the user.
