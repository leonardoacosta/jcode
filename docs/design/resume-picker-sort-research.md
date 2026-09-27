# Sort UX Research: Session Picker

## Sources analyzed

| Source | Default sort | Toggle mechanism | Sort fields | Key pattern |
|--------|-------------|------------------|-------------|-------------|
| **fzf** | Fuzzy relevance | CTRL-R toggles chronological | relevance, chronological | `--scheme=history` for command-oriented inputs |
| **tmux choose-tree** | name | O cycles, r reverses | index, name, time | Interactive keybinding cycling |
| **tmux choose-client** | name | O cycles, r reverses | name, size, creation, activity | Activity sort for session mgmt |
| **Zed command palette** | frecency | N/A | frecency (freq+recency) | Uses Mozilla-style algorithm |
| **Mozilla Places** | frecency | N/A | typed > bookmark > link | 4 time-decay buckets (4d/14d/31d/90d) |
| **NNg/Smashing** | N/A | Dropdown/segmented | recency, A-Z, popularity | Sort is separate from filter |

## Design principles

1. **Sort is separate from filter** — filters narrow, sort reorders. Should not be conflated.
2. **Default matches primary use case** — for session resume, that's recency (fzf `--scheme=history`).
3. **Toggle should be discoverable but non-modal** — tmux's O/r pattern is the industry standard.
4. **On search: relevance matters** — matching items should float to top by match quality.
5. **Tie-breaking matters** — title/saved sorts must fall back to recency for stability.
6. **Time-decay improves recency** — Mozilla's bucket weights (100/70/50/30/10) prevent old-but-frequent from dominating recent-but-rare.

## What we implemented

- `SessionSortMode` enum: Recency, Title, SavedFirst, Creation, Relevance
- `o` cycles sort field, `O` reverses (tmux-compatible)
- Title bar indicator when non-default: `↓ title (o sort · O reverse)`
- Tie-breaking on recency for title/saved sorts
- Active mode preserves ready-first triage with chosen sort

## What remains (future work)

- **Relevance sort** when searching: needs match quality scoring from search index
- **Frecency scoring** as an alternative to pure recency: needs visit counting infrastructure
- **Persisted sort preference** between sessions