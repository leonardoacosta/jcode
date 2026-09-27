# Herdr agent tabs and Jcode version provenance

Exploration date: 2026-09-16 UTC. Status: recommendation, not approved implementation. Scope: agent-card layout, Jcode lifecycle authority, local executable provenance. No source/configuration edits, installs, builds, daemon restarts, or commits were performed. This document is the deliverable under /improve.

## Intent and boundaries

Desired card:

```text
● tab name
  pane name · agent/session name
```

Omit pane name and separator when absent. Remove decorative emoji. Interpret the second dot as a separator, not another status indicator. Preserve explicit human names and existing workspace organization.

This agent has no HERDR_ENV=1, so live Herdr inspection/control was not attempted. Current shell directory /home/nyaptor/dev/jcode is a runtime directory, not a source checkout. Git there resolves to /home/nyaptor. Do not commit or build from it.

## Current evidence and internal prior art

- /home/nyaptor/dev/herdr: Rust/Ratatui Herdr checkout at 57fed7a1, 2026-08-16. A second checkout /home/nyaptor/dev/personal/herdr is at e8ae12bf with unrelated dirty files. Neither is proven to match the installed September binary. Inspect installed command help from an authorized Herdr pane before future runtime work.
- /home/nyaptor/.jcode: restored source checkout, HEAD 59588ec. Its docs/HERDR.md:17-74 recommend session identity plus screen detection until full blocked lifecycle exists.
- /home/nyaptor/dev/personal/dots/dev/jcode: source subtree inside the dots repository, not a standalone Jcode checkout. Enclosing repository HEAD 80172bb42, subtree Cargo version 0.77.0. This explains why a current-looking commit hash does not prove a newer Jcode source lineage.
- /home/nyaptor/dev/jcode-target-v081: preserved source tree, Cargo version 0.81.1, not a working Git repository. Contains a previously implemented native reporter and openspec/changes/add-native-herdr-agent-status/tasks.md. Reuse as reviewed prior art, not as a branch to blindly copy or promote.
- Memory search found no relevant durable memory. Session history contains a September 15 user statement that the server was v0.81.86-dev, but that statement alone is not current binary evidence.

## Track 1: layout

Herdr already has configurable rows. src/config/sidebar.rs:370-450 defines rows, rows_by_agent, and row_gap. src/ui/sidebar/tokens.rs resolves state_icon, tab, pane, agent, terminal_title, terminal_title_stripped, and custom metadata. Missing values disappear. Separators are a space after state_icon and ` · ` between other values.

Candidate configuration, to apply only after resolving which client/server profile owns the visible cards:

```toml
[ui.sidebar.agents]
row_gap = 0
rows = [["state_icon", "tab"], ["pane", "agent"]]
```

Keep existing [ui] status_indicators = "dots". Review per-agent row overrides because they can defeat the shared layout.

Important edge cases:

1. src/ui/sidebar.rs:153-169 deliberately suppresses tab labels for a lone auto-named tab. Thus the candidate can still produce a dot-only first row. For an unconditional title requirement, expose a tab-label fallback rather than relying on naming every tab manually. Preserve old defaults for other users if introducing a new token or opt-in.
2. src/workspace/aggregate.rs:39-67 uses presentation display_agent before registered name, and presentation title before manual pane label. The existing `agent` token is not necessarily a clean Jcode session name. Confirm the user's meaning of “name” against a rendered example. Recommended meaning is the named agent or friendly session, with jcode as a final fallback.
3. terminal_title_stripped removes recognized activity/spinner prefixes, not arbitrary embedded emoji. Do not promise it removes session animals or connection symbols. Jcode's crates/jcode-tui/src/tui/app/tui_lifecycle_runtime.rs:60-119 explicitly chooses connection/session icons for OSC window titles. Prefer explicit clean identity metadata over parsing a decorated title, and scope any icon suppression to the requested Herdr surface.
4. The config files inspected under ~/.config/herdr include custom $hs_group/$hs_tab/$hs_logo/status tokens and Claude/Codex overrides around lines 92-135. These are plugin presentation, not native row defaults. Exact loaded profile and actual emoji producer remain unverified without a live pane. Do not delete plugin installations to fix presentation.
5. Test unnamed tabs, optional pane names, long names, Unicode widths, narrow sidebar truncation, multiple panes/tabs, sorting, selection/hit targets, and all status states. Preserve a textual status surface for users who cannot distinguish dot colors.

Recommendation: configuration-first, with the smallest renderer/token extension only if unconditional tab labels or clean session identity require it. No new sidebar plugin.

## Track 2: lifecycle correctness

Do not create another reporter before recovering and reviewing existing work. The v081 source has crates/jcode-app-core/src/herdr_reporter.rs, turn hooks at agent/turn_execution.rs:151,175, and bash blocking pairing at tool/bash.rs:57-73. It reports source custom:jcode, semantic working/idle/blocked, and release. Herdr derives unseen completion from idle plus seen state. Do not emit done as an independent runtime state or treat unknown as successful completion.

