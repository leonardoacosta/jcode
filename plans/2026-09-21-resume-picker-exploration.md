# Resume picker: exploration

Date: 2026-09-21
Status: recommendation only. No formal change, implementation, or approval implied.

## Intent and constraints

Make the TUI resume view useful for finding and recognizing a session among hundreds: better sorting, composable filtering, useful search, compact contextual rows, and a responsive viewport. The supplied screenshot shows 261 sessions, clipped titles and controls, low-contrast metadata, a narrow list, and a large transcript pane. Keep existing resume/import/recovery contracts. No new technology was required.

## Evidence and provenance

Inspected the current workspace's tracked source at `/home/nyaptor/.jcode`, HEAD `c9a6cec`. Its picker source was restored in commit `59588ec` on September 15. The running binary reports `2db43408e`, which is not present as an object in this repository. Therefore this is source inspection plus screenshot evidence, not runtime verification of the running build. A read-only lookup in `/home/nyaptor/dev/jcode/source/jcode` also found the fixed 40/60 split and corresponding filter/render functions. No code from another checkout or branch was copied or integrated. Resolve the authoritative implementation checkout before authoring an implementation change.

Internal references (paths relative to this repository):

- `crates/jcode-tui/src/tui/session_picker.rs`: input modes, async preview loading and rendering, fixed pane split.
- `crates/jcode-tui/src/tui/session_picker/render.rs`: card content, hard-coded truncation, list rendering.
- `crates/jcode-tui/src/tui/session_picker/filter.rs`: matching, ordering, grouping, selection retention.
- `crates/jcode-tui/src/tui/session_picker/navigation.rs`: focus, mouse routing, paging.
- `crates/jcode-tui/src/tui/session_picker/loading.rs`: bounded search material and fast in-memory AND-token matcher.
- `crates/jcode-tui-session-picker/src/lib.rs`: shared session descriptor, source classification, mutually exclusive filters.
- `crates/jcode-tui/src/tui/session_picker_tests.rs` and `session_picker/loading_tests.rs`: existing search, filter, preview, mouse, paging, and loading tests.
- Bundled `docs/RESUME_BEHAVIOR.md`: Enter/current-terminal, alternate/new-terminal, external imports, bookmarks, multi-selection behavior.

No matching project memory was returned. Session-history search returned unrelated older discussions and was scan-limited, so it provides no design authority. No resume/picker matches were found under the local `openspec` directory. This does not establish absence from all external archives.

External precedent: https://github.com/junegunn/fzf/blob/master/README.md, fetched September 21. Its search syntax, explicit relevance ordering, toggleable preview, and configurable preview layout are useful interaction precedents. This is not a recommendation to install fzf or replace Jcode's integrated picker with a subprocess. Context7 was not required because no new library, framework, API, or function was explicitly required.

## Current state and diagnosis

1. Ordinary cards contain title/status, message counts/tokens, first prompt, creation time/path, and a spacer. Crashed cards add a reason. Roughly five or six lines buy only two lines of task context.
2. Titles are truncated at 54 characters and prompts at 72 before actual pane width is considered. Paths have another fixed limit. Character counts are not terminal-cell widths.
3. The list gets 40% of available width at every terminal size. A transcript occupies the remaining 60%, even when that makes titles and help unusable.
4. Ordering uses descending `last_message_time`. The readily visible timestamp says `created`, making the order harder to interpret. All-mode then partitions saved/server/orphan sections, so it is not global recency. Active mode additionally prioritizes ready sessions.
5. `SessionFilterMode` cycles through mutually exclusive directory, catch-up, saved, active, and source filters. Users cannot express current directory AND saved AND a source.
6. Search already supports case-insensitive, order-independent AND tokens and highlighting. It is not merely title search. However it matches a bounded, sampled in-memory corpus, not guaranteed full transcripts. Loading a preview can expand that corpus. It has no explicit relevance sort or field filters.
7. Source classification can mix origin and model/provider identity. For example, the Codex predicate also matches a model name. A redesigned source facet must say whether it means transcript origin or model route rather than silently changing semantics.
8. Preview is asynchronously loaded, wrapped, cached, and rendered using visible slices. Preserve this work. Default scroll goes to the transcript end or first visible search match. Rebuilding filters resets preview scrolling even if the selected session remains unchanged.
9. Paging uses fixed step constants rather than the actual rendered page. Selection is retained by ID across filtering, but hidden multi-selected sessions are pruned. This safety behavior needs to remain explicit.
10. Existing tests named `session_matches_query_*` can exercise a test-only transcript fallback. They do not by themselves prove interactive full-history search coverage.

