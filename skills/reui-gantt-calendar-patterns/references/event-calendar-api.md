
# reui Event Calendar — API Reference

> Source: `docs/recon/reui-gantt-calendar.md` (evidence-audited `/recon` run, 2026-07-23) against
> `github.com/keenthemes/reui` (MIT license). Every field/method below was independently
> re-fetched and verified during that recon's Phase 3.5 evidence audit.

## Install

```bash
pnpm dlx shadcn@latest add @reui/event-calendar
```

Package: `@reui/components-base-event-calendar` internally; deps `@base-ui/react@1.5.0`,
`@date-fns/tz@^1.5.0`, `date-fns@^4.1.0` (no `class-variance-authority` — a genuinely distinct
package from Gantt, not a re-export); peer `react`/`react-dom >=18`.

## Views (6 total)

Month, Week, Day, N-days (configurable multi-day), Agenda (chronological list), Resource
(time-grid booking view — distinct from Gantt's resource timeline, a separate component/API).

## Composition Contract

```tsx
<EventCalendar defaultEvents={events} defaultView="week">
  <EventCalendarNav />
  <EventCalendarToolbar />
  <EventCalendarContent />
</EventCalendar>
```

Navigation sub-parts: `EventCalendarNavToday`, `EventCalendarNavPrev`, `EventCalendarNavNext`,
`EventCalendarTitle`, `EventCalendarViewSwitcher`, `EventCalendarDatePicker`. View components:
`EventCalendarMonthView`, `EventCalendarWeekView`, `EventCalendarDayView`, `EventCalendarDaysView`,
`EventCalendarAgendaView`, `EventCalendarResourceView`, `EventCalendarTimeGrid` (shared week/day/
N-days engine).

## Core Props

- **State**: `events`/`defaultEvents`, `view`/`defaultView`, `date`/`defaultDate`,
  `dayCount`/`defaultDayCount`, `selection`/`defaultSelection`,
  `interactions`/`defaultInteractions` (drag/resize/slot-select toggles),
  `viewSettings`/`defaultViewSettings`, `loading`.
- **Display**: `timeZone` (IANA, defaults to system), `locale` (date-fns locale object),
  `weekStartsOn` (0-6), `dayStartHour`/`dayEndHour`, `slotDuration` (minutes),
  `snapDuration` (drag/resize snap granularity), `agendaDayCount`,
  `scrollMode` (`"contained" | "page"`), `nowIndicator`, `showWeekNumbers`, `offDays`.
- **Rendering overrides**: `renderEvent`, `renderEventTooltip`, `renderMonthCell`,
  `renderDayHeader`, `renderTimeGutterSlot`, `classNames` (per-element class hooks), `components`
  (swap individual view implementations).
- **i18n**: `i18n` — partial config with labels, view names, date formats, formatter functions.

## EventCalendarApi (via `apiRef` / `instance.api`)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `next()` / `prev()` | `() => void` | Navigate periods |
| `today()` | `() => void` | Jump to current date |
| `goTo(date)` | `(date: Date) => void` | Jump to specific date |
| `setView(view, opts?)` | `(view: CalendarView, opts?: { dayCount?: number }) => void` | Switch views |
| `getEvents()` / `setEvents()` / `addEvent()` / `updateEvent()` / `removeEvent()` | — | Event CRUD |
| `getOccurrences(range?)` | — | Expanded occurrences with recurrence applied |
| `select()` / `selectEvent()` / `clearSelection()` | — | Selection control |
| `setInteractions(patch)` | — | Toggle drag/resize/slot-select at runtime |
| `getVisibleRange()` / `getActiveRange()` | — | Rendered vs. logical date range |
| `scrollToTime(time)` | — | Scroll time grids to a specific time |

## Callbacks

`onEventClick` / `onEventDoubleClick`, `onEventUpdate` (validation funnel for drag/resize/API
timing changes — same `false | void/true | {start?,end?,allDay?}` contract as Gantt's),
`canDropEvent` (live gesture validation), `onSlotClick`, `onSelectSlot` (drag-create),
`canSelectSlot`, `onRangeChange`, `onViewChange`, `onDateChange`, `onDayCountChange`,
`onEventsChange`, `onMoreClick` ("+N more" indicator clicked).

## Event Shape

```typescript
interface CalendarEvent<TData = unknown> {
  id: string
  title: string
  start: Date
  end: Date
  allDay?: boolean
  recurrence?: string           // raw RRULE, or the structured rule shape shared with Gantt
  recurringEventId?: string
  originalStart?: Date
  color?: string
  readOnly?: boolean
  draggable?: boolean
  resizable?: boolean
  priority?: number
  resourceId?: string
  data?: TData
}
```

## Recurrence (shared engine with Gantt)

RFC-5545 subset: `freq`, `interval`, `count`/`until`, `byWeekday`, `byMonthDay`, `byMonth`,
`exDates`/`rDates` (excluded/additional dates), `weekStart`. Raw RRULE strings accepted directly.
Helpers: `expandRecurrence()`, `parseRRuleString()`, `formatRRuleString()`.

## Timezone Handling

All event math happens in the display timezone — DST-safe wall-time iteration via `toZoned()`/
`zonedStartOfDay()`. `timeZone` prop controls rendering; switching timezones at runtime (e.g. a
settings panel) does not mutate stored event data, only the display layer.

## Headless Hooks

`useEventCalendarState()` (headless root), `useEventCalendar()` (instance from context),
`useEventCalendarSelector()` (fine-grained subscription), `useEventCalendarView()`,
`useEventCalendarNavigation()`, `useEventCalendarOccurrences()`, `useEventCalendarDay()`
(per-cell subscription), `useEventCalendarViewSettings()`, `useEventCalendarGestures()`
(pointer gesture wiring), `useNow()` (current time with refresh interval).

## Helper Functions (pure, React-free)

From `event-calendar-lib.tsx`: `flattenResources()`, `buildEventIndex()`, `segmentOccurrence()`
(multi-day segmentation), `packTimedSegments()` (Google-Calendar-style overlap packing),
`packWeekRowLanes()`, `getViewDateRange()`, `stepDate()`, `getDayKey()`, `snapMinutes()`,
`rangesIntersect()`/`eventsOverlap()`, `isBarOccurrence()`/`spansMultipleDays()`.

## Styling

CSS variables: `--ec-event-color`, `--ec-sticky-offset`, `--ec-gutter-width`. Tailwind class
hooks via `classNames` prop.

## Where the real source lives (if reading the reui repo directly)

Same convention as Gantt: `packages/registry/bases/{base,radix}/components/event-calendar/src/index.ts`
is a generated preview-loader manifest, not the implementation. Real shipped source is in
`public/r/styles/<theme>/event-calendar*.json` — what the shadcn CLI actually installs.
