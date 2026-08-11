
# reui Gantt — API Reference

> Source: `docs/recon/reui-gantt-calendar.md` (evidence-audited `/recon` run, 2026-07-23) against
> `github.com/keenthemes/reui` (MIT license, 3140 stars at recon time). Every field/method below
> was independently re-fetched and verified during that recon's Phase 3.5 evidence audit.

## Install

```bash
pnpm dlx shadcn@latest add @reui/gantt
```

Package: `@reui/components-base-gantt` internally; deps `@base-ui/react@1.5.0`,
`@date-fns/tz@^1.5.0`, `class-variance-authority@^0.7.1`, `date-fns@^4.1.0`; peer
`react`/`react-dom >=18`.

## Composition Contract

```tsx
<Gantt defaultEvents={events} resources={resources} defaultScale="month" className="h-[480px]">
  <GanttNav />
  <GanttView />
</Gantt>
```

`<Gantt>` is the root provider/container. Child slots read from shared context and are fully
replaceable: `<GanttNav>` (with sub-parts `GanttNavToday`, `GanttNavPrev`, `GanttNavNext`,
`GanttTitle`, `GanttScaleSwitcher`, `GanttDatePicker`), `<GanttView>` (main timeline + tree pane),
`<GanttBar>` (individual schedulable event), `<GanttToolbar>` (consumer action-button slot).

## Data Shapes

```typescript
interface GanttEvent<TData = unknown> {
  id: string
  title: string
  start: Date
  end: Date
  allDay?: boolean
  recurrence?: GanttRecurrenceRule | string
  color?: string
  readOnly?: boolean
  draggable?: boolean
  resizable?: boolean
  priority?: number
  progress?: number
  resourceId?: string
  data?: TData
}

interface GanttResource {
  id: string
  title: string
  children?: GanttResource[]
  color?: string
}
```

## View Configuration Props

| Prop | Type | Default | Purpose |
|------|------|---------|---------|
| `nowIndicator` | boolean | true | Red now-line on axis |
| `interval` | number | 60 | Day-scale unit interval (minutes) |
| `scrollbars` | `"custom" \| "native"` | `"custom"` | Scroll implementation |
| `dragCreate` | boolean | false | Empty-track drag-to-create |
| `offDays` | `boolean \| GanttOffDaysConfig` | true | Off-day marking (weekends + custom dates) |
| `barLabel` | `"inside" \| "outside" \| "auto"` | `"inside"` | Title placement |
| `timelineLines` | `"vertical" \| "both" \| "none"` | `"vertical"` | Gridline display |
| `zoomRange` | `{ min?, max?, step? }` | 0.5-3, step 0.25 | Zoom bounds |

## Callbacks

```typescript
onEventUpdate(update: GanttProposedUpdate<TData>): GanttUpdateResult
// Return false to reject, void/true to accept, or an adjusted {start?, end?, allDay?}
// -- this is the SINGLE commit gate for drag AND resize.

canDropEvent(update: GanttProposedUpdate<TData>): boolean
// Live validity predicate DURING drag/resize -- not the commit gate.

onSelectSlot(slot: GanttSlotDraft): void
// Drag-create commit handler (requires dragCreate: true).

onResourceReorder(...): boolean
// Validates drag-reorder of resource rows.
```

## Interactions

- **Drag**: horizontal move within a resource row only — never across rows. Row reassignment is
  a separate `resourceId` mutation via `onEventUpdate`, not a drag gesture.
- **Resize**: edge resizing on bar segments.
- **Slot selection**: drag-create gestures for new-event scheduling (`dragCreate: true`).
- **Row selection**: checkbox support for leaf rows, controlled or uncontrolled.
- **Resource reordering**: drag-reorder rows, validated via `onResourceReorder`.

## Recurrence

RFC-5545 subset: `freq` (daily/weekly/monthly/yearly), `interval`, `count`/`until`, `byWeekday`
(with ordinals for monthly/yearly). Expansion is DST-safe, capped at `MAX_OCCURRENCES = 1000`
occurrences per event, and throws `GanttRecurrenceError` on unsupported rule shapes rather than
silently mis-expanding. `parseRRuleString(input: string, timeZone?: string): GanttRecurrenceRule`
also accepts a raw RRULE string directly on an event's `recurrence` field.

## Headless Hooks

- `useGanttState` — root headless integration hook.
- `useGanttSelector` — fine-grained subscription (avoids re-rendering on unrelated state changes).

## Layout Metrics (customizable)

```typescript
interface GanttMetrics {
  laneHeight?: number         // default 1.75rem
  rowPadding?: number         // default 0.5rem
  minRowHeight?: number       // default 2.5rem
  autoLabelMin?: number       // default 7px
  unitWidths?: Partial<Record<GanttScale, number>>
  minTimelineWidth?: number   // default 200px
  infiniteScrollEdge?: number // default 160px
}
```

## i18n

Deep-partial overrides: labels (Today, Previous, Next, scale names, etc.), date-fns format
strings for titles/time labels, custom formatters, RTL support (Arabic example in upstream docs).

## Other Features

- **Summary bars**: duration-weighted rollups on parent resource rows.
- **Tree panel resizing**: splitter, configurable bounds (180-640px default).
- **Infinite scrolling**: automatic range extension with a configurable growth cap.
- **Custom columns**: extra tree-panel columns with per-row renderers.

## Where the real source lives (if reading the reui repo directly)

`packages/registry/bases/{base,radix}/components/gantt/src/index.ts` is a **generated
preview-loader manifest** (demo IDs → lazy imports), not the implementation. The actual shipped
`.tsx` source is in `public/r/styles/<theme>/gantt*.json` (e.g. `base-luma/gantt-bar.json`) —
this is what the shadcn CLI actually downloads and injects into a consumer's repo.