Prior task claims are not fresh verification. The archived-in-place tasks file explicitly substitutes bash stdin blocking for approval/question waits and records working/done observation, but no live blocked acceptance (tasks.md:25-34,48-53). Do not report full acceptance based on its checked boxes.

Prioritized static findings to vet with regression tests before enabling authority:

| Finding | Impact | Effort | Fix risk | Evidence/confidence |
|---|---|---|---|---|
| Registry keys use pane_id only, not socket plus pane identity | Two Herdr servers both using w1:p1 can collide or route ownership incorrectly | M | Medium | herdr_reporter.rs:100-108,239-255, high |
| Identity assigns seq under lock but enqueues after dropping it | Concurrent turn report can overtake identity, contrary to ordered-send claim | M | Medium | herdr_reporter.rs:262-268 versus publish_if_changed:393-413, high |
| Blocking cleanup is code after await, not drop-safe | Aborted/dropped future can leave counted blocked state behind | M | Medium | tool/bash.rs:62-69, high mechanism confidence, runtime reachability needs test |
| last_published is recorded before delivery, responses discarded | Transport/API failures can leave state stale with no retry on identical desired state | M | Medium | herdr_reporter.rs:213-225,393-413, high |
| Fallback uses daemon process HERDR env when task-local is absent | Missing context may misattribute a background/child session | M | Medium | herdr_reporter.rs:154-161, high mechanism confidence |
| Clock-seeded sequence is not durable across clock rollback | Restart monotonicity claim exceeds implementation | M | Medium | herdr_reporter.rs:77-86, high |

Also verify ownership handoff across queued old reports, inherited child environments, close/reload distinctions, disconnect/crash, bounded queue pressure, and stale-authority recovery. Existing source comments are not proof that these are safe.

Recommended status contract: request-scoped client identity, one top-level owner per (Herdr socket, pane), ordered reports, blocked overlay dominating working, idle when ready, release only when authority truly ends, and explicit recovery/expiry when delivery or process health is uncertain. Preserve best-effort behavior without making diagnostics impossible. Keep identity plus screen fallback until the full enabled wait surfaces pass tests.

## External research

Used installed authenticated Firecrawl CLI 1.23.3 through bounded developer searches (3-5 results each) and web searches, on 2026-09-16. Used query-only operations, no fetched result URLs, no scraping, browser interaction, setup, or alternative transport. Indexed passages establish prior art, not present merge/release status. Full-page fetch was not attempted because redirect-hop enforcement could not be established under the local Firecrawl policy.

Primary sources and useful comparisons:

- https://github.com/1jehuang/jcode/issues/750: request for first-class native Herdr state instead of fragile TUI inference.
- https://github.com/1jehuang/jcode/issues/1127: lifecycle contract, pane-local client context, sequence/coexistence, and a reference branch explicitly described as not a PR.
- https://github.com/1jehuang/jcode/pull/758: composable client-scoped lifecycle hooks, linked to Herdr integration PR https://github.com/herdrdev/herdr/pull/2248. Search evidence does not prove the installed binaries contain either change.
- https://herdr.dev/docs/agents/ and https://herdr.dev/docs/agent-automation/: attention rollups and named-worker start/prompt/wait/read workflows.
- https://github.com/yeachan-heo/oh-my-codex/blob/cb955b0d5becbef76d2c1f0096b6e1f238e1e7f7/docs/herdr-bridge.md: opt-in best-effort native authority, durable sequence ordering, canonical lifecycle dispatcher, stale-authority reconciliation.
- https://github.com/ogulcancelik/herdr/issues/49: real false-completion failure from Pi children inheriting the parent's Herdr environment. This is a concrete reason to test non-owner suppression.
- https://github.com/yigitkonur/awesome-herdr: indexed usage examples include Attamusc/pi-herdr state reporting, namtx/pi-herd supervision, automatic descriptive tab naming, and attention-focused sidebars. These are discovery leads, not audited dependencies or endorsements.

No new library/framework is required. Context7 is not applicable to a dependency-free configuration and native protocol exploration, so none was introduced.

## Track 3: version mismatch

Direct --version and symlink observations:

| Surface | Observed target/version |
|---|---|
| ~/.local/bin/jcode | ~/.jcode/builds/current/jcode |
| current and stable | versions/80172bb42-dirty/jcode, v0.77.0-dev |
| shared-server pointer | versions/v081-cursorlink/jcode, v0.81.1-dev (unknown commit) |
| Running processes | Many point to 80172bb42-dirty. One points to a deleted older runtime path. A pointer is not proof of the daemon's loaded image. |

The active default server registry (~/.jcode/servers.json) names PID 3950682 on /run/user/1000/jcode.sock as v0.77.0-dev (80172bb42, dirty). /proc/3950682/exe independently resolves to that same old binary. This is not merely stale client display. Another preserved executable at /home/nyaptor/dev/jcode-target-v081/target/commit-deploy-cargo/release/jcode reports v0.81.11-dev (d2148e7e7). Neither recovered v081 executable proves the historical v0.81.86 build. The v081 tree's .git file points to the missing /home/nyaptor/dev/jcode/.git/worktrees/jcode-target-v081.

