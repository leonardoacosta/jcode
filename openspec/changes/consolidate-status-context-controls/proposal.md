# Consolidate Status and Context Controls

## Why

The current TUI presents model/provider/effort/context information in more than one surface. Context, KV cache, and provider limits also compete for transcript margin space. This makes the status information harder to scan and leaves no simple way to hide secondary panels while keeping todos and memories visible.

## What Changes

- Consolidate model, provider, access/effort, and context usage into one persistent status-line group.
- Add independently clickable status segments for KV cache and provider limits.
- Clicking the grouped model/provider/effort/context segment toggles the matching context/model detail widget.
- Clicking the KV segment toggles the KV cache detail widget.
- Clicking the provider-limits segment toggles the usage-limits widget.
- Preserve todos and memory widgets independently. Hiding context, KV cache, or provider limits must not hide todos or memories.
- Reuse existing `InfoWidgetData`, widget renderers, visibility settings, and KV/cache telemetry. No duplicate provider or context facts should be rendered in the default status-plus-widget presentation.
- Make hidden/visible state visually apparent in the status line without requiring a new modal.

## Scope and Exclusions

In scope: TUI status-line composition, widget visibility/toggle state, mouse hit regions, keyboard-compatible toggle plumbing if already supported by the widget controls, and deterministic render tests.

Out of scope: provider API behavior, rate-limit collection, new persistence formats, redesign of todo or memory content, and changes to model routing.

## Acceptance

- Model, provider, effort, and context usage appear once in the primary status line group.
- KV cache and provider limits appear as separate status-line segments when data is available.
- Clicking each segment toggles only its corresponding detail widget.
- Todos and memories remain visible when context/model, KV cache, or provider-limit widgets are hidden.
- Status segments degrade safely at narrow widths and do not overlap the input or transcript.
- Existing widget data and rendering tests remain green.
- New tests cover independent toggles, no-data segments, repeated clicks, and preservation of todos/memories.
