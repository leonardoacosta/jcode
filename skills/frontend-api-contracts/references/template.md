
# Contract Template + Worked Example

## Blank Template

```
## Procedure: <router>.<name>

**Trigger**: [When does frontend call this?]
**Auth**: [none | session required | role: admin]
**Type**: [query | mutation]

### Input
```typescript
z.object({
  // list all fields — be explicit about optionals
})
```

### Output
```
[Describe the shape in prose or pseudotype]
- topLevelField: type — [what it represents]
- nested.relation: populated? yes/no — [populated via `with: { x: true }`]
- nullable fields: [which fields can be null and why]
```

### Drizzle Notes
- Tables touched: [list]
- Relations needed: [explicit `with:` clauses required]
- Sort/filter: [orderBy, where clauses]
- N+1 risk: [flag if relation is inside a loop — needs `with:` not a second query]

### Frontend Usage
- Component: [which component consumes this]
- Pagination: [cursor | offset | none] — [expected page size]
- Invalidation: [what mutations should invalidate this query]
```

---

## Worked Example

**Scenario:**
- Feature: Event attendee list
- Procedure: `event.attendees.list`
- Input: `{ eventId: string, cursor?: string, limit?: number }`
- Output: `{ items: Attendee[], nextCursor: string | null }`
- Relations: attendee → user (with), attendee → ticket (with)
- N+1 risk: yes (`user.profile` nested in attendee)

---

## Procedure: event.attendees.list

**Trigger**: Rendered when the organizer opens the attendee tab on the event dashboard.
**Auth**: session required (organizer role)
**Type**: query

### Input
```typescript
z.object({
  eventId: z.string(),
  cursor: z.string().optional(),
  limit: z.number().min(1).max(100).default(20),
})
```

### Output
```
{ items: AttendeeRow[], nextCursor: string | null }

- items[].id: string — attendee record id
- items[].status: "confirmed" | "waitlisted" | "cancelled"
- items[].user: populated — name, email via `with: { user: true }`
- items[].user.profile: populated — avatar URL via `with: { user: { with: { profile: true } } }`
- items[].ticket: populated — tier name, price via `with: { ticket: true }`
- nextCursor: string | null — null when last page reached
```

Frontend types via inference:
```typescript
import type { RouterOutputs } from "@workspace/api";
type AttendeeRow = RouterOutputs["event"]["attendees"]["list"]["items"][number];
```

### Drizzle Notes
- Tables touched: `attendee`, `user`, `userProfile`, `ticket`
- Relations needed:
  - `with: { user: { with: { profile: true } } }` — nested profile causes N+1 if not eager-loaded
  - `with: { ticket: { columns: { id: true, tierName: true, price: true } } }` — partial select
- Sort/filter: `orderBy: asc(attendee.createdAt)`, `where: eq(attendee.eventId, input.eventId)`
- N+1 risk: YES — `user.profile` is a second-level relation. Must be in the `with:` clause or it
  generates one extra query per attendee row. Do NOT load profile in a separate loop.

### Frontend Usage
- Component: `<AttendeeTable>` inside the organizer event dashboard
- Pagination: cursor — page size 20; "Load more" button appends next page
- Invalidation: `utils.event.attendees.list.invalidate({ eventId })` after:
  - `event.attendees.remove` mutation
  - `event.attendees.updateStatus` mutation

> ❓ Does deleting an attendee cascade to their tickets, or are tickets preserved for reporting?
