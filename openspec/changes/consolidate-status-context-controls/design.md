# Design

## Architecture

Create one status-segment model from the existing `TuiState::info_widget_data()` and related provider/cache snapshots. The model exposes optional segments for:

1. `model-context`: model, provider, access/effort, and context usage.
2. `kv-cache`: cache hit/read/write summary when telemetry exists.
3. `usage-limits`: provider quota/limit summary when available.

The renderer uses the same segment model for text and hit testing so visual labels and click regions cannot drift. The segment group is rendered in the existing primary status-line row. Existing floating widgets remain available but are controlled by independent visibility state.

## Interaction

A click on `model-context` toggles `WidgetKind::ModelInfo` and `WidgetKind::ContextUsage` as one grouped detail surface. A click on `kv-cache` toggles only `WidgetKind::KvCache`. A click on `usage-limits` toggles only `WidgetKind::UsageLimits`. Todos and memories are never included in these toggle groups.

The active state uses the existing accent/highlight styling. Unavailable segments are omitted and have no hit region. Repeated clicks are idempotent toggles. Narrow widths drop secondary segment detail before dropping the model-context group.

## State and compatibility

Use the existing session-scoped widget visibility mechanism if it can represent independent visibility. If current state only supports global widget visibility, add a small session-scoped set of hidden widget kinds with defaults preserving current behavior. Do not introduce a provider or persistence schema change. Existing `/todos` and memory controls remain authoritative for those surfaces.

## Verification

Add pure segment-model tests for ordering, omission, truncation, and hit regions. Add input tests for each independent toggle and preservation of todos/memories. Add render snapshots for wide and narrow layouts and no-data states. Run the existing TUI test suite and a focused visual smoke render.