## Alternatives

| Option | Benefit | Cost / limitation |
| --- | --- | --- |
| Cosmetic compression only | Smallest change, quick density improvement | Does not fix discoverability, search coverage, or filtering model |
| New presentation and query model over existing loader/actions | Addresses all requested problems while keeping import/recovery contracts | Requires input-state and shared-descriptor compatibility work |
| External fzf picker or wholesale rewrite | Mature generic fuzzy-selection interaction | Rebuilds integrated previews, onboarding, live presence, recovery and action semantics |

Recommend the middle option. Replace the presentation and query state, not the session backend.

## Recommended experience

### Information hierarchy

Three-line rows, without a blank spacer:

```text
> Factory durability research                         4m
  Resume workflows without launching duplicate attempts
  jcode · ready · saved
```

The first line is the task title plus right-aligned last-activity age. The second is a useful contextual excerpt. The third is the short project identity and only meaningful state/source badges. In the illustration, `jcode` is a project label. Distinguish project and external source labels in the real UI.

Context selection: search-hit excerpt when searching, otherwise latest meaningful user request, otherwise first meaningful user request. Show a short, explicitly role-labelled latest assistant excerpt in the preview. Strip known transport/system wrappers deterministically, not arbitrary content that merely looks technical. Do not invent an AI-generated summary or claim a task is complete from its prose. Do not synchronously load every full transcript just to fill rows. The current descriptor lacks a dedicated latest-request excerpt, so loader/cache additions are necessary, not just render changes.

Move message counts, token estimates, created time, full path, provider/model, session ID and detailed crash reason into a details area. Retain a concise interrupted indicator in rows. Use readable secondary text and non-color state cues.

### Sorting and filtering

Default to flat global last-activity order, newest first. Show that same activity time in the row. Specify the activity source and fallback for each transcript origin. Break ties by stable target identity. Offer oldest activity, newest creation, and title sorting. Enable relevance sort when a query exists, with recency and stable identity as tie-breakers. Saved is a facet/badge, not an implicit override of the selected sort. Preserve explicit ready-first ordering for the active-manager entry point and label it.

Keep existing all-session scope as the safe initial default. Provide independent project/directory, state, saved, source-origin and time filters. Expose applied filters and a clear-all action. Distinguish active/ready/working, interrupted, and needs-catch-up. Model/provider filtering is separate from transcript source. Define whether project means exact directory, repository root, or related worktrees before shipping it. Do not assume unrelated directories sharing a basename are the same project.

### Searching

Always-visible search affordance with match count and coverage status. Retain AND-token matching initially, improve ranking by exact title/label, title tokens, recent request, then older sampled content. Show the matching excerpt and highlights so the result explains itself. Do not introduce permissive fuzzy matching of long transcript bodies in the first increment.

A subsequent explicit deep-search action can search older/full transcripts asynchronously, with progress, cancellation, result deduplication and scope disclosure. A zero-result view must distinguish no matches in indexed data from no matches in complete history. Query syntax such as `project:`, `state:`, and `source:` is optional later, not required for the initial facet UI.

### Viewport and input

Make the list primary. Wide terminals get a list-majority split, subject to minimum useful pane widths. Narrow terminals show the full-width list with a toggle to the selected preview, not two unreadable columns. Derive breakpoints from cell widths and fixtures, not screenshot pixels.

