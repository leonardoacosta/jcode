# Rendering Contract

## Source ownership

- `open-items --json --live-beads` owns Beads classification, active OpenSpec status, plans, and
  source availability. Live failure is unavailable, never permission to use cached JSONL.
- `triage-list-drafts --json --include-approved` owns proposal approval state and queue metadata.
- Use producer fields as data. A narrowly scoped detail lookup is allowed only when an item is
  explicitly marked ambiguous or a proposal dependency must be resolved before ownership.

For overlapping Beads signals, preserve the producer precedence exactly:
`blocked > in_progress > disposition > human_only > open`. Capability epics are containers.
Tasks linked to an active proposal are represented by proposal progress. Each tracked object has
exactly one owning section.

## Exact compact shape

Begin with this sentence, including zeros:

```text
<unresolved> Beads items remain unresolved: <open> open, <in_progress> in progress, <blocked> blocked.
```

Then emit only non-empty applicable sections in this order:

1. `In progress`
2. `Only open proposal` for one, or `Open proposals (N)` for many
3. `P1 work`
4. `Other actionable work`
5. `Open capability containers`
6. `Active OpenSpec changes`
7. `Blocked`
8. `Human-only`
9. `Open plan rows`
10. `Archive-ready proposals`
11. `Source warnings`

Use compact bullets:

```markdown
In progress:

- `<id>` — <concise title>
```

Active changes include `<done>/<total> tasks`. P1 and other actionable sections contain only open,
non-container, non-proposal-owned work. Keep every explicit ID when grouping siblings. Do not emit a
second count summary, duplicate descriptions, or render the same item under multiple headings.

## Detail fallback

Use a table only for a path whose truncation or multiple blocked/human-only reasons cannot fit safely
in one bullet. Columns are `ID | P | Title`, plus `Blocked by` or `Why | Default action` as applicable.
Do not also emit compact duplicates for that path. When `truncated` is true, state the visible and
total counts and identify the full-list command — `open-items --json --live-beads --limit=0` — without
silently implying completeness. `item_cap` is the visible-row limit; the headline counts and
`bucket_counts` always describe every retained bead, so quote them as-is rather than recounting rows.

## Proposals and warnings

An approved open proposal is apply-eligible; an unapproved proposal is human-only; an in-progress
proposal is in progress; an open blocking dependency overrides these states; and a done proposal is
archive-ready. Omit the proposal heading for zero proposals.

When a source is unavailable, retain every available section and add one bounded warning that names
the source and its reported error. Never infer missing work from another source.