The source installer scripts/install_release.sh:1-9 explicitly promotes both current and stable and repoints the launcher. This matches the observed channel shape but does not establish which historical command or actor performed the change. The strongest established cause is a locally installed/restored 0.77 source lineage selected for both PATH and the running default daemon, while newer artifacts remain separately parked.

Current/stable pointers were changed September 15 at 15:49 local filesystem display time. The shared-server pointer was changed at 12:55. Thus new PATH launches really do run the old build. A newer shared-server pointer does not automatically switch existing clients or an already-running daemon. The exact v0.81.86 artifact and its provenance must be verified separately before proposing a safe update target.

Never fix this by changing Cargo.toml's version label. Select the intended source lineage and tested binary, verify client/server protocol compatibility on an isolated socket, then schedule any shared-runtime promotion with explicit approval and session preservation. Do not repoint links or restart the live daemon during exploration.

## Proposed next scope and verification

1. Approve the intended executable/source lineage and recover version provenance. This is prerequisite to meaningful runtime lifecycle testing, not to designing the card.
2. Implement the two-row layout in the actual loaded profile. Use native tokens, remove only decorative presentation, and handle missing tab/name values explicitly.
3. Review and harden existing native reporter rather than add a competing one. Preserve legacy hook coexistence and screen fallback during rollout.
4. From a genuine Herdr pane, verify two clients on one daemon, two Herdr sockets with identical pane IDs, nested/headless workers, begin/end/error/abort, blocked enter/exit, disconnect/reload/close, and unseen completion viewed by two clients.

Herdr checks from justfile: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo nextest run --locked with sidebar/token filters, python3 -m unittest scripts.test_ui_hot_path_architecture, then relevant integration-asset tests. For restored Jcode: cargo test -p jcode-app-core herdr and focused blocking/ownership tests after confirming the authoritative checkout, plus its build on an isolated target/socket. No test/build pass is claimed in this exploration.

## Requirement-to-evidence acceptance ledger

| Requirement | Check and observed result | Acceptance status |
|---|---|---|
| First row has status dot plus tab name | Read actual Herdr row resolver and tab-label producer. Resolver supports state_icon + tab, but producer suppresses a lone auto-named tab. | Recommendation supported, rendered result NOT verified. Live acceptance blocked by missing HERDR_ENV and exploration-only scope. |
| Second row has optional pane name, separator, and clean name | Read actual token separator/elision and metadata precedence. Missing pane token disappears, separator is emitted only between present values. Display-agent metadata can override the clean registered name. | Configuration candidate is not a proven emoji-free result. Requires live identity and profile inspection. |
| Remove emoji | Read Jcode OSC title producer and Herdr stripped-title contract. Connection/session icons are deliberately added, while stripped-title handles recognized activity prefixes only. | No change made. Actual visible emoji source remains unverified. Do not claim fixed. |
| Improve working/blocked/ready/completion indication | Inspected preserved native emitter and real call sites, including turn hooks and bash stdin wait. Existing task record lacks live blocked acceptance. Identified ordering, cancellation, identity, and stale-delivery risks. | Improvement plan only. No before/after runtime observation, no passing acceptance claim. |
| Use Firecrawl for latest lifecycle and other usage | Executed installed Firecrawl 1.23.3 public developer/search CLI with bounded result counts. Queries succeeded and returned the cited primary-source passages and ecosystem examples. | Research operation verified. Latest merged implementation/release state NOT verified from indexed passages. |
| Explain current v0.77 despite newer builds | Executed real installed binaries with --version. Current reports v0.77.0-dev, shared-server target reports v0.81.1-dev, preserved executable reports v0.81.11-dev. Default server registry identifies PID 3950682 as v0.77, independently matched by /proc executable. | Current mismatch explanation verified through real executable interfaces and OS process evidence. Historical v0.81.86 artifact and exact promotion cause unresolved. |
| Deliver exploration rather than implement | Saved this record under plans/ only. No source/configuration/build/promotion/restart operations performed. | Exploration artifact delivered, not a shipped improvement. |

No acceptance test is appropriate that changes a live pane or shared daemon from this context. HERDR_ENV is unset and the active Herdr skill explicitly prohibits live session inspection/control outside Herdr. A synthetic rendering harness would not establish the user's actual loaded profile or visible card, so it was not substituted for acceptance. Further public-interface checks require resuming from an authorized Herdr pane and, for implementation, a separately approved change. The real CLI version checks above do close the current-version diagnostic question, but not the UI/lifecycle outcome.

Stop conditions: no live Herdr context, source/build lineage divergence, unverified exact version artifact, and unverified installed renderer. These block deployment acceptance, not the recommendations above. Full security/performance/dependency audit of either project is out of scope.