Use a persistent, width-aware toolbar and short action footer. Overflow secondary controls into help rather than clipping them. Show result position/range and a list scrollbar. Page movement uses the actual available row geometry. Retain selected identity and its viewport position across refreshes and resize. Preserve preview position per session when returning to it. Only auto-follow newly loaded content while the user has not manually scrolled away.

Start preview at the latest user/assistant exchange, or the matching excerpt when searching, with full transcript available. Avoid defaulting to the middle of a long final assistant response just because it fills the pane.

Do not silently replace navigation shortcuts with type-to-search. Keep `/` as explicit search entry initially, with visible affordance. Define focus, Esc stages, Enter while searching, empty-result Enter, and footer actions consistently. Preserve configured Enter/Ctrl+Enter behavior and offer a discoverable alternate-action route for terminals that cannot distinguish those keys.

## Adversarial findings and invariants

- Preserve stable session/ResumeTarget identity. Never choose an action target by a stale display index after async results reorder.
- Do not turn historical `Active` snapshots into claims of live ownership. Reuse live-presence checks and Claude takeover confirmation.
- Keep the crash-group restore set independent of arbitrary search/filter results, and clearly state exactly what recovery acts on.
- Multi-selection must not invisibly act on hidden items. Retain current prune-hidden behavior initially and indicate selection-count changes.
- Avoid ranking/list jumps on every streaming update. Freeze or defer order updates while the user is actively navigating, with a refresh indication.
- Search expansion invalidates cached match sets. The existing prefix-narrowing optimization needs a corpus-generation key if data arrives asynchronously.
- Bound memory, scan concurrency and preview work. Never add transcript disk I/O to every keystroke. Benchmark both hundreds and thousands of sessions.
- Sanitize terminal control sequences in excerpts. Do not send private transcripts to external summarization or indexing services.
- Handle corrupt/missing transcripts, disappearing files, denied permissions, duplicate imports, unavailable servers and stale cache entries as per-row states where possible.
- Full paths, emoji, combining marks and wide characters require terminal-cell-aware truncation. Ensure tiny heights and huge transcripts cannot panic or overflow scroll counters.
- Shared picker types have consumers beyond this view. Audit serde/cache compatibility and onboarding, active-manager, standalone picker, embedded picker and external-import callers before changing the filter enum or descriptor.
- No credentials, external approval, provisioning, or irreversible actions are needed for this proposal. No deployment or daemon restart is authorized by this exploration.

## Proposed delivery scope

1. Compact contextual rows, readable hierarchy, adaptive panes, bounded footer, real page navigation, stable scroll/selection. Preserve current actions and search semantics.
2. Explicit sort model, flat default results, independent facets, useful indexed-search ranking and match snippets. Preserve specialized entry-point behavior and expose coverage.
3. Separately scoped deep-history search only if indexed search still misses sessions the user needs. No semantic/vector search, automatic summaries, or new search service by default.

## Verification and remaining decisions

Before feature authoring, confirm authoritative source checkout and preferred row/preview hierarchy. Proposed defaults are three-line rows, flat recent activity, all projects, preview only when enough width exists. Exact responsive breakpoints and input bindings should be settled with before/after frames.

Acceptance cases: known title lookup, recalling a recent request, combining project+saved+state filters, finding a deep-history-only phrase, recovering the exact interrupted group, and resuming the intended target after refresh or sort change. Distinguish fast indexed-search acceptance from deep-search requirements.

Use deterministic renderer snapshots at 80x24, 100x30, 120x40, 160x50, narrow/tall, and tiny dimensions. Include long titles, identical basenames, multilingual/wide text, no results, loading and corrupt transcripts. Test scrollbar/range math, focus, click hitboxes, viewport-relative paging, async stale-result rejection, coverage messaging, stable sort ties, and multi-selection pruning. Run real TUI debug testers against the actual changed binary on an isolated socket. Test standalone and in-app resume, onboarding, active manager, external import and configured alternate Enter actions.

No tests, build, or runtime acceptance checks were run because this task is exploratory and code was not modified. Route next to feature authoring after design direction is accepted.
