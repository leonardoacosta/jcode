# Tasks

- [x] Inventory the existing status-line, info-widget visibility, click dispatch, KV cache, usage limits, todos, and memory paths. Record touched modules and avoid unrelated in-progress changes.
- [x] Add a shared status-segment data model built from existing TUI snapshots.
- [x] Render model/provider/effort/context once in the primary status line and remove duplicate default rendering where the new group owns the fact.
- [x] Add independent KV cache and provider-limit status segments with narrow-width degradation.
- [x] Add hit testing and click dispatch for the three segment controls.
- [x] Wire segment toggles to the existing widget visibility state. Preserve todos and memory visibility independently.
- [ ] Add unit tests for segment composition, missing data, truncation, active styling, and hit regions.
- [x] Add input/integration tests for independent toggles, repeated clicks, and todo/memory preservation.
- [ ] Run focused TUI tests, the full relevant crate test suite, and a rendered smoke check at wide and narrow terminal sizes.
- [x] Run strict OpenSpec validation and review the diff for scope or duplicate facts.
